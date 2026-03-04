use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::bible::canon::CANON;
use crate::ui::theme::Theme;

pub struct StatusBar;

impl StatusBar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        book_index: u8,
        chapter: u16,
        verse: u8,
        translation: &str,
    ) {
        let book_name = if (book_index as usize) < CANON.len() {
            CANON[book_index as usize].name
        } else {
            "?"
        };

        let position = format!(" {} {}:{}", book_name, chapter, verse);

        let bar = Paragraph::new(Line::from(vec![
            Span::styled(position, Theme::status_bar()),
            Span::styled(" | ", Theme::status_bar()),
            Span::styled(translation, Theme::status_key()),
            Span::styled(" | ", Theme::status_bar()),
            Span::styled("j/k", Theme::status_key()),
            Span::styled(" scroll  ", Theme::status_hint()),
            Span::styled("/", Theme::status_key()),
            Span::styled("search  ", Theme::status_hint()),
            Span::styled("Tab", Theme::status_key()),
            Span::styled(" nav  ", Theme::status_hint()),
            Span::styled("B", Theme::status_key()),
            Span::styled(" bookmark  ", Theme::status_hint()),
            Span::styled("q", Theme::status_key()),
            Span::styled(" quit", Theme::status_hint()),
        ]))
        .style(Theme::status_bar());

        frame.render_widget(bar, area);
    }
}
