use actix_cors::Cors;
use actix_files::Files;
use actix_htmx::HtmxMiddleware;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_static_files::ResourceFiles;
use clearscreen;
use std::convert::Infallible;
use std::ffi::OsString;
use std::io::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, str};
use tokio;
use tokio::process::Command;
use watchexec::handler::SyncFnHandler;
use watchexec::ErrorHook;
use watchexec::{
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    error::ReconfigError,
    event::Event,
};
use watchexec_filterer_globset::GlobsetFilterer;
use watchexec_signals::Signal;

mod websocket;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub async fn filt(filters: &[&str], ignores: &[&str], extensions: &[&str]) -> GlobsetFilterer {
    //let origin = std::fs::canonicalize(".").expect("failed to canonicalize path");
    //// Don't know why this throws
    let mut path = PathBuf::new();
    path.push(".");
    GlobsetFilterer::new(
        path,
        filters.iter().map(|s| ((*s).to_string(), None)),
        ignores.iter().map(|s| ((*s).to_string(), None)),
        vec![],
        extensions.iter().map(OsString::from),
    )
    .await
    .expect("making filterer")
}

//#[actix_rt::main]
pub async fn server(port: u16) -> Result<(), Error> {
    let reloader = websocket::Reload::new();
    let ch = reloader.clone();
    let mut runtime = RuntimeConfig::default();
    let mut init = InitConfig::default();

    if !std::path::Path::new("./output").exists() {
        std::fs::create_dir("./output").expect("failed creating output directory");
    }

    init.on_error(SyncFnHandler::from(
        |err: ErrorHook| -> std::result::Result<(), Infallible> {
            eprintln!("Watchexec Runtime Error: {}", err.error);
            Ok(())
        },
    ));

    runtime
        .pathset(["trees", "theme", "assets"])
        .filterer(Arc::new(
            filt(&[], &["__latexindent*"], &["tree", "js", "css", "xsl"]).await,
        ))
        .on_action(move |action: Action| {
            log::info!("hello?");
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
                                //log::info!("{}", String::from_utf8(output.stdout).unwrap());
                                clearscreen::clear().expect("failed to clear screen");
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

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("tauri://localhost")
            .allowed_methods(vec!["GET", "POST"]);
        let generated = generate();

        App::new()
            .service(ResourceFiles::new("/", generated))
            .wrap(HtmxMiddleware)
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(reloader.clone()))
            .service(new_tree)
            .route("/reload", web::get().to(websocket::ws))
        //.service(get_reload_file)
        //.service(reload) //.to_string();)
        //.service(index)
        //.service(Files::new("/", "./output")) //.index_file("index.html"))
    })
    .bind(("127.0.0.1", port))
    .expect("Failed to bind addr")
    .run()
    .await
    //let _ = tokio::join!(
    //    server
    //        .bind(("127.0.0.1", args.port))
    //        .expect("Failed to bind addr")
    //        .run(),
    //    {
    //        log::info!("Starting watchexec");
    //        Watchexec::new(init, runtime.clone()).unwrap().main()
    //    },
    //);

    //Ok(())
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
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
