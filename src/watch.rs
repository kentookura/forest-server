use axum::response::sse;
use core::fmt::Debug;
use log::{debug, error, info};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use watchexec::{
    command::Command, command::Program, error::RuntimeError, filter::Filterer, Watchexec,
};
use watchexec_events::Tag::Path;
use watchexec_events::{Event, Priority};
use watchexec_signals::Signal;

use notify_rust::Notification;

pub struct Watcher {
    pub watchexec: Arc<Watchexec>,
}

#[derive(Debug)]
struct MyFilterer;

impl Filterer for MyFilterer {
    fn check_event(&self, event: &Event, _: Priority) -> Result<bool, RuntimeError> {
        let evt = event.clone();
        Ok(evt.tags.clone().into_iter().any(|tag| {
            if let Path {
                ref path,
                file_type: _,
            } = tag
            {
                path.extension().is_some_and(|ext| ext == "tree")
            } else {
                false
            }
        }))
    }
}

fn proper_pathset() -> io::Result<Vec<String>> {
    let r = fs::read_dir(".")?
        .map(|res| res.map(|e| e.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>, io::Error>>();
    r.map(|v| {
        v.into_iter()
            .filter(|path| {
                !path.contains(".git")
                    && !path.contains(".hg")
                    && !path.contains("node_modules")
                    && path != "output"
                    && path != "assets"
                    && path != "theme"
            })
            .collect()
    })
}

impl Watcher {
    pub async fn run(sender: Sender<sse::Event>, forester_args: &str) -> Result<()> {
        let args = shlex::split(forester_args).expect("failed to parse arguments for forester");
        let sender = sender.clone();

        let wx = Watchexec::new_async({
            let sender = sender.clone();

            move |action| {
                let sender = sender.clone();
                let args = args.clone();

                Box::new(async move {
                    if action.signals().any(|sig| sig == Signal::Interrupt) {
                        info!("Goodbye!");
                        std::process::exit(0);
                    }
                    let args = args.clone();

                    let forester = Arc::new(Command {
                        program: Program::Exec {
                            prog: PathBuf::from("forester"),
                            args,
                        },
                        options: Default::default(),
                    });

                    if action.paths().next().is_some()
                        || action.events.iter().any(|event| event.tags.is_empty())
                    {
                        let output = forester.to_spawnable().output().await.unwrap_or_else(|e| {
                            error!("Command forester should be installed");
                            error!("{}", e);
                            exit(1)
                        });
                        let stdout = String::from_utf8(output.stdout).expect("Output not UTF8");
                        if output.status.success() {
                            info!("Build Succeeded!");
                            match sender.send(sse::Event::default().data("reload")) {
                                Ok(r) => {
                                    debug!("{:?}", r);
                                    info!("Reloading");
                                }
                                Err(e) => {
                                    error!("Error sending message: {}", e);
                                }
                            };
                        } else {
                            Notification::new().summary("Build failed!").show().unwrap();
                            println!("\n{}", stdout);
                            match sender.send(sse::Event::default().data(stdout)) {
                                Ok(r) => {
                                    debug!("{:?}", r);
                                }
                                Err(e) => {
                                    error!("Error sending message: {}", e);
                                }
                            };
                        }
                    }
                    action
                })
            }
        })
        .into_diagnostic()?;

        let set = proper_pathset().expect("cannot open current directory");
        wx.config.pathset(set);
        wx.config.filterer(MyFilterer {});

        let main = wx.main();

        let _ = wx
            .send_event(Event::default(), Priority::Urgent)
            .await
            .map(|_| {
                info!("Initial Build:");
            });

        let _result = main.await;
        Ok(())
    }
}
