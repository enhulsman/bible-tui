//! C1 public suite: whole-chapter ranges in `parse_range`. See `spec/c1.md`.
//! Test names carry the SHALL number they check. No scripture text here:
//! the parser works on canon data only.

use std::collections::HashSet;

use bible_core::bible::model::{VerseRange, VerseRef};
use bible_core::bible::reference::parse_range;

const END: u8 = VerseRange::END_OF_CHAPTER;

const GEN: u8 = 0;
const PSA: u8 = 18;
const SNG: u8 = 21;
const OBA: u8 = 30;
const JHN: u8 = 42;
const ROM: u8 = 44;
const CO1: u8 = 45;

fn vr(book_index: u8, chapter: u16, verse: u8) -> VerseRef {
    VerseRef { book_index, chapter, verse }
}

fn range(start: VerseRef, end: VerseRef) -> VerseRange {
    VerseRange { start, end }
}

fn chapters(book: u8, first: u16, last: u16) -> VerseRange {
    range(vr(book, first, 1), vr(book, last, END))
}

fn parse(input: &str) -> Option<VerseRange> {
    parse_range(input, None, None)
}

fn assert_none(input: &str, book: Option<u8>, chapter: Option<u16>) {
    let got = parse_range(input, book, chapter);
    assert!(
        got.is_none(),
        "parse_range({input:?}, {book:?}, {chapter:?}) should be None, got {got:?}"
    );
}

// --- SHALL 1: public shape frozen ---

#[test]
fn shall01_public_shape_is_frozen() {
    let f: fn(&str, Option<u8>, Option<u16>) -> Option<VerseRange> = parse_range;
    let _ = f;

    let start = VerseRef { book_index: 0u8, chapter: 1u16, verse: 1u8 };
    let r = VerseRange { start, end: VerseRef { book_index: 0, chapter: 2, verse: 3 } };
    let VerseRange { start: s, end: e } = r;
    let VerseRef { book_index, chapter, verse } = e;
    assert_eq!((book_index, chapter, verse), (0u8, 2u16, 3u8));
    assert_eq!(s, start);

    // Copy + Clone + Eq + Hash + Debug
    let copied = r;
    #[allow(clippy::clone_on_copy)]
    let cloned = r.clone();
    assert_eq!(copied, cloned);
    let mut set = HashSet::new();
    set.insert(r);
    set.insert(copied);
    assert_eq!(set.len(), 1);
    assert!(!format!("{r:?}").is_empty());

    assert_eq!(VerseRange::END_OF_CHAPTER, u8::MAX);
}

// --- SHALL 2: whole-chapter range ---

#[test]
fn shall02_rom_8_9_is_chapter_range() {
    assert_eq!(parse("Rom 8-9"), Some(chapters(ROM, 8, 9)));
}

#[test]
fn shall02_full_and_code_book_names() {
    assert_eq!(parse("Romans 8-9"), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse("ROM 8-9"), Some(chapters(ROM, 8, 9)));
}

#[test]
fn shall02_numbered_book() {
    assert_eq!(parse("1 Cor 13-14"), Some(chapters(CO1, 13, 14)));
}

#[test]
fn shall02_multiword_book() {
    assert_eq!(parse("Song of Solomon 1-2"), Some(chapters(SNG, 1, 2)));
}

#[test]
fn shall02_up_to_last_chapter() {
    assert_eq!(parse("Psa 149-150"), Some(chapters(PSA, 149, 150)));
    assert_eq!(parse("Gen 1-50"), Some(chapters(GEN, 1, 50)));
    assert_eq!(parse("Rom 1-16"), Some(chapters(ROM, 1, 16)));
}

// --- SHALL 3: single chapter as a range ---

#[test]
fn shall03_rom_8_8_equals_rom_8() {
    let expected = chapters(ROM, 8, 8);
    assert_eq!(parse("Rom 8-8"), Some(expected));
    assert_eq!(parse("Rom 8"), Some(expected));
}

#[test]
fn shall03_single_chapter_book_and_last_chapter() {
    assert_eq!(parse("Oba 1-1"), Some(chapters(OBA, 1, 1)));
    assert_eq!(parse("Rom 16-16"), Some(chapters(ROM, 16, 16)));
}

// --- SHALL 4: reversed rejected ---

#[test]
fn shall04_reversed_chapters_rejected() {
    assert_none("Rom 9-8", None, None);
    assert_none("Rom 16-1", None, None);
    assert_none("Rom 9\u{2013}8", None, None);
}

// --- SHALL 5: out-of-canon rejected ---

#[test]
fn shall05_out_of_canon_rejected() {
    assert_none("Rom 8-17", None, None); // Romans has 16 chapters
    assert_none("Rom 17-18", None, None);
    assert_none("Rom 0-2", None, None);
    assert_none("Rom 8-0", None, None);
    assert_none("Oba 1-2", None, None); // Obadiah has 1 chapter
    assert_none("Psa 150-151", None, None);
}

#[test]
fn shall05_oversized_numbers_do_not_panic() {
    assert_none("Rom 8-300", None, None);
    assert_none("Rom 8-70000", None, None);
    assert_none("Rom 8-99999999999999999999", None, None);
}

// --- SHALL 6: separators ---

#[test]
fn shall06_en_dash_accepted() {
    assert_eq!(parse("Rom 8\u{2013}9"), Some(chapters(ROM, 8, 9)));
}

#[test]
fn shall06_whitespace_around_separator() {
    assert_eq!(parse("Rom 8 - 9"), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse("Rom 8 \u{2013} 9"), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse("  Rom 8-9  "), Some(chapters(ROM, 8, 9)));
}

// --- SHALL 7: context book, no context chapter ---

#[test]
fn shall07_context_book_chapter_range() {
    assert_eq!(parse_range("8-9", Some(ROM), None), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse_range("8\u{2013}9", Some(ROM), None), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse_range("16-18", Some(JHN), None), Some(chapters(JHN, 16, 18)));
}

#[test]
fn shall07_context_book_single_chapter_range() {
    assert_eq!(parse_range("8-8", Some(ROM), None), Some(chapters(ROM, 8, 8)));
}

#[test]
fn shall07_context_book_rejections() {
    assert_none("9-8", Some(ROM), None);
    assert_none("8-17", Some(ROM), None);
    assert_none("0-2", Some(ROM), None);
    assert_none("1-2", Some(OBA), None);
}

// --- SHALL 8: context book and chapter unchanged ---

#[test]
fn shall08_context_chapter_keeps_verse_ranges() {
    assert_eq!(
        parse_range("16-18", Some(JHN), Some(3)),
        Some(range(vr(JHN, 3, 16), vr(JHN, 3, 18)))
    );
    assert_eq!(
        parse_range("8-9", Some(ROM), Some(3)),
        Some(range(vr(ROM, 3, 8), vr(ROM, 3, 9)))
    );
    assert_eq!(
        parse_range("8-9:5", Some(ROM), Some(3)),
        Some(range(vr(ROM, 3, 8), vr(ROM, 9, 5)))
    );
}

#[test]
fn shall08_no_context_still_rejected() {
    assert_none("8-9", None, None);
    assert_none("8-9", None, Some(3));
}

// --- SHALL 9: explicit book wins ---

#[test]
fn shall09_named_book_overrides_context() {
    assert_eq!(parse_range("Rom 8-9", Some(GEN), Some(3)), Some(chapters(ROM, 8, 9)));
    assert_eq!(parse_range("Rom 8-9", Some(GEN), None), Some(chapters(ROM, 8, 9)));
}

// --- SHALL 10: chapter to verse (DECISION D1) ---

#[test]
fn shall10_chapter_to_verse() {
    assert_eq!(parse("Rom 8-9:5"), Some(range(vr(ROM, 8, 1), vr(ROM, 9, 5))));
    assert_eq!(parse("Rom 8 \u{2013} 9:5"), Some(range(vr(ROM, 8, 1), vr(ROM, 9, 5))));
}

#[test]
fn shall10_chapter_to_verse_same_chapter() {
    assert_eq!(parse("Rom 9-9:5"), Some(range(vr(ROM, 9, 1), vr(ROM, 9, 5))));
}

#[test]
fn shall10_chapter_to_verse_with_context_book() {
    assert_eq!(
        parse_range("8-9:5", Some(ROM), None),
        Some(range(vr(ROM, 8, 1), vr(ROM, 9, 5)))
    );
}

#[test]
fn shall10_chapter_to_verse_rejections() {
    assert_none("Rom 9-8:5", None, None);
    assert_none("Rom 8-17:1", None, None);
    assert_none("Rom 8-9:0", None, None);
    assert_none("9-8:5", Some(ROM), None);
}

// --- SHALL 11: verse forms unchanged ---

#[test]
fn shall11_bare_end_after_verse_is_still_a_verse() {
    assert_eq!(parse("Rom 8:5-9"), Some(range(vr(ROM, 8, 5), vr(ROM, 8, 9))));
    assert_eq!(parse("John 3:16-18"), Some(range(vr(JHN, 3, 16), vr(JHN, 3, 18))));
}

#[test]
fn shall11_existing_forms_unchanged() {
    assert_eq!(parse("Gen 1:1-2:3"), Some(range(vr(GEN, 1, 1), vr(GEN, 2, 3))));
    assert_eq!(parse("Rom 8"), Some(chapters(ROM, 8, 8)));
    assert_eq!(parse("Genesis"), Some(chapters(GEN, 1, 1)));
    assert_eq!(parse("John 3:16"), Some(range(vr(JHN, 3, 16), vr(JHN, 3, 16))));
    assert_eq!(
        parse_range("3:16-4:2", Some(JHN), None),
        Some(range(vr(JHN, 3, 16), vr(JHN, 4, 2)))
    );
    assert_none("Rom 8:9-5", None, None);
}

// --- SHALL 12: malformed rejected ---

#[test]
fn shall12_malformed_rejected() {
    for input in [
        "Rom 8-",
        "Rom 8-9-10",
        "Rom 8-9 extra",
        "Rom 8-x",
        "Rom 8-1 Cor 2",
        "Rom 8-9:",
        "Rom 8-:5",
    ] {
        assert_none(input, None, None);
    }
    assert_none("8-", Some(ROM), None);
    assert_none("8-9-10", Some(ROM), None);
}
