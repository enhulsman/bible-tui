use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::action::Action;
use crate::bible::canon::CANON;

#[derive(Debug, PartialEq)]
enum CompletionContext {
    Command,
    Translation,
    Goto,
    Other,
}

const COMMANDS: &[&str] = &["quit", "goto", "translation"];

pub struct CommandPalette {
    input: String,
    cursor: usize,
    suggestions: Vec<String>,
    suggestion_state: ListState,
    selected: usize,
    available_translations: Vec<String>,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            suggestions: Vec::new(),
            suggestion_state: ListState::default(),
            selected: 0,
            available_translations: Vec::new(),
        }
    }

    pub fn set_translations(&mut self, names: Vec<String>) {
        self.available_translations = names;
    }

    pub fn open(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.suggestions.clear();
        self.selected = 0;
        self.suggestion_state.select(None);
        self.update_suggestions();
    }

    fn detect_context(&self) -> CompletionContext {
        let trimmed = self.input.trim_start();
        if !trimmed.contains(' ') {
            return CompletionContext::Command;
        }
        let cmd = trimmed.split_whitespace().next().unwrap_or("");
        match cmd {
            "t" | "translation" => CompletionContext::Translation,
            "goto" | "g" => CompletionContext::Goto,
            _ => CompletionContext::Other,
        }
    }

    fn argument_prefix(&self) -> &str {
        let trimmed = self.input.trim_start();
        if let Some(pos) = trimmed.find(' ') {
            trimmed[pos..].trim_start()
        } else {
            trimmed
        }
    }

    fn update_suggestions(&mut self) {
        let ctx = self.detect_context();
        self.suggestions = match ctx {
            CompletionContext::Command => {
                let prefix = self.input.trim_start().to_lowercase();
                COMMANDS
                    .iter()
                    .filter(|c| c.starts_with(&prefix))
                    .map(|c| c.to_string())
                    .collect()
            }
            CompletionContext::Translation => {
                let prefix = self.argument_prefix().to_lowercase();
                self.available_translations
                    .iter()
                    .filter(|t| t.starts_with(&prefix))
                    .cloned()
                    .collect()
            }
            CompletionContext::Goto => {
                let prefix = self.argument_prefix().to_lowercase();
                if prefix.is_empty() {
                    CANON.iter().map(|b| b.name.to_string()).collect()
                } else {
                    CANON
                        .iter()
                        .filter(|b| b.name.to_lowercase().starts_with(&prefix))
                        .map(|b| b.name.to_string())
                        .collect()
                }
            }
            CompletionContext::Other => Vec::new(),
        };
        self.suggestions.truncate(15);
        self.selected = 0;
        if self.suggestions.is_empty() {
            self.suggestion_state.select(None);
        } else {
            self.suggestion_state.select(Some(0));
        }
    }

    fn accept_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let suggestion = self.suggestions[self.selected].clone();
        let ctx = self.detect_context();

        match ctx {
            CompletionContext::Command => {
                self.input = format!("{suggestion} ");
                self.cursor = self.input.len();
            }
            CompletionContext::Translation | CompletionContext::Goto => {
                // Replace argument portion
                let trimmed = self.input.trim_start();
                if let Some(pos) = trimmed.find(' ') {
                    let cmd_part = &self.input[..self.input.len() - trimmed.len() + pos + 1];
                    if ctx == CompletionContext::Goto {
                        self.input = format!("{cmd_part}{suggestion} ");
                    } else {
                        self.input = format!("{cmd_part}{suggestion}");
                    }
                } else {
                    self.input = suggestion;
                }
                self.cursor = self.input.len();
            }
            CompletionContext::Other => {}
        }
        self.update_suggestions();
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => Action::ExitCommandMode,
            KeyCode::Enter => {
                let cmd = self.input.trim().to_string();
                Action::ExecuteCommand(cmd)
            }
            KeyCode::Tab => {
                self.accept_suggestion();
                Action::None
            }
            KeyCode::Down => {
                if !self.suggestions.is_empty() {
                    self.selected = (self.selected + 1).min(self.suggestions.len() - 1);
                    self.suggestion_state.select(Some(self.selected));
                }
                Action::None
            }
            KeyCode::Up => {
                if !self.suggestions.is_empty() {
                    self.selected = self.selected.saturating_sub(1);
                    self.suggestion_state.select(Some(self.selected));
                }
                Action::None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.update_suggestions();
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
                    self.update_suggestions();
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let width = area.width.min(60);
        let x = (area.width.saturating_sub(width)) / 2;

        let suggestion_height = if self.suggestions.is_empty() {
            0
        } else {
            (self.suggestions.len() as u16).min(area.height.saturating_sub(6))
        };
        let total_height = 3 + suggestion_height;
        let popup = Rect::new(x, 2, width, total_height);

        frame.render_widget(Clear, popup);

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(suggestion_height),
        ])
        .split(popup);

        // Input line
        let input_line = Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow)),
            Span::raw(&self.input),
            Span::styled("\u{2588}", Style::default().fg(Color::Gray)),
        ]);

        let input_widget = Paragraph::new(input_line).block(
            Block::default()
                .title(" Command ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(input_widget, chunks[0]);

        // Suggestion list
        if !self.suggestions.is_empty() {
            let items: Vec<ListItem> = self
                .suggestions
                .iter()
                .map(|s| ListItem::new(Line::from(Span::raw(s.as_str()))))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_stateful_widget(list, chunks[1], &mut self.suggestion_state);
        }
    }
}
