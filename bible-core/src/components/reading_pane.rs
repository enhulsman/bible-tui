use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use unicode_width::UnicodeWidthStr;

use crate::bible::model::{Chapter, VerseSpan};
use crate::ui::theme::Theme;
use crate::ui::wrap::{superscript_number, wrap_spans};

pub struct ReadingPane {
    /// Pre-wrapped lines ready for rendering.
    lines: Vec<Line<'static>>,
    /// Maps each display line index to its verse number.
    line_to_verse: Vec<u8>,
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
    /// Translation-specific book name for the title.
    book_name: String,
}

impl ReadingPane {
    pub fn new() -> Self {
        Self {
            lines: vec![Line::from("Loading...")],
            line_to_verse: vec![1],
            scroll: 0,
            total_lines: 1,
            visible_height: 0,
            book_index: 0,
            chapter_num: 1,
            book_name: String::new(),
        }
    }

    pub fn set_chapter(&mut self, book_index: u8, book_name: &str, chapter: &Chapter, width: u16) {
        self.book_index = book_index;
        self.book_name = book_name.to_string();
        self.chapter_num = chapter.number;
        self.scroll = 0;
        self.rebuild_lines(chapter, width);
    }

    pub fn rebuild_lines(&mut self, chapter: &Chapter, width: u16) {
        if width < 4 {
            self.lines = vec![Line::from("")];
            self.line_to_verse = vec![1];
            self.total_lines = 1;
            debug_assert_eq!(self.lines.len(), self.line_to_verse.len());
            return;
        }

        let content_width = width.saturating_sub(2); // 1 char padding each side
        let mut all_lines: Vec<Line<'static>> = Vec::new();
        let mut line_verses: Vec<u8> = Vec::new();
        let first_verse = chapter.verses.first().map(|v| v.number).unwrap_or(1);

        // Chapter title
        let title = format!("{} {}", self.book_name, self.chapter_num);
        all_lines.push(Line::from(Span::styled(title, Theme::chapter_title())));
        line_verses.push(first_verse);
        all_lines.push(Line::from(""));
        line_verses.push(first_verse);

        // Compute uniform indent width from max verse number.
        // Superscript digits are each 1 display column (UAX #11), so
        // format!("{:<width$}", s) pads correctly with spaces.
        let max_verse = chapter.verses.last().map(|v| v.number).unwrap_or(1);
        let indent_width = superscript_number(max_verse).width() + 1; // +1 for space after number
        let use_indent = content_width > indent_width as u16 + 10;

        for verse in &chapter.verses {
            // Check for section headings before this verse
            for heading in &chapter.headings {
                if heading.before_verse == verse.number {
                    let hv = heading.before_verse.max(1);
                    all_lines.push(Line::from(""));
                    line_verses.push(hv);
                    all_lines.push(Line::from(Span::styled(
                        heading.text.clone(),
                        Theme::section_heading(),
                    )));
                    line_verses.push(hv);
                    all_lines.push(Line::from(""));
                    line_verses.push(hv);
                }
            }

            // Build text spans (without verse number)
            let mut text_spans: Vec<(String, Style)> = Vec::new();
            for span in &verse.spans {
                match span {
                    VerseSpan::Plain(text) => {
                        text_spans.push((text.clone(), Theme::verse_text()));
                    }
                    VerseSpan::RedLetter(text) => {
                        text_spans.push((text.clone(), Theme::red_letter()));
                    }
                    VerseSpan::Selah => {
                        text_spans.push(("Selah".to_string(), Theme::section_heading()));
                    }
                }
            }
            text_spans.push((" ".to_string(), Style::default()));

            if use_indent {
                // Right-pad verse number to uniform indent width
                let num_str = format!("{:<width$}", superscript_number(verse.number), width = indent_width);
                let text_width = content_width.saturating_sub(indent_width as u16);
                let wrapped = wrap_spans(&text_spans, text_width);

                for (i, line) in wrapped.into_iter().enumerate() {
                    let prefix = if i == 0 {
                        Span::styled(num_str.clone(), Theme::verse_number())
                    } else {
                        Span::raw(" ".repeat(indent_width))
                    };
                    let mut spans = vec![prefix];
                    spans.extend(line.spans);
                    all_lines.push(Line::from(spans));
                    line_verses.push(verse.number);
                }
            } else {
                // Narrow terminal fallback: no hanging indent
                let mut spans: Vec<(String, Style)> = Vec::new();
                let num_str = format!("{} ", superscript_number(verse.number));
                spans.push((num_str, Theme::verse_number()));
                spans.extend(text_spans);
                let wrapped = wrap_spans(&spans, content_width);
                let count = wrapped.len();
                all_lines.extend(wrapped);
                for _ in 0..count {
                    line_verses.push(verse.number);
                }
            }
        }

        debug_assert_eq!(all_lines.len(), line_verses.len());
        self.total_lines = all_lines.len() as u16;
        self.line_to_verse = line_verses;
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
        if self.line_to_verse.is_empty() {
            return 1;
        }
        let idx = (self.scroll as usize).min(self.line_to_verse.len() - 1);
        self.line_to_verse[idx]
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

    pub fn book_name(&self) -> &str {
        &self.book_name
    }

    pub fn book_index(&self) -> u8 {
        self.book_index
    }

    pub fn chapter_num(&self) -> u16 {
        self.chapter_num
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bible::model::{Chapter, SectionHeading, Verse, VerseSpan};

    fn make_verse(number: u8, text: &str) -> Verse {
        Verse {
            number,
            spans: vec![VerseSpan::Plain(text.to_string())],
            paragraph_break: false,
        }
    }

    fn make_chapter(number: u16, verses: Vec<Verse>, headings: Vec<SectionHeading>) -> Chapter {
        Chapter { number, verses, headings }
    }

    #[test]
    fn verse_tracking_three_verses_no_headings() {
        let chapter = make_chapter(1, vec![
            make_verse(1, "First verse text."),
            make_verse(2, "Second verse text."),
            make_verse(3, "Third verse text."),
        ], vec![]);

        let mut pane = ReadingPane::new();
        pane.book_name = "Test".to_string();
        pane.set_chapter(0, "Test", &chapter, 80);

        assert_eq!(pane.current_verse_approx(), 1);

        // Scroll past title (2 lines) to verse 1
        pane.scroll = 2;
        assert_eq!(pane.current_verse_approx(), 1);

        // Scroll to last line — should be verse 3
        pane.scroll = pane.total_lines.saturating_sub(1);
        assert_eq!(pane.current_verse_approx(), 3);
    }

    #[test]
    fn verse_tracking_with_heading() {
        let chapter = make_chapter(1, vec![
            make_verse(1, "First verse."),
            make_verse(2, "Second verse."),
            make_verse(3, "Third verse."),
        ], vec![
            SectionHeading { text: "A Heading".to_string(), before_verse: 2 },
        ]);

        let mut pane = ReadingPane::new();
        pane.set_chapter(0, "Test", &chapter, 80);

        // Heading lines should map to verse 2
        // Find the heading: after verse 1 lines, there are 3 heading lines (blank, heading, blank)
        // Title(v1) + blank(v1) + verse1(v1) + blank(v2) + heading(v2) + blank(v2) + verse2(v2) + verse3(v3)
        pane.scroll = 3; // blank line before heading
        assert_eq!(pane.current_verse_approx(), 2);

        pane.scroll = 4; // heading text
        assert_eq!(pane.current_verse_approx(), 2);
    }

    #[test]
    fn verse_tracking_empty_chapter() {
        let chapter = make_chapter(1, vec![], vec![]);

        let mut pane = ReadingPane::new();
        pane.set_chapter(0, "Test", &chapter, 80);

        assert_eq!(pane.current_verse_approx(), 1);
    }

    #[test]
    fn verse_tracking_heading_before_verse_1() {
        let chapter = make_chapter(1, vec![
            make_verse(1, "First verse."),
        ], vec![
            SectionHeading { text: "Intro".to_string(), before_verse: 1 },
        ]);

        let mut pane = ReadingPane::new();
        pane.set_chapter(0, "Test", &chapter, 80);

        // Title + blank + heading lines + verse 1
        pane.scroll = 0;
        assert_eq!(pane.current_verse_approx(), 1);

        pane.scroll = pane.total_lines.saturating_sub(1);
        assert_eq!(pane.current_verse_approx(), 1);
    }

    #[test]
    fn verse_tracking_narrow_width() {
        let chapter = make_chapter(1, vec![
            make_verse(1, "Some text."),
            make_verse(2, "More text."),
        ], vec![]);

        let mut pane = ReadingPane::new();
        pane.set_chapter(0, "T", &chapter, 15);

        assert_eq!(pane.current_verse_approx(), 1);

        pane.scroll = pane.total_lines.saturating_sub(1);
        assert_eq!(pane.current_verse_approx(), 2);
    }

    #[test]
    fn verse_tracking_very_narrow_width() {
        let chapter = make_chapter(1, vec![
            make_verse(1, "Text."),
        ], vec![]);

        let mut pane = ReadingPane::new();
        pane.set_chapter(0, "T", &chapter, 3);

        assert_eq!(pane.current_verse_approx(), 1);
    }
}
