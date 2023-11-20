use anyhow::Result;

use ratatui::{prelude::*, widgets::*};

pub struct HelpItem {
    pub key: char,
    pub help: String,
}

pub fn draw_help_item(help: &HelpItem) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("{}", &help.key),
        Style::default(),
    )))
}

impl HelpItem {
    //fn draw(k: char, help: String) -> ListItem {
    //    ListItem::new(Line::from(Span::styled(format!("{}", self.key))))
    //}
    pub fn new(key: char, help: String) -> HelpItem {
        HelpItem { key, help }
    }
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = Line::from(vec![
            Span::styled(format!("{}", self.key), Style::new().red()),
            Span::raw(format!("to {}", self.help)),
        ]);
        Paragraph::new(text);
    }
}

pub fn help_line(area: Rect, buf: &mut Buffer) {
    let layout = Layout::default().direction(Direction::Horizontal);
    //let text = vec![Line::from(vec![Span::raw("Hit")])];
    //let helpItems = vec![HelpItem::new('q', "quit".to_string())];
    //for item in helpItems {
    //    item.render(area, buf);
    //}
}
