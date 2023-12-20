use core::fmt;
use parking_lot::Mutex;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::Line,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::sync::Arc;
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Default)]
pub struct LogCollector {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

pub struct LogMessage {
    pub level: String,
    pub target: String,
    pub message: String,
}

impl LogCollector {
    #[must_use]
    pub fn logs(&self) -> Arc<Mutex<Vec<LogMessage>>> {
        self.logs.clone()
    }
}

impl<S> Layer<S> for LogCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut logs = self.logs.lock();
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target();
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message;
        let log = LogMessage {
            level,
            target: target.to_string(),
            message,
        };
        logs.push(log);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

pub struct LogWidget {
    logs: Arc<Mutex<Vec<LogMessage>>>,
}

impl LogWidget {
    pub fn new(logs: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        Self { logs }
    }
}

impl Widget for LogWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logs = self.logs.lock();
        let max_lines = area.height as usize;
        let start_index = if logs.len() > max_lines {
            logs.len() - max_lines
        } else {
            0
        };

        let text = logs[start_index..]
            .iter()
            .map(|log| {
                let level_color = match log.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Blue,
                    "TRACE" => Color::Cyan,
                    _ => Color::White,
                };
                Line::from(vec![
                    Span::styled(&log.level, Style::default().fg(level_color)),
                    Span::raw(" "),
                    Span::styled(&log.target, Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(": ", Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(&log.message, Style::default()),
                ])
            })
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}
