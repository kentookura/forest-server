use axum::response::sse;
use core::fmt::Debug;
use log::{debug, error, info};
use miette::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use watchexec::{
    command::Command, command::Program, error::RuntimeError, filter::Filterer, Watchexec,
};
use watchexec_events::filekind::FileEventKind::Modify;
use watchexec_events::filekind::ModifyKind::Data;
use watchexec_events::Tag::FileEventKind;
use watchexec_events::{Event, Priority};
use watchexec_signals::Signal;

pub struct Watcher {
    pub watchexec: Arc<Watchexec>,
}

#[derive(Debug)]
struct MyFilterer;

impl Filterer for MyFilterer {
    fn check_event(&self, event: &Event, _: Priority) -> Result<bool, RuntimeError> {
        let evt = event.clone();
        Ok(evt
            .tags
            .into_iter()
            .any(|tag| matches!(tag, FileEventKind(Modify(Data(_))))))
    }
}

impl Watcher {
    pub async fn run(dir: String, sender: Sender<sse::Event>) -> Result<()> {
        let dir = dir.clone();
        let sender = sender.clone();

        let wx = Watchexec::new_async({
            let dr = dir.clone();
            let sender = sender.clone();

            move |mut action| {
                let dr = dr.clone();
                let sender = sender.clone();

                Box::new(async move {
                    if action.signals().any(|sig| sig == Signal::Interrupt) {
                        info!("Quitting Wachexec");
                        info!("[BUG! Press CTRL-C again to exit]");
                        action.quit();
                        return action;
                    }

                    let forester = Arc::new(Command {
                        program: Program::Exec {
                            prog: PathBuf::from("forester"),
                            args: vec![
                                "build".to_string(),
                                "--dev".to_string(),
                                "--root".to_string(),
                                "index".to_string(),
                                dr.to_string(),
                            ],
                        },
                        options: Default::default(),
                    });

                    if action.paths().next().is_some()
                        || action.events.iter().any(|event| event.tags.is_empty())
                    {
                        let output = forester.to_spawnable().output().await.unwrap();
                        let sout = String::from_utf8(output.stdout).expect("Output not UTF8");
                        if output.status.success() {
                            info!("Build Succeeded!");
                            match sender.send(sse::Event::default().data("reload")) {
                                Ok(r) => {
                                    debug!("{:?}", r);
                                    info!("Reloading");
                                }
                                Err(_) => {
                                    error!("Error sending message!");
                                }
                            };
                        } else {
                            error!("{}", sout);
                            match sender.send(sse::Event::default().data(sout)) {
                                Ok(r) => {
                                    debug!("{:?}", r);
                                }
                                Err(_) => {
                                    error!("Error sending message!");
                                }
                            };
                        }
                    }
                    action
                })
            }
        })
        .unwrap();

        wx.config.pathset([dir]);
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
        //.unwrap();
    }
}
