use color_eyre::eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{fs, process::Output};
use std::{
    io::{stdout, Error},
    path::PathBuf,
};
use std::{process::exit, str};
use tokio::process::Command;

use crate::build::*;
use crate::help_line::HelpItem;
use crate::server::*;
use crate::Args;

pub struct App {
    port: u16,
    errors: Vec<String>,
    tree_dir: String,
    root: String,
    current_screen: CurrentScreen,
}

enum CurrentScreen {
    Init,
    Watching,
    CreatingNewTree,
}

enum ForestError {
    TreeNotFound,
}

impl App {
    pub fn new(port: u16, root: String, dir: String) -> App {
        App {
            port: port,
            root: root,
            errors: vec![],
            tree_dir: dir,
            current_screen: CurrentScreen::Init,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        tokio::spawn(server(self.port));

        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        match fs::metadata(format!("{}/index.tree", &self.tree_dir)) {
            Ok(_) => {}
            Err(err) => {}
        }
        let mut counter = 0;

        // TODO main loop
        loop {
            terminal.draw(|frame| {
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
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(250))? {
                // If a key event occurs, handle it
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        match key.code {
                            crossterm::event::KeyCode::Char('j') => counter += 1,
                            crossterm::event::KeyCode::Char('k') => counter -= 1,
                            crossterm::event::KeyCode::Char('n') => break,
                            crossterm::event::KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );

        Ok(())
    }
}

//enum Event {
//    Key(crossterm::event::KeyEvent),
//}
//
//struct EventHandler {
//    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
//}

//impl EventHandler {
//    fn new() -> Self {
//        let tick_rate = std::time::Duration::from_millis(250);
//        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//        tokio::spawn(async move {
//            loop {
//                if crossterm::event::poll(tick_rate).unwrap() {
//                    match crossterm::event::read().unwrap() {
//                        crossterm::event::Event::Key(e) => {
//                            if e.kind == KeyEventKind::Press {
//                                tx.send(Event::Key(e)).unwrap()
//                            }
//                        }
//                        _ => unimplemented!(),
//                    }
//                }
//            }
//        });
//        EventHandler { rx }
//    }
//    async fn next(&mut self) -> Result<Event> {
//        self.rx
//            .recv()
//            .await
//            .ok_or(color_eyre::eyre::eyre!("Unable to get event"))
//    }
//}
