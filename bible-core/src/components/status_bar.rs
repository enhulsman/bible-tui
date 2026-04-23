use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::theme::Theme;

pub struct StatusBar;

impl StatusBar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        book_name: &str,
        chapter: u16,
        verse: u8,
        translation: &str,
        error: Option<&str>,
    ) {
        let position = format!(" {} {}:{}", book_name, chapter, verse);

        let spans = if let Some(err) = error {
            vec![
                Span::styled(position, Theme::status_bar()),
                Span::styled(" | ", Theme::status_bar()),
                Span::styled(err.to_string(), Style::default().fg(Color::Red)),
            ]
        } else {
            vec![
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
                Span::styled("?", Theme::status_key()),
                Span::styled(" help  ", Theme::status_hint()),
                Span::styled("q", Theme::status_key()),
                Span::styled(" quit", Theme::status_hint()),
            ]
        };

        let bar = Paragraph::new(Line::from(spans)).style(Theme::status_bar());
        frame.render_widget(bar, area);
    }
}
