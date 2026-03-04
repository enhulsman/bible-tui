use ratatui::style::Style;
use ratatui::text::{Line, Span};
use textwrap::{wrap, Options, WordSplitter};
use unicode_width::UnicodeWidthStr;

/// A styled fragment for pre-wrapping. We break styled text into wrapped lines
/// so that each Line maps to exactly one screen row.
#[derive(Clone)]
#[allow(dead_code)]
struct StyledFragment {
    text: String,
    style: Style,
}

/// Pre-wrap a sequence of styled spans into Lines that fit exactly within `width`.
/// This is the ratatui #2342 workaround: scroll offset == line index, no ambiguity.
pub fn wrap_spans(spans: &[(String, Style)], width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![];
    }

    // Build one big string and track style ranges
    let mut full_text = String::new();
    let mut ranges: Vec<(usize, usize, Style)> = Vec::new();

    for (text, style) in spans {
        let start = full_text.len();
        full_text.push_str(text);
        let end = full_text.len();
        if start < end {
            ranges.push((start, end, *style));
        }
    }

    if full_text.is_empty() {
        return vec![Line::from("")];
    }

    let opts = Options::new(width as usize).word_splitter(WordSplitter::NoHyphenation);
    let wrapped_lines = wrap(&full_text, opts);

    let mut result = Vec::new();
    let mut byte_offset = 0;

    for wrapped in &wrapped_lines {
        let line_str = wrapped.as_ref();
        // textwrap may strip leading whitespace on continuation lines
        // We need to find where this line starts in the original text
        let line_start = byte_offset;
        let line_end = line_start + line_str.len();

        // Skip whitespace between wrapped lines
        byte_offset = line_end;
        // Skip the whitespace/newline that was the wrap point
        if byte_offset < full_text.len() {
            let ch = full_text.as_bytes()[byte_offset];
            if ch == b' ' || ch == b'\n' {
                byte_offset += 1;
            }
        }

        // Build styled spans for this line
        let mut line_spans: Vec<Span<'static>> = Vec::new();
        let mut pos = line_start;

        for &(range_start, range_end, style) in &ranges {
            if range_end <= pos || range_start >= line_end {
                continue;
            }
            let seg_start = pos.max(range_start);
            let seg_end = line_end.min(range_end);
            if seg_start < seg_end {
                // Fill any gap before this range with default style
                if seg_start > pos {
                    let gap = &full_text[pos..seg_start];
                    if !gap.is_empty() {
                        line_spans.push(Span::styled(gap.to_string(), Style::default()));
                    }
                }
                let segment = &full_text[seg_start..seg_end];
                line_spans.push(Span::styled(segment.to_string(), style));
                pos = seg_end;
            }
        }

        // Fill remaining with default style
        if pos < line_end {
            let remaining = &full_text[pos..line_end];
            if !remaining.is_empty() {
                line_spans.push(Span::styled(remaining.to_string(), Style::default()));
            }
        }

        if line_spans.is_empty() {
            result.push(Line::from(""));
        } else {
            result.push(Line::from(line_spans));
        }
    }

    if result.is_empty() {
        result.push(Line::from(""));
    }

    result
}

/// Superscript digits for verse numbers.
const SUPERSCRIPT: &[char] = &['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

pub fn superscript_number(n: u8) -> String {
    n.to_string()
        .chars()
        .map(|c| SUPERSCRIPT[c.to_digit(10).unwrap() as usize])
        .collect()
}

#[allow(dead_code)]
pub fn superscript_width(n: u8) -> usize {
    superscript_number(n).width()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_wrap() {
        let spans = vec![("Hello world, this is a test".to_string(), Style::default())];
        let lines = wrap_spans(&spans, 15);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn empty_input() {
        let spans: Vec<(String, Style)> = vec![];
        let lines = wrap_spans(&spans, 80);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn superscript_digits() {
        assert_eq!(superscript_number(1), "¹");
        assert_eq!(superscript_number(16), "¹⁶");
        assert_eq!(superscript_number(100), "¹⁰⁰");
    }
}
