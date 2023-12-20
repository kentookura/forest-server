use crate::{
    events::{Event, Outcome},
    home::Home,
    logging::{LogMessage, LogWidget},
    widgets::{StatusBar, TitleBar},
};
use parking_lot::Mutex;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

enum State {
    Home,
}

pub struct Root {
    _event_sender: Sender<Event>,
    state: State,
    //authentication: Authentication,
    home: Home,
    logs: Arc<Mutex<Vec<LogMessage>>>,
    show_logs: bool,
}

/// The root component is the top-level component of the application.
/// It is responsible for starting and stopping all other components.
/// It is also responsible for handling events and drawing the UI.
impl Root {
    pub fn new(event_sender: Sender<Event>, logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        //let authentication_data = Arc::new(RwLock::new(None));
        //let authentication =
        //    Authentication::new(event_sender.clone(), Arc::clone(&authentication_data));
        //let home = Home::new(event_sender.clone(), Arc::clone(&authentication_data));

        let home = Home::new(event_sender.clone());
        // show logs if we set TOOT_RS_SHOW_LOGS to anything
        let show_logs = std::env::var("TOOT_RS_SHOW_LOGS").is_ok();
        Self {
            _event_sender: event_sender,
            state: State::Home,
            // authentication,
            home,
            logs,
            show_logs,
        }
    }

    #[instrument(name = "root", skip_all)]
    pub async fn start(&mut self) -> miette::Result<()> {
        Ok(())
        //self.authentication
        //    .start()
        //    .await
        //    .context("Authentication component failed to start")
    }

    /// Handles an event.
    /// Returns an `Outcome` that indicates whether the event was handled or not.
    #[instrument(name = "root::handle_event", skip_all)]
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        match self.state {
            //State::Authentication => {
            //    if event == &Event::AuthenticationSuccess {
            //        self.state = State::Home;
            //        self.home.start().await.ok();
            //        return Outcome::Handled;
            //    }
            //    self.authentication.handle_event(event).await
            //}
            State::Home => self.home.handle_event(event).await,
        }
    }

    #[instrument(name = "root::draw", skip_all)]
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        if let [top, mid, logs, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TitleBar::HEIGHT),
                Constraint::Min(0),
                Constraint::Length(if self.show_logs { 7 } else { 0 }), // logs
                Constraint::Length(StatusBar::HEIGHT),
            ])
            .split(area)
        {
            match self.state {
                State::Home => {
                    f.render_widget(TitleBar::new(self.home.title()), top);
                    f.render_widget(StatusBar::new(self.home.status()), bottom);
                    self.home.draw(f, mid);
                }
            }
            f.render_widget(LogWidget::new(self.logs.clone()), logs);
        }
    }
}
