use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use miette::Result;
use parking_lot::RwLock;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};
use std::sync::Arc;
//use time::format_description;
use tokio::sync::mpsc::Sender;
use tracing::{error, instrument};

use crate::events::{Event, Outcome};

pub struct Home {
    _event_sender: Sender<Event>,
    title: String,
    status: String,
    list_state: Arc<RwLock<ListState>>,
}

impl Home {
    pub fn new(event_sender: Sender<Event>) -> Self {
        Self {
            _event_sender: event_sender,
            title: String::new(),
            status: String::new(),
            list_state: Arc::new(RwLock::new(ListState::default())),
        }
    }

    #[instrument(name = "home", skip_all)]
    pub async fn start(&mut self) -> Result<()> {
        self.title = "Not logged in".to_string();
        Ok(())
    }

    #[instrument(name = "home::handle_event", skip_all)]
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Crossterm(event) => {
                if let CrosstermEvent::Key(key) = *event {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            if let Err(err) = self.scroll_down().await {
                                error!("failed to scroll down: {:#}", err);
                            }
                        }
                        (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            if let Err(err) = self.scroll_up().await {
                                error!("failed to scroll up: {:#}", err);
                            }
                        }
                        _ => return Outcome::Ignored,
                    }
                }
                Outcome::Handled
            }
            _ => Outcome::Ignored,
        }
    }

    async fn scroll_down(&mut self) -> Result<()> {
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        let index = list_state.selected().map_or(0, |s| s + 1);
        list_state.select(Some(index));
        Ok(())
    }

    async fn scroll_up(&mut self) -> Result<()> {
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        let index = list_state.selected().unwrap_or(0);
        list_state.select(Some(index));
        Ok(())
    }

    fn update_status(&mut self, selected: usize) {}

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    #[instrument(name = "home::draw", skip_all)]
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = vec![];
        // this looks great on a dark theme, but not so much on a light one
        let style = Style::default().bg(Color::Rgb(16, 32, 64));
        let list = List::new(items).highlight_style(style);
        // let mut state = ListState::default();
        // state.select(Some(self.selected));
        let list_state = Arc::clone(&self.list_state);
        let mut state = list_state.write();
        frame.render_stateful_widget(list, area, &mut state);
    }
}
