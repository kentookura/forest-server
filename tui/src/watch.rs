use crate::Broadcaster;
use actix_web::web;
use clearscreen;
use std::convert::Infallible;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
use tokio;
use tokio::process::Command;
use watchexec::handler::SyncFnHandler;
use watchexec::{
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    error::{CriticalError, ReconfigError},
    event::Event,
};
use watchexec::{ErrorHook, Watchexec};
use watchexec_filterer_globset::GlobsetFilterer;
use watchexec_signals::Signal;

pub async fn filt(filters: &[&str], ignores: &[&str], extensions: &[&str]) -> GlobsetFilterer {
    // Don't know why this throws
    //let origin = std::fs::canonicalize(".").expect("failed to canonicalize path");
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

//pub async fn watch(dir: String, broadcaster: web::Data<Broadcaster>) -> Result<(), CriticalError> {
pub async fn watch(dir: String, broadcaster: web::Data<Broadcaster>) -> Arc<Watchexec> {
    let mut init = InitConfig::default();
    init.on_error(SyncFnHandler::from(
        |err: ErrorHook| -> std::result::Result<(), Infallible> {
            eprintln!("Watchexec Runtime Error: {}", err.error);
            Ok(())
        },
    ));

    let mut runtime = RuntimeConfig::default();

    runtime
        .pathset([dir.clone(), "theme".to_string(), "assets".to_string()])
        //.filterer(Arc::new(
        //    filt(&[], &["__latexindent*"], &["tree", "js", "css", "xsl"]).await,
        //))
        .on_action(move |action: Action| {
            //let ch = ch.clone();
            let evs = action.events.clone();
            let bcstr = broadcaster.clone();
            let dir = dir.clone();
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
                        .args(&["build", "--dev", "--root", "index", &dir])
                        .output()
                        .await
                    {
                        Ok(output) => {
                            log::info!("{:?}", output);
                            if output.status.success() {
                                //log::info!("{}", String::from_utf8(output.stdout).unwrap());
                                clearscreen::clear().expect("failed to clear screen");
                                log::info!("Reloading...");
                                //ch.send("reload".into()).await;
                                bcstr.broadcast("Build Succeeded;").await;
                            } else {
                                let msg = str::from_utf8(&output.stdout).unwrap();
                                log::error!("\n{}", msg);
                                //ch.send(msg.into()).await;
                            }
                        }
                        Err(_) => {
                            log::error!("Error!");
                        }
                    }
                }
                Ok::<(), ReconfigError>(())
            }
        });
    Watchexec::new(init, runtime.clone()).unwrap()
    //we.reconfigure(runtime);
    //we.main().await.unwrap().expect("Failed to start watchexec");
    //Ok(())
}
