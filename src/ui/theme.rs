use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn verse_number() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn verse_text() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn red_letter() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn section_heading() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn chapter_title() -> Style {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_bar() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }

    pub fn status_key() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn status_hint() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn nav_selected() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn nav_normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn nav_border() -> Style {
        Style::default().fg(Color::DarkGray)
    }
}
