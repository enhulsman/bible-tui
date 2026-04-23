use super::canon;
use super::model::VerseRef;

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
}
