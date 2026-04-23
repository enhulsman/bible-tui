use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::bible::canon::CANON;
use crate::bible::model::{Chapter, VerseSpan};
use crate::ui::theme::Theme;
use crate::ui::wrap::{superscript_number, wrap_spans};

pub struct ReadingPane {
    /// Pre-wrapped lines ready for rendering.
    lines: Vec<Line<'static>>,
    /// Current scroll offset (in wrapped lines).
    scroll: u16,
    /// Total number of wrapped lines.
    total_lines: u16,
    /// Visible height of the pane.
    visible_height: u16,
    /// Current book index for the title.
    book_index: u8,
    /// Current chapter number for the title.
    chapter_num: u16,
}

impl ReadingPane {
    pub fn new() -> Self {
        Self {
            lines: vec![Line::from("Loading...")],
            scroll: 0,
            total_lines: 1,
            visible_height: 0,
            book_index: 0,
            chapter_num: 1,
        }
    }

    pub fn set_chapter(&mut self, book_index: u8, chapter: &Chapter, width: u16) {
        self.book_index = book_index;
        self.chapter_num = chapter.number;
        self.scroll = 0;
        self.rebuild_lines(chapter, width);
    }

    pub fn rebuild_lines(&mut self, chapter: &Chapter, width: u16) {
        if width < 4 {
            self.lines = vec![Line::from("")];
            self.total_lines = 1;
            return;
        }

        let content_width = width.saturating_sub(2); // 1 char padding each side
        let mut all_lines: Vec<Line<'static>> = Vec::new();

        // Chapter title
        let book_name = if (self.book_index as usize) < CANON.len() {
            CANON[self.book_index as usize].name
        } else {
            "Unknown"
        };
        let title = format!("{} {}", book_name, self.chapter_num);
        all_lines.push(Line::from(Span::styled(title, Theme::chapter_title())));
        all_lines.push(Line::from(""));

        for verse in &chapter.verses {
            // Check for section headings before this verse
            for heading in &chapter.headings {
                if heading.before_verse == verse.number {
                    all_lines.push(Line::from(""));
                    all_lines.push(Line::from(Span::styled(
                        heading.text.clone(),
                        Theme::section_heading(),
                    )));
                    all_lines.push(Line::from(""));
                }
            }

            // Build spans for this verse: superscript number + text
            let mut spans: Vec<(String, Style)> = Vec::new();

            let num_str = format!("{} ", superscript_number(verse.number));
            spans.push((num_str, Theme::verse_number()));

            for span in &verse.spans {
                match span {
                    VerseSpan::Plain(text) => {
                        spans.push((text.clone(), Theme::verse_text()));
                    }
                    VerseSpan::RedLetter(text) => {
                        spans.push((text.clone(), Theme::red_letter()));
                    }
                    VerseSpan::Selah => {
                        spans.push(("Selah".to_string(), Theme::section_heading()));
                    }
                }
            }
            spans.push((" ".to_string(), Style::default()));

            let wrapped = wrap_spans(&spans, content_width);
            all_lines.extend(wrapped);
        }

        self.total_lines = all_lines.len() as u16;
        self.lines = all_lines;
    }

    pub fn scroll_down(&mut self, n: u16) {
        let max_scroll = self.total_lines.saturating_sub(self.visible_height);
        self.scroll = (self.scroll + n).min(max_scroll);
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.total_lines.saturating_sub(self.visible_height);
    }

    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_height.saturating_sub(2));
    }

    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_height.saturating_sub(2));
    }

    pub fn current_verse_approx(&self) -> u8 {
        // Rough estimate based on scroll position
        if self.total_lines == 0 {
            return 1;
        }
        let ratio = self.scroll as f32 / self.total_lines.max(1) as f32;
        // This is a rough approximation - could be improved with line-to-verse mapping
        1u8.max((ratio * 30.0) as u8) // assume ~30 verses average
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.visible_height = area.height.saturating_sub(2); // account for borders

        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default());

        // Slice the lines based on scroll
        let start = self.scroll as usize;
        let end = (start + self.visible_height as usize).min(self.lines.len());
        let visible_lines: Vec<Line> = if start < self.lines.len() {
            self.lines[start..end].to_vec()
        } else {
            vec![]
        };

        let text = Text::from(visible_lines);
        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, area);
    }

    pub fn book_index(&self) -> u8 {
        self.book_index
    }

    pub fn chapter_num(&self) -> u16 {
        self.chapter_num
    }
}
