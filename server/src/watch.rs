use axum::response::sse;
use core::fmt::Debug;
use log::{debug, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use tokio::sync::broadcast::Sender;
use watchexec::{
    command::Command, command::Program, error::RuntimeError, filter::Filterer, job::CommandState,
    Id, Watchexec,
};
use watchexec_events::filekind::FileEventKind::Modify;
use watchexec_events::filekind::ModifyKind::Data;
use watchexec_events::Tag::FileEventKind;
use watchexec_events::{Event, Priority, ProcessEnd};
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
    pub async fn run(dir: String, sender: Sender<sse::Event>) {
        let build_id = Id::default();

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
                        info!("[Quitting...]");
                        action.quit();
                        return action;
                    }
                    let build = action.get_or_create_job(build_id, || {
                        Arc::new(Command {
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
                        })
                    });
                    if action.paths().next().is_some()
                        || action.events.iter().any(|event| event.tags.is_empty())
                    {
                        build.restart().await;
                    }
                    build.to_wait().await;
                    build.run(move |context| {
                        if let CommandState::Finished {
                            status: ProcessEnd::Success,
                            ..
                        } = context.current
                        {
                            info!("{:?}", context.current);
                            match sender.send(sse::Event::default().data("Hello")) {
                                Ok(r) => {
                                    debug!("{:?}", r);
                                    info!("Sent reload message");
                                }
                                Err(_) => {}
                            };
                        }
                    });
                    action
                })
            }
        })
        .unwrap();

        wx.config.pathset([dir]);
        wx.config.filterer(MyFilterer {});

        let main = wx.main();

        let _ = main.await;

        wx.send_event(Event::default(), Priority::Urgent)
            .await
            .unwrap();
    }
}
