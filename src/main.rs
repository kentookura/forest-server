use actix_files::{Files, NamedFile};
use actix_web::http::{header, StatusCode};
use actix_web::{
    get, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpResponseBuilder,
    HttpServer, Responder,
};
use actix_ws::{Message, Session};
use clap::Parser;
use futures::stream::{FuturesUnordered, StreamExt};
use log::info;
use miette::IntoDiagnostic;
use serde::Deserialize;
use std::convert::Infallible;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Error, Write};
use std::str;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio;
use tokio::process::Command;
use tokio::sync::Mutex;
use watchexec::handler::SyncFnHandler;
use watchexec::ErrorHook;
//use watchexec::command::Command;
use watchexec::{
    self,
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    error::ReconfigError,
    event::Event,
    Watchexec,
};
use watchexec_filterer_globset::GlobsetFilterer;
use watchexec_signals::Signal;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short, long)]
    trees: Option<String>,
}

#[derive(Clone)]
struct Reload {
    inner: Arc<Mutex<ReloadInner>>,
}

struct ReloadInner {
    sessions: Vec<Session>,
}

impl Reload {
    fn new() -> Self {
        Reload {
            inner: Arc::new(Mutex::new(ReloadInner {
                sessions: Vec::new(),
            })),
        }
    }

    async fn insert(&self, session: Session) {
        self.inner.lock().await.sessions.push(session);
    }

    async fn send(&self, msg: String) {
        let mut inner = self.inner.lock().await;
        let mut unordered = FuturesUnordered::new();

        for mut session in inner.sessions.drain(..) {
            let msg = msg.clone();
            unordered.push(async move {
                let res = session.text(msg).await;
                res.map(|_| session).map_err(|_| info!("Dropping session"))
            });
        }

        while let Some(res) = unordered.next().await {
            if let Ok(session) = res {
                inner.sessions.push(session);
            }
        }
    }
}

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    reloader: web::Data<Reload>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    reloader.insert(session.clone()).await;
    info!("Inserted session");

    let alive = Arc::new(Mutex::new(Instant::now()));

    let mut session2 = session.clone();
    let alive2 = alive.clone();

    actix_rt::spawn(async move {
        let mut interval = actix_rt::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if session2.ping(b"").await.is_err() {
                break;
            }

            if Instant::now().duration_since(*alive2.lock().await) > Duration::from_secs(10) {
                let _ = session2.close(None).await;
                break;
            }
        }
    });

    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(s) => {
                    info!("Relaying text, {}", s);
                    let s: &str = s.as_ref();
                    reloader.send(s.into()).await;
                }
                Message::Close(reason) => {
                    let _ = session.close(reason).await;
                    info!("Got close, bailing");
                    return;
                }
                Message::Continuation(_) => {
                    let _ = session.close(None).await;
                    info!("Got continuation, bailing");
                    return;
                }
                Message::Pong(_) => {
                    *alive.lock().await = Instant::now();
                }
                _ => (),
            };
        }
        let _ = session.close(None).await;
    });
    info!("Spawned");

    Ok(response)
}

pub async fn filt(filters: &[&str], ignores: &[&str], extensions: &[&str]) -> GlobsetFilterer {
    let origin = tokio::fs::canonicalize(".").await.unwrap();
    GlobsetFilterer::new(
        origin,
        filters.iter().map(|s| ((*s).to_string(), None)),
        ignores.iter().map(|s| ((*s).to_string(), None)),
        vec![],
        extensions.iter().map(OsString::from),
    )
    .await
    .expect("making filterer")
}

//#[actix_rt::main]
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let args = Args::parse();

    let reloader = Reload::new();
    let ch = reloader.clone();
    let mut init = InitConfig::default();
    init.on_error(SyncFnHandler::from(
        |err: ErrorHook| -> std::result::Result<(), Infallible> {
            eprintln!("Watchexec Runtime Error: {}", err.error);
            Ok(())
        },
    ));
    let mut runtime = RuntimeConfig::default();
    runtime
        .pathset(["trees", "theme", "assets"])
        .filterer(Arc::new(
            filt(&[], &["__latexindent*"], &["tree", "js", "css", "xsl"]).await,
        ))
        .on_action(move |action: Action| {
            let ch = ch.clone();
            let evs = action.events.clone();
            async move {
                let sigs = action
                    .events
                    .iter()
                    .flat_map(Event::signals)
                    .collect::<Vec<_>>();
                if sigs.iter().any(|sig| sig == &Signal::Interrupt) {
                    action.outcome(Outcome::Exit);
                }

                for event in evs.iter() {
                    log::info!("{:?}", event);
                    match Command::new("forester")
                        .args(&["build", "--dev", "--root", "index", "trees"])
                        .output()
                        .await
                    {
                        Ok(output) => {
                            log::info!("{:?}", output);
                            if output.status.success() {
                                log::info!("{}", String::from_utf8(output.stdout).unwrap());
                                log::info!("Reloading...");
                                ch.send("reload".into()).await;
                            } else {
                                let msg = str::from_utf8(&output.stdout).unwrap();
                                log::error!("\n{}", msg);
                                ch.send(msg.into()).await;
                            }
                        }
                        Err(_) => {
                            log::error!("Error!")
                        }
                    }
                    //.expect("Failed to run forester. Is the program installed?");
                }
                Ok::<(), ReconfigError>(())
            }
        });
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(reloader.clone()))
            .service(new_tree)
            //.route("/", web::get().to(index))
            .route("/reload", web::get().to(ws))
            .service(get_reload_file)
            .service(Files::new("/", "./output").index_file("index.xml"))
    });

    tokio::spawn(
        server
            .bind("127.0.0.1:8080")
            .expect("Failed to bind addr")
            .run(),
    );

    let we = Watchexec::new(init, runtime.clone()).unwrap();
    tokio::spawn(we.main()).await?.unwrap();
    //tokio::spawn(we.main()).await?.unwrap();

    Ok(())
}

#[derive(Deserialize)]
struct New {
    prefix: String,
}

#[post("/tree")]
async fn new_tree(new: String) -> impl Responder {
    match Command::new("forester")
        .args(&[
            "new",
            "--dir",
            "trees",
            "--prefix",
            new.as_ref(), //.appendChild(new.prefix.into()),
        ])
        .output()
        .await
    {
        Ok(output) => HttpResponse::Created().body(output.stdout),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    };
    HttpResponse::Accepted()
}

#[get("/reload.js")]
async fn get_reload_file() -> impl Responder {
    let js = "
var ws = null;
var overlay = null;

function addElement(msg) {
  // create a new div element
  const newDiv = document.createElement(\"div\");

  // and give it some content
  const newContent = document.createTextNode(msg);

  // add the text node to the newly created div
  newDiv.appendChild(newContent);

  // add the newly created element and its content into the DOM
  const currentDiv = document.getElementById(\"div1\");
  document.body.insertBefore(newDiv, currentDiv);
}



function connect() {
  const { location } = window;
  const proto = location.protocol.startsWith(\"https\") ? \"wss\" : \"ws\";
  const uri = `${proto}://${location.host}/reload`;
  ws = new WebSocket(uri);

  ws.onmessage = function (e) {
    //console.log(\"message:\", e.data);
    if (e.data == \"reload\") {
      location.reload();
    } else {
      //console.log(e.data)
      addElement(e.data)
    }
      
  };

  ws.onopen = function () {
    console.log(\"live reload: connected to websocket\");
  };

  ws.onclose = function (e) {
    console.log(
      \"socket is closed. reconnect will be attempted in 1 second.\",
      e.reason
    );
    ws = null;

    setTimeout(function () {
      connect();
    }, 100);
  };
  ws.onerror = function (err) {
    console.error(\"socket encountered error: \", err.message, \"closing socket\");
  };
}
connect();"
        .to_string();
    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::APPLICATION_JAVASCRIPT))
        .body(js)
}
