use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use miette::{IntoDiagnostic, Result};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io::{self, Stdout};
use tracing::{error, info};

type Backend = CrosstermBackend<Stdout>;

pub struct UI {
    terminal: Terminal<Backend>,
}

impl UI {
    pub fn new() -> Result<Self> {
        let terminal =
            Terminal::new(Backend::new(io::stdout())).expect("Failed to create terminal");
        Ok(Self { terminal })
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting UI");
        terminal::enable_raw_mode().into_diagnostic();
        crossterm::execute!(io::stdout(), EnterAlternateScreen).into_diagnostic();
        self.terminal.hide_cursor().into_diagnostic();
        self.terminal.clear().into_diagnostic();
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f).into_diagnostic();
        Ok(())
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        // uncomment the following line in case there is a panic that prevents the UI from being
        // properly shut down
        // TODO find a way to do this without sleeping
        //std::thread::sleep(std::time::Duration::from_millis(3000));
        if let Err(e) = terminal::disable_raw_mode() {
            error!("Error disabling raw mode: {}", e);
        }
        if let Err(e) = crossterm::execute!(io::stdout(), LeaveAlternateScreen) {
            error!("Error leaving alternate screen: {}", e);
        }
        if let Err(e) = self.terminal.show_cursor() {
            error!("Error showing cursor: {}", e);
        }
    }
}
