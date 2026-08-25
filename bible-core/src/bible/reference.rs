use super::canon;
use super::model::{VerseRange, VerseRef};

/// Parse a Bible reference string into a VerseRef.
///
/// Supports formats:
/// - "John 3:16" → full reference
/// - "1 Corinthians 13:4" → numbered book
/// - "Gen 1:1" → abbreviation
/// - "3:16" → chapter:verse (needs context book)
/// - "16" → verse only (needs context book+chapter)
pub fn parse_reference(input: &str, context_book: Option<u8>, context_chapter: Option<u16>) -> Option<VerseRef> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Try full reference: "Book Chapter:Verse" or "Book Chapter"
    if let Some(vr) = try_full_reference(input) {
        return Some(vr);
    }

    // Try chapter:verse with context book
    if let Some(book) = context_book {
        if let Some(vr) = try_chapter_verse(input, book) {
            return Some(vr);
        }

        // Try verse-only with context book and chapter
        if let Some(chapter) = context_chapter {
            if let Ok(verse) = input.parse::<u8>() {
                return Some(VerseRef {
                    book_index: book,
                    chapter,
                    verse,
                });
            }
        }
    }

    None
}

/// Parse a verse-range reference string into a VerseRange.
///
/// Supports formats:
/// - "John 3:16-18" → verse range within a chapter
/// - "Gen 1:1-2:3" → cross-chapter range
/// - "Rom 8" → whole chapter (end verse = `VerseRange::END_OF_CHAPTER`)
/// - "Genesis" → whole first chapter (parity with `parse_reference`)
/// - "John 3:16" → single verse as a one-verse range
/// - "3:16-18" → needs context book
/// - "16-18" / "16" → needs context book + chapter
///
/// Endpoints may be separated by '-' or '–' (en dash), with optional
/// whitespace around the dash. A bare number after the dash is a verse in
/// the start's chapter; "chapter:verse" after the dash switches chapters.
///
/// Returns None for reversed ranges, chapters outside the canon,
/// chapter/verse 0, trailing junk, or otherwise unparseable input. Verse
/// numbers are NOT validated against per-chapter verse counts — that is
/// translation data, unavailable at this layer.
pub fn parse_range(
    input: &str,
    context_book: Option<u8>,
    context_chapter: Option<u16>,
) -> Option<VerseRange> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Endpoints may be separated by '-' or '–' (en dash), with optional
    // surrounding whitespace.
    let normalized = input.replace('\u{2013}', "-");
    let (start_part, end_part) = match normalized.split_once('-') {
        Some((start, end)) => (start.trim(), Some(end.trim())),
        None => (normalized.as_str(), None),
    };

    match resolve_start(start_part, context_book, context_chapter)? {
        StartEndpoint::WholeChapter(book_index, chapter) => {
            // Whole-chapter references don't compose into ranges.
            if end_part.is_some() {
                return None;
            }
            Some(VerseRange {
                start: VerseRef { book_index, chapter, verse: 1 },
                end: VerseRef { book_index, chapter, verse: VerseRange::END_OF_CHAPTER },
            })
        }
        StartEndpoint::Single(start) => {
            let end = match end_part {
                Some(end_part) => resolve_end(end_part, start.book_index, start)?,
                None => start,
            };
            Some(VerseRange { start, end })
        }
    }
}

/// A resolved left-hand endpoint of a range.
enum StartEndpoint {
    /// An exact verse; usable as either endpoint of a range.
    Single(VerseRef),
    /// A whole chapter, keyed by book index and chapter.
    WholeChapter(u8, u16),
}

fn chapter_in_canon(book_index: u8, chapter: u16) -> bool {
    chapter >= 1 && chapter <= canon::CANON[book_index as usize].chapter_count
}

fn single_verse(book_index: u8, chapter: u16, verse: u8) -> Option<StartEndpoint> {
    if !chapter_in_canon(book_index, chapter) || verse == 0 {
        return None;
    }
    Some(StartEndpoint::Single(VerseRef { book_index, chapter, verse }))
}

fn whole_chapter(book_index: u8, chapter: u16) -> Option<StartEndpoint> {
    if !chapter_in_canon(book_index, chapter) {
        return None;
    }
    Some(StartEndpoint::WholeChapter(book_index, chapter))
}

fn resolve_start(
    part: &str,
    context_book: Option<u8>,
    context_chapter: Option<u16>,
) -> Option<StartEndpoint> {
    if let Some((book_index, remainder)) = split_book_prefix(part) {
        let book_index = book_index as u8;
        if remainder.is_empty() {
            // Bare book name: whole first chapter.
            return whole_chapter(book_index, 1);
        }
        return match remainder.split_once(':') {
            Some((ch, vs)) => {
                single_verse(book_index, ch.trim().parse().ok()?, vs.trim().parse().ok()?)
            }
            None => whole_chapter(book_index, remainder.trim().parse().ok()?),
        };
    }

    // Partial forms need a context book; a bare verse also needs a chapter.
    let book_index = context_book?;
    match part.split_once(':') {
        Some((ch, vs)) => single_verse(book_index, ch.trim().parse().ok()?, vs.trim().parse().ok()?),
        None => {
            let chapter = context_chapter?;
            single_verse(book_index, chapter, part.parse().ok()?)
        }
    }
}

fn resolve_end(part: &str, book_index: u8, start: VerseRef) -> Option<VerseRef> {
    let end = match part.split_once(':') {
        Some((ch, vs)) => VerseRef {
            book_index,
            chapter: ch.trim().parse().ok()?,
            verse: vs.trim().parse().ok()?,
        },
        None => VerseRef {
            book_index,
            chapter: start.chapter,
            verse: part.parse().ok()?,
        },
    };
    if !chapter_in_canon(book_index, end.chapter) || end.verse == 0 {
        return None;
    }
    if (end.chapter, end.verse) < (start.chapter, start.verse) {
        return None;
    }
    Some(end)
}

/// Split a leading book name off `input`, returning the book index and the
/// unparsed remainder ("John 3:16" → `(42, "3:16")`). Returns None when no
/// book name matches; an exact match on the whole input yields an empty
/// remainder.
fn split_book_prefix(input: &str) -> Option<(usize, String)> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    // Books can start with a digit ("1 John"), so try progressively shorter
    // prefixes as book names — longest first so multi-word names win.
    for split_at in (1..words.len()).rev() {
        let book_part = words[..split_at].join(" ");
        if let Some(index) = canon::find_book(&book_part) {
            return Some((index, words[split_at..].join(" ")));
        }
    }

    // The whole input may itself be a book name.
    canon::find_book(input).map(|index| (index, String::new()))
}

fn try_full_reference(input: &str) -> Option<VerseRef> {
    // Find the split between book name and numbers.
    // Books can start with a digit ("1 John"), so we need to find the last
    // word boundary before a number pattern like "3:16" or just "3".
    let mut book_end = None;

    // Strategy: try progressively shorter prefixes as book names
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    for split_at in (1..words.len()).rev() {
        let book_part = words[..split_at].join(" ");
        if canon::find_book(&book_part).is_some() {
            book_end = Some((split_at, book_part));
            break;
        }
    }

    // Also try the whole input as a book name (no chapter/verse specified)
    if book_end.is_none() {
        if let Some(_idx) = canon::find_book(input) {
            return Some(VerseRef {
                book_index: _idx as u8,
                chapter: 1,
                verse: 1,
            });
        }
    }

    let (split_at, book_name) = book_end?;
    let book_index = canon::find_book(&book_name)? as u8;
    let remainder = words[split_at..].join(" ");

    // Parse "chapter:verse" or just "chapter"
    if let Some((ch, vs)) = remainder.split_once(':') {
        let chapter = ch.trim().parse::<u16>().ok()?;
        let verse = vs.trim().parse::<u8>().ok()?;
        Some(VerseRef { book_index, chapter, verse })
    } else {
        let chapter = remainder.trim().parse::<u16>().ok()?;
        Some(VerseRef { book_index, chapter, verse: 1 })
    }
}

fn try_chapter_verse(input: &str, book_index: u8) -> Option<VerseRef> {
    if let Some((ch, vs)) = input.split_once(':') {
        let chapter = ch.trim().parse::<u16>().ok()?;
        let verse = vs.trim().parse::<u8>().ok()?;
        Some(VerseRef { book_index, chapter, verse })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_reference() {
        let vr = parse_reference("John 3:16", None, None).unwrap();
        assert_eq!(vr.book_index, 42); // John is index 42
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn parse_numbered_book() {
        let vr = parse_reference("1 Corinthians 13:4", None, None).unwrap();
        assert_eq!(vr.book_index, 45);
        assert_eq!(vr.chapter, 13);
        assert_eq!(vr.verse, 4);
    }

    #[test]
    fn parse_abbreviation() {
        let vr = parse_reference("Gen 1:1", None, None).unwrap();
        assert_eq!(vr.book_index, 0);
        assert_eq!(vr.chapter, 1);
        assert_eq!(vr.verse, 1);
    }

    #[test]
    fn parse_chapter_verse_with_context() {
        let vr = parse_reference("3:16", Some(42), None).unwrap();
        assert_eq!(vr.book_index, 42);
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn parse_verse_only_with_context() {
        let vr = parse_reference("16", Some(42), Some(3)).unwrap();
        assert_eq!(vr.book_index, 42);
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn parse_book_name_only() {
        let vr = parse_reference("Genesis", None, None).unwrap();
        assert_eq!(vr.book_index, 0);
        assert_eq!(vr.chapter, 1);
        assert_eq!(vr.verse, 1);
    }

    #[test]
    fn parse_revelation_last_verse() {
        let vr = parse_reference("Rev 22:21", None, None).unwrap();
        assert_eq!(vr.book_index, 65);
        assert_eq!(vr.chapter, 22);
        assert_eq!(vr.verse, 21);
    }

    #[test]
    fn parse_prefix_book_name() {
        let vr = parse_reference("Psalm 23", None, None).unwrap();
        assert_eq!(vr.book_index, 18); // Psalms
        assert_eq!(vr.chapter, 23);
    }

    #[test]
    fn parse_numbered_prefix() {
        let vr = parse_reference("1 Cor 13", None, None).unwrap();
        assert_eq!(vr.book_index, 45); // 1 Corinthians
        assert_eq!(vr.chapter, 13);
    }

    #[test]
    fn parse_ambiguous_prefix_fails() {
        // "Jo" matches Job, Joel, Jonah, Joshua, John — ambiguous
        assert!(parse_reference("Jo 3", None, None).is_none());
    }

    #[test]
    fn parse_bare_prefix_book() {
        let vr = parse_reference("Psalm", None, None).unwrap();
        assert_eq!(vr.book_index, 18);
        assert_eq!(vr.chapter, 1);
    }

    // --- parse_range ---

    fn vr(book_index: u8, chapter: u16, verse: u8) -> VerseRef {
        VerseRef { book_index, chapter, verse }
    }

    #[test]
    fn range_within_chapter() {
        let r = parse_range("John 3:16-18", None, None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 18));
    }

    #[test]
    fn range_cross_chapter() {
        let r = parse_range("Gen 1:1-2:3", None, None).unwrap();
        assert_eq!(r.start, vr(0, 1, 1));
        assert_eq!(r.end, vr(0, 2, 3));
    }

    #[test]
    fn range_whole_chapter() {
        let r = parse_range("Rom 8", None, None).unwrap();
        assert_eq!(r.start, vr(44, 8, 1));
        assert_eq!(r.end, vr(44, 8, VerseRange::END_OF_CHAPTER));
    }

    #[test]
    fn range_single_verse() {
        let r = parse_range("John 3:16", None, None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 16));
    }

    #[test]
    fn range_equal_endpoints() {
        let r = parse_range("John 3:16-16", None, None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 16));
    }

    #[test]
    fn range_bare_book_name() {
        let r = parse_range("Genesis", None, None).unwrap();
        assert_eq!(r.start, vr(0, 1, 1));
        assert_eq!(r.end, vr(0, 1, VerseRange::END_OF_CHAPTER));
    }

    #[test]
    fn range_multiword_book_cross_chapter() {
        let r = parse_range("Song of Solomon 1:1-2:3", None, None).unwrap();
        assert_eq!(r.start, vr(21, 1, 1));
        assert_eq!(r.end, vr(21, 2, 3));
    }

    #[test]
    fn range_numbered_book() {
        let r = parse_range("1 Cor 13:4-7", None, None).unwrap();
        assert_eq!(r.start, vr(45, 13, 4));
        assert_eq!(r.end, vr(45, 13, 7));
    }

    #[test]
    fn range_whitespace_around_dash() {
        let r = parse_range("John 3:16 - 18", None, None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 18));
    }

    #[test]
    fn range_en_dash() {
        let r = parse_range("John 3:16\u{2013}18", None, None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 18));
    }

    #[test]
    fn range_context_book() {
        let r = parse_range("3:16-18", Some(42), None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 18));
    }

    #[test]
    fn range_context_book_cross_chapter() {
        let r = parse_range("3:16-4:2", Some(42), None).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 4, 2));
    }

    #[test]
    fn range_context_book_and_chapter() {
        let r = parse_range("16-18", Some(42), Some(3)).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 18));
    }

    #[test]
    fn range_context_single_verse() {
        let r = parse_range("16", Some(42), Some(3)).unwrap();
        assert_eq!(r.start, vr(42, 3, 16));
        assert_eq!(r.end, vr(42, 3, 16));
    }

    #[test]
    fn range_reversed_verses_none() {
        assert!(parse_range("John 3:18-16", None, None).is_none());
    }

    #[test]
    fn range_reversed_chapters_none() {
        assert!(parse_range("Gen 2:3-1:1", None, None).is_none());
    }

    #[test]
    fn range_reversed_with_context_none() {
        assert!(parse_range("18-16", Some(42), Some(3)).is_none());
    }

    #[test]
    fn range_start_chapter_out_of_canon_none() {
        // John has 21 chapters
        assert!(parse_range("John 99:1-2", None, None).is_none());
    }

    #[test]
    fn range_end_chapter_out_of_canon_none() {
        assert!(parse_range("John 3:16-99:1", None, None).is_none());
    }

    #[test]
    fn range_whole_chapter_out_of_canon_none() {
        // Romans has 16 chapters
        assert!(parse_range("Rom 99", None, None).is_none());
    }

    #[test]
    fn range_chapter_zero_none() {
        assert!(parse_range("John 0:1-2", None, None).is_none());
    }

    #[test]
    fn range_verse_zero_none() {
        assert!(parse_range("John 3:0-5", None, None).is_none());
    }

    #[test]
    fn range_garbage_none() {
        assert!(parse_range("", None, None).is_none());
        assert!(parse_range("wibble", None, None).is_none());
        assert!(parse_range("wibble 3:16-18", None, None).is_none());
        assert!(parse_range("John 3:16-", None, None).is_none());
        assert!(parse_range("-", None, None).is_none());
        assert!(parse_range("John 3:16-18-20", None, None).is_none());
        assert!(parse_range("John 3:16-18 extra", None, None).is_none());
    }

    #[test]
    fn range_needs_context_none() {
        // No context book/chapter → partial forms must not parse
        assert!(parse_range("3:16-18", None, None).is_none());
        assert!(parse_range("16-18", None, None).is_none());
        assert!(parse_range("16-18", Some(42), None).is_none());
    }
}
