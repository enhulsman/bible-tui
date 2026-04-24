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
        Style::default().bg(Color::Rgb(40, 38, 35)).fg(Color::Rgb(168, 159, 145))
    }

    pub fn status_key() -> Style {
        Style::default().bg(Color::Rgb(40, 38, 35)).fg(Color::Rgb(210, 170, 90))
    }

    pub fn status_hint() -> Style {
        Style::default().bg(Color::Rgb(40, 38, 35)).fg(Color::Rgb(135, 128, 118))
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
