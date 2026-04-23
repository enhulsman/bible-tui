use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::action::Action;

pub struct CommandPalette {
    input: String,
    cursor: usize,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
        }
    }

    pub fn open(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => Action::ExitCommandMode,
            KeyCode::Enter => {
                let cmd = self.input.trim().to_string();
                Action::ExecuteCommand(cmd)
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                Action::None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let prev = self.input[..self.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input.drain(prev..self.cursor);
                    self.cursor = prev;
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.min(60);
        let x = (area.width.saturating_sub(width)) / 2;
        let popup = Rect::new(x, 2, width, 3);

        frame.render_widget(Clear, popup);

        let input_line = Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow)),
            Span::raw(&self.input),
            Span::styled("█", Style::default().fg(Color::Gray)),
        ]);

        let widget = Paragraph::new(input_line).block(
            Block::default()
                .title(" Command ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        frame.render_widget(widget, popup);
    }
}
