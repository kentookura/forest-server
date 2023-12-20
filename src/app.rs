use crate::events::Outcome;
use crate::events::{Event, Events};
use crate::logging::LogMessage;
use crate::root::Root;
use crate::server::*;
use crate::ui::UI;
use crate::watch::*;
use axum::response::sse::Event as SseEvent;
use crossterm::event::{Event::Key, KeyCode, KeyCode::Char};
use log::{debug, error, info, trace};
use miette::Result;
use parking_lot::Mutex;
//use std::process::exit;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct App {
    pub port: u16,
    events: Events,
    ui: UI,
    root: Root,
    watcher: Watcher,
    server: Server,
    forester_args: String,
}

pub async fn run(port: u16, logs: Arc<Mutex<Vec<LogMessage>>>, forester_args: &str) -> Result<()> {
    let mut app = App::new(port, logs, forester_args).unwrap();
    app.run().await.unwrap();
    Ok(())
}

impl App {
    pub fn new(port: u16, logs: Arc<Mutex<Vec<LogMessage>>>, args: &str) -> Result<Self> {
        let (tx, rx) = broadcast::channel::<SseEvent>(100);
        let events = Events::new();
        let root = Root::new(events.tx.clone(), logs);
        let ui = UI::new().expect("Initializing UI failed");
        let watcher = Watcher::new(tx, args);
        let server = Server::new(port, rx);

        Ok(Self {
            port,
            events,
            ui,
            root,
            watcher,
            server,
            forester_args: args.to_string(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        //tokio::join!(
        //    self.watcher.run(),
        //    self.server.run(),
        //);

        self.ui.start().expect("failed to start UI");
        self.events.start().expect("failed to start events");
        self.root
            .start()
            .await
            .expect("Failed to start root component");

        self.main_loop().await.expect("Main loop failed");

        //if tokio::signal::ctrl_c().await.is_ok() {
        //    exit(0)
        //}
        info!("Shutting down");

        Ok(())
    }

    pub async fn main_loop(&mut self) -> Result<()> {
        loop {
            self.ui.draw(|f| {
                self.root.draw(f, f.size());
            })?;

            match self.events.next().await {
                Some(Event::Quit) => {
                    info!("Received quit event");
                    break;
                }
                Some(Event::Tick) => {
                    //trace!("Received tick event");
                    self.root.handle_event(&Event::Tick).await;
                }
                Some(event) => {
                    if let Event::Crossterm(Key(key)) = event {
                        match key.code {
                            KeyCode::Esc => {
                                debug!("Received quit key");
                                break;
                            }
                            KeyCode::Char('?') => {
                                debug!("Show help!");
                                break
                            }
                            KeyCode::Char('q') => {
                                break;
                            }
                            _ => {
                                debug!("Unhandled")
                            }
                        }
                    };
                    if self.root.handle_event(&event).await == Outcome::Handled {
                        trace!("Event handled by root component: {:?}", event);
                        continue;
                    }
                    if let Event::Crossterm(Key(key)) = event {
                        if key.code == Char('q') {
                            debug!("Received quit key");
                            break;
                        }
                    }
                }
                None => {
                    error!("Event channel closed. Exiting as we won't receive any more events.");
                    break;
                }
            }
        }
        Ok(())
    }
}
