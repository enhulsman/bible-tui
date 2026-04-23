use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::action::Action;
use crate::bible::canon::CANON;
use crate::search::index::{SearchIndex, SearchResult};

pub struct SearchBar {
    query: String,
    cursor: usize,
    results: Vec<SearchResult>,
    result_state: ListState,
    selected: usize,
}

impl SearchBar {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor: 0,
            results: Vec::new(),
            result_state: ListState::default(),
            selected: 0,
        }
    }

    pub fn open(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.results.clear();
        self.selected = 0;
        self.result_state.select(None);
    }

    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        index: &SearchIndex,
    ) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => Action::ExitSearchMode,
            KeyCode::Enter => {
                if let Some(result) = self.results.get(self.selected) {
                    Action::NavigateToRef {
                        book: result.verse_ref.book_index,
                        chapter: result.verse_ref.chapter,
                    }
                } else {
                    Action::ExitSearchMode
                }
            }
            KeyCode::Char(c) => {
                self.query.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.do_search(index);
                Action::None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let prev = self.query[..self.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.query.drain(prev..self.cursor);
                    self.cursor = prev;
                    self.do_search(index);
                }
                Action::None
            }
            KeyCode::Down => {
                if !self.results.is_empty() {
                    self.selected = (self.selected + 1).min(self.results.len() - 1);
                    self.result_state.select(Some(self.selected));
                }
                Action::None
            }
            KeyCode::Up => {
                if !self.results.is_empty() {
                    self.selected = self.selected.saturating_sub(1);
                    self.result_state.select(Some(self.selected));
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn do_search(&mut self, index: &SearchIndex) {
        if self.query.len() >= 2 {
            self.results = index.search(&self.query);
            // Limit display to first 50
            self.results.truncate(50);
            self.selected = 0;
            if !self.results.is_empty() {
                self.result_state.select(Some(0));
            } else {
                self.result_state.select(None);
            }
        } else {
            self.results.clear();
            self.result_state.select(None);
        }
    }

    #[allow(dead_code)]
    pub fn current_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected)
    }

    #[allow(dead_code)]
    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }

    #[allow(dead_code)]
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Navigate to next search result (n in normal mode)
    pub fn next_result(&mut self) -> Option<Action> {
        if self.results.is_empty() {
            return None;
        }
        self.selected = (self.selected + 1) % self.results.len();
        self.result_state.select(Some(self.selected));
        self.results.get(self.selected).map(|r| Action::NavigateToRef {
            book: r.verse_ref.book_index,
            chapter: r.verse_ref.chapter,
        })
    }

    /// Navigate to previous search result (N in normal mode)
    pub fn prev_result(&mut self) -> Option<Action> {
        if self.results.is_empty() {
            return None;
        }
        self.selected = if self.selected == 0 {
            self.results.len() - 1
        } else {
            self.selected - 1
        };
        self.result_state.select(Some(self.selected));
        self.results.get(self.selected).map(|r| Action::NavigateToRef {
            book: r.verse_ref.book_index,
            chapter: r.verse_ref.chapter,
        })
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Search overlay: input bar at top + results below
        let popup_height = (self.results.len() as u16 + 3).min(area.height.saturating_sub(4));
        let popup_width = area.width.min(70);
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = 2;

        let popup_area = Rect::new(x, y, popup_width, popup_height.max(3));

        frame.render_widget(Clear, popup_area);

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(popup_area);

        // Search input
        let input_line = Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(&self.query),
            Span::styled("█", Style::default().fg(Color::Gray)),
        ]);

        let input = Paragraph::new(input_line).block(
            Block::default()
                .title(" Search ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(input, chunks[0]);

        // Results list
        if !self.results.is_empty() {
            let items: Vec<ListItem> = self
                .results
                .iter()
                .map(|r| {
                    let book_name = if (r.verse_ref.book_index as usize) < CANON.len() {
                        CANON[r.verse_ref.book_index as usize].name
                    } else {
                        "?"
                    };
                    let ref_str = format!(
                        "{} {}:{}",
                        book_name, r.verse_ref.chapter, r.verse_ref.verse
                    );
                    let truncated: String = r.text.chars().take(40).collect();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<25}", ref_str),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(truncated, Style::default().fg(Color::White)),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_stateful_widget(list, chunks[1], &mut self.result_state);
        }
    }
}
