use actix_files::{Files, NamedFile};
use actix_rt::Arbiter;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{Message, Session};
use async_priority_channel as priority;
use futures::stream::{FuturesUnordered, StreamExt};
use log::info;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio;
//use tokio::process::Command;
use tokio::sync::Mutex;
use watchexec::command::Command;
use watchexec::{
    self,
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    error::ReconfigError,
    event::{Event, Priority, Tag},
    Watchexec,
};
use watchexec_filterer_globset::GlobsetFilterer;
use watchexec_signals::Signal;

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

async fn index() -> impl Responder {
    NamedFile::open_async("./output/index.xml").await.unwrap()
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

    let reloader = Reload::new();
    let ch = reloader.clone();
    let init = InitConfig::default();
    let mut runtime = RuntimeConfig::default();
    runtime
        .pathset(["trees", "theme", "assets"])
        .filterer(Arc::new(
            filt(&[], &[], &["tree", "js", "css", "xsl"]).await,
        ))
        .command(
            Command::Exec {
                prog: "forester".into(),
                args: vec![
                    "build".to_string(),
                    "--dev".to_string(),
                    "--root".to_string(),
                    "index".to_string(),
                    "trees".to_string(),
                ],
            }, //("forester")
               //    .arg("build")
               //    .arg("--dev")
               //    .arg("--root")
               //    .arg("index")
               //    .arg("trees")
               //    .status()
               //    .await
               //    .unwrap();
        )
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
                    log::info!("detected changes");
                    //Command::Exec {
                    //  prog: "forester".into(),
                    //};
                    //("forester")
                    //    .arg("build")
                    //    .arg("--dev")
                    //    .arg("--root")
                    //    .arg("index")
                    //    .arg("trees")
                    //    .status()
                    //    .await
                    //    .unwrap();
                    ch.send("reload".into()).await;
                }
                Ok::<(), ReconfigError>(())
            }
        });
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(reloader.clone()))
            .route("/", web::get().to(index))
            .route("/reload", web::get().to(ws))
            .service(Files::new("/", "./output"))
    });
    //.bind("127.0.0.1:8080");

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
