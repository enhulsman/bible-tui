use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::action::Action;
use crate::bible::canon::CANON;
use crate::ui::theme::Theme;

#[derive(Debug, PartialEq, Eq)]
enum NavFocus {
    Books,
    Chapters,
}

pub struct NavPanel {
    book_state: ListState,
    selected_book: usize,
    selected_chapter: usize,
    chapter_count: u16,
    focus: NavFocus,
}

impl NavPanel {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            book_state: state,
            selected_book: 0,
            selected_chapter: 0,
            chapter_count: CANON[0].chapter_count,
            focus: NavFocus::Books,
        }
    }

    pub fn sync_to(&mut self, book_index: u8, chapter: u16) {
        self.selected_book = book_index as usize;
        self.book_state.select(Some(self.selected_book));
        self.selected_chapter = (chapter as usize).saturating_sub(1);
        self.chapter_count = CANON[self.selected_book].chapter_count;
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc | KeyCode::Tab => Action::ToggleNavPanel,
            KeyCode::Char('j') | KeyCode::Down => {
                match self.focus {
                    NavFocus::Books => self.move_book(1),
                    NavFocus::Chapters => self.move_chapter(1),
                }
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.focus {
                    NavFocus::Books => self.move_book(-1),
                    NavFocus::Chapters => self.move_chapter(-1),
                }
                Action::None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.focus == NavFocus::Books {
                    self.focus = NavFocus::Chapters;
                    self.selected_chapter = 0;
                    self.chapter_count = CANON[self.selected_book].chapter_count;
                }
                Action::None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if self.focus == NavFocus::Chapters {
                    self.focus = NavFocus::Books;
                }
                Action::None
            }
            KeyCode::Enter => {
                let chapter = if self.focus == NavFocus::Chapters {
                    (self.selected_chapter + 1) as u16
                } else {
                    1
                };
                Action::NavigateToRef {
                    book: self.selected_book as u8,
                    chapter,
                }
            }
            _ => Action::None,
        }
    }

    fn move_book(&mut self, delta: i32) {
        let new = self.selected_book as i32 + delta;
        if new >= 0 && (new as usize) < 66 {
            self.selected_book = new as usize;
            self.book_state.select(Some(self.selected_book));
            self.chapter_count = CANON[self.selected_book].chapter_count;
            self.selected_chapter = 0;
        }
    }

    fn move_chapter(&mut self, delta: i32) {
        let new = self.selected_chapter as i32 + delta;
        if new >= 0 && (new as u16) < self.chapter_count {
            self.selected_chapter = new as usize;
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(area);

        self.render_books(frame, chunks[0]);
        self.render_chapters(frame, chunks[1]);
    }

    fn render_books(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = CANON
            .iter()
            .enumerate()
            .map(|(i, book)| {
                let style = if i == self.selected_book {
                    Theme::nav_selected()
                } else {
                    Theme::nav_normal()
                };
                let prefix = if i == self.selected_book { "▶ " } else { "  " };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, book.name),
                    style,
                )))
            })
            .collect();

        let border_style = if self.focus == NavFocus::Books {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Theme::nav_border()
        };

        let list = List::new(items).block(
            Block::default()
                .title(" Books ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_stateful_widget(list, area, &mut self.book_state);
    }

    fn render_chapters(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focus == NavFocus::Chapters {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Theme::nav_border()
        };

        let block = Block::default()
            .title(" Chapters ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render chapters in a grid layout
        let cols = (inner.width / 4).max(1) as usize;
        let mut lines: Vec<Line> = Vec::new();
        let mut row_spans: Vec<Span> = Vec::new();

        for ch in 0..self.chapter_count as usize {
            let style = if ch == self.selected_chapter {
                Theme::nav_selected()
            } else {
                Theme::nav_normal()
            };
            let label = format!("{:>3} ", ch + 1);
            row_spans.push(Span::styled(label, style));

            if row_spans.len() >= cols {
                lines.push(Line::from(std::mem::take(&mut row_spans)));
            }
        }
        if !row_spans.is_empty() {
            lines.push(Line::from(row_spans));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}
