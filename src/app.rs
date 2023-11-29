use crate::server::*;
use crate::watch::*;
use axum::response::sse::Event;
//use crate::help_line::HelpItem;
//use crossterm::event::DisableMouseCapture;
//use crossterm::terminal::LeaveAlternateScreen;
//use crossterm::ExecutableCommand;
//use crossterm::{
//    execute,
//    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
//};
use miette::Result;
//use ratatui::prelude::*;
//use ratatui::widgets::{List, ListItem, Paragraph};
//use std::io::stdout;
use std::process::exit;
use tokio::sync::broadcast;

pub struct Application {
    port: u16,
    tree_dir: String,
}

impl Application {
    pub fn new(port: u16, dir: String) -> Application {
        Application {
            port,
            tree_dir: dir,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let dir = self.tree_dir.clone();
        let (tx, rx) = broadcast::channel::<Event>(100);

        let backend = async move {
            server(self.port, rx).await;
        };

        let watcher = async {
            Watcher::run(dir, tx).await;
        };

        tokio::join!(backend, watcher);

        if tokio::signal::ctrl_c().await.is_ok() {
            exit(0)
        }

        Ok(())
        /*

        stdout()
            .execute(EnterAlternateScreen)
            .expect("failed to enter alternate screen");
        enable_raw_mode().expect("failed to enable raw mode");
        let mut terminal =
            Terminal::new(CrosstermBackend::new(stdout())).expect("failed to get terminal");
        terminal.clear().expect("failed to clear terminal");

        // TODO main loop
        loop {
            terminal
                .draw(|frame| {
                    let main_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(frame.size());

                    let mut list_items = Vec::<ListItem>::new();
                    let help_items: Vec<HelpItem> = vec![
                        HelpItem::new('q', "to quit".to_string()),
                        HelpItem::new('n', "to create a new tree".to_string()),
                        HelpItem::new('h', "for help".to_string()),
                    ];

                    for item in help_items {
                        list_items.push(ListItem::new(Span::styled(
                            format!("{} {}", item.key, item.help),
                            Style::default(),
                        )))
                    }
                    let help_line = List::new(list_items);
                    let help_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(vec![Constraint::Percentage(100)])
                        .split(main_layout[1]);
                    frame.render_widget(help_line, help_layout[0]);

                    frame.render_widget(
                        Paragraph::new(format!("Server running at localhost:{}", self.port)),
                        main_layout[0],
                    );
                })
                .unwrap();

            if crossterm::event::poll(std::time::Duration::from_millis(250)).unwrap() {
                // If a key event occurs, handle it
                if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        match key.code {
                            crossterm::event::KeyCode::Char('n') => break,
                            crossterm::event::KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        */
    }
}

/*
enum Event {
    Key(crossterm::event::KeyEvent),
}

struct EventHandler {
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(250);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            loop {
                if crossterm::event::poll(tick_rate).unwrap() {
                    match crossterm::event::read().unwrap() {
                        crossterm::event::Event::Key(e) => {
                            if e.kind == KeyEventKind::Press {
                                tx.send(Event::Key(e)).unwrap()
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        });
        EventHandler { rx }
    }
    async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }
}
*/
