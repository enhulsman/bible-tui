use std::cell::RefCell;
use std::rc::Rc;

use bible_core::action::Action;
use bible_core::app::{App, Mode};
use bible_core::bible::loader;
use bible_core::bible::model::{
    BibleData, BundledTranslation, TranslationId, TranslationInfo, VerseRef,
};
use bible_core::keys::{KeyCode, KeyEvent};
use bible_core::persistence::{NoPersistence, Persistence, TranslationProvider};
use bible_core::storage::bookmark::{Bookmark, BookmarkStore};
use bible_core::storage::state::AppState;
use chrono::{TimeZone, Utc};
use ratatui::{backend::TestBackend, Terminal};

struct Provider;

impl TranslationProvider for Provider {
    fn available_translations(&self) -> Vec<TranslationInfo> {
        vec![
            BundledTranslation::Kjv.info(),
            TranslationInfo {
                id: TranslationId::Imported("short".into()),
                name: "Without Revelation".into(),
                abbreviation: "SHORT".into(),
            },
            TranslationInfo {
                id: TranslationId::Imported("missing-chapter".into()),
                name: "Without Genesis 1".into(),
                abbreviation: "NOCHAPTER".into(),
            },
            TranslationInfo {
                id: TranslationId::Imported("missing-verse".into()),
                name: "Genesis 1 with verses 1, 2, 4".into(),
                abbreviation: "NOVERSE".into(),
            },
        ]
    }

    fn load_translation(&self, id: &TranslationId) -> Option<BibleData> {
        let mut bible = loader::load_bundled(BundledTranslation::Kjv);
        if let TranslationId::Imported(name) = id {
            match name.as_str() {
                // Remove only the trailing book, preserving all positional indices.
                "short" => {
                    bible.books.pop();
                }
                "missing-chapter" => {
                    bible.books[0].chapters.remove(0);
                }
                "missing-verse" => {
                    bible.books[0].chapters[0]
                        .verses
                        .retain(|verse| matches!(verse.number, 1 | 2 | 4));
                }
                _ => {}
            }
            bible.translation = id.clone();
        }
        Some(bible)
    }
}

struct MemoryPersistence {
    bookmarks: Vec<Bookmark>,
    saved: Rc<RefCell<Vec<VerseRef>>>,
}

impl Persistence for MemoryPersistence {
    fn load_bookmarks(&self) -> BookmarkStore {
        BookmarkStore {
            bookmarks: self.bookmarks.clone(),
        }
    }
    fn save_bookmarks(&self, store: &BookmarkStore) {
        *self.saved.borrow_mut() = store.bookmarks.iter().map(Bookmark::verse_ref).collect();
    }
    fn load_state(&self) -> AppState {
        AppState::default()
    }
    fn save_state(&self, _: &AppState) {}
}

fn bookmark(book_index: u8, chapter: u16, verse: u8, seconds: i64) -> Bookmark {
    Bookmark {
        book_index,
        chapter,
        verse,
        note: None,
        created_at: Utc.timestamp_opt(1_700_000_000 + seconds, 0).unwrap(),
    }
}

fn seeded(bookmarks: Vec<Bookmark>) -> App {
    App::new(
        Box::new(MemoryPersistence {
            bookmarks,
            saved: Rc::default(),
        }),
        Box::new(Provider),
    )
}

fn key(app: &mut App, code: KeyCode) {
    let action = app.handle_key(KeyEvent::plain(code));
    app.update(&action);
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        key(app, KeyCode::Char(c));
    }
}

fn type_command(app: &mut App, text: &str) {
    key(app, KeyCode::Char(':'));
    type_text(app, text);
}

fn command(app: &mut App, text: &str) {
    type_command(app, text);
    key(app, KeyCode::Enter);
}

fn rows(app: &mut App, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn at(app: &mut App, reference: &str, first_line: &str, width: u16) {
    let r = rows(app, width, 12);
    assert!(r[11].contains(reference), "{r:?}");
    assert!(r[0].starts_with(first_line), "{r:?}");
}

fn assert_order(app: &mut App, expected: &[&str]) {
    let text = rows(app, 100, 24).join("\n");
    let positions: Vec<_> = expected
        .iter()
        .map(|s| {
            text.find(s)
                .unwrap_or_else(|| panic!("missing {s}: {text}"))
        })
        .collect();
    assert!(positions.windows(2).all(|p| p[0] < p[1]), "{text}");
}

#[test]
fn aliases_open_without_moving_then_enter_jumps() {
    for alias in ["bookmarks", "bm"] {
        let mut app = seeded(vec![bookmark(42, 8, 12, 0)]);
        command(&mut app, "goto Genesis 1:1");
        command(&mut app, alias);
        assert_eq!(app.mode, Mode::Command);
        assert_order(&mut app, &["John 8:12"]);
        key(&mut app, KeyCode::Enter);
        assert_eq!(app.mode, Mode::Normal);
        at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    }
}

#[test]
fn newest_first_is_a_view_and_does_not_reorder_persistence() {
    let bookmarks = vec![
        bookmark(42, 8, 12, 1),
        bookmark(0, 1, 3, 3),
        bookmark(42, 3, 16, 2),
    ];
    let expected: Vec<_> = bookmarks.iter().map(Bookmark::verse_ref).collect();
    let saved = Rc::default();
    let mut app = App::new(
        Box::new(MemoryPersistence {
            bookmarks,
            saved: Rc::clone(&saved),
        }),
        Box::new(Provider),
    );
    command(&mut app, "bookmarks");
    assert_order(&mut app, &["Genesis 1:3", "John 3:16", "John 8:12"]);
    key(&mut app, KeyCode::Esc);
    app.save_state();
    assert_eq!(*saved.borrow(), expected);
}

#[test]
fn equal_timestamps_use_book_chapter_then_verse() {
    let mut app = seeded(vec![
        bookmark(42, 8, 16, 0),
        bookmark(61, 1, 1, 0),
        bookmark(42, 3, 16, 0),
        bookmark(0, 1, 3, 0),
        bookmark(42, 8, 12, 0),
    ]);
    command(&mut app, "bm");
    assert_order(
        &mut app,
        &[
            "Genesis 1:3",
            "John 3:16",
            "John 8:12",
            "John 8:16",
            "1 John 1:1",
        ],
    );
}

#[test]
fn prefix_filter_is_case_insensitive_and_enter_uses_selection() {
    let mut app = seeded(vec![
        bookmark(42, 8, 12, 2),
        bookmark(42, 8, 16, 1),
        bookmark(61, 1, 1, 3),
    ]);
    type_command(&mut app, "bm jOhN 8:1");
    assert_order(&mut app, &["John 8:12", "John 8:16"]);
    assert!(!rows(&mut app, 80, 24).join("\n").contains("1 John"));
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:16", "¹⁶ And yet if I judge", 80);
}

#[test]
fn opening_then_typing_filters_and_up_changes_selection() {
    let mut app = seeded(vec![
        bookmark(42, 8, 12, 2),
        bookmark(42, 8, 16, 1),
        bookmark(0, 1, 3, 3),
    ]);
    command(&mut app, "bookmarks");
    type_text(&mut app, "john 8:1");
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Up);
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
}

#[test]
fn scrolling_reaches_bookmarks_beyond_fifteen() {
    let mut app = seeded(
        (1..=21)
            .map(|verse| bookmark(42, 8, verse, -(verse as i64)))
            .collect(),
    );
    command(&mut app, "bm");
    for _ in 0..20 {
        key(&mut app, KeyCode::Down);
        rows(&mut app, 80, 12);
    }
    let text = rows(&mut app, 80, 12).join("\n");
    assert!(text.contains("John 8:21"), "{text}");
    assert!(!text.contains("│John 8:1 "), "{text}");
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:21", "²¹ Then said Jesus again", 80);
}

#[test]
fn empty_browser_reports_status_and_keeps_position_for_both_aliases() {
    for alias in ["bookmarks", "bm", "bm "] {
        let mut app = seeded(vec![]);
        command(&mut app, "goto John 8:12");
        command(&mut app, alias);
        assert_eq!(app.mode, Mode::Normal);
        let text = rows(&mut app, 80, 12).join("\n");
        assert!(
            text.contains("No bookmarks yet: press B on a verse"),
            "{text}"
        );
        at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    }
}

#[test]
fn missing_translation_book_is_listed_but_cannot_move() {
    let mut app = seeded(vec![bookmark(65, 22, 20, 0)]);
    command(&mut app, "t short");
    command(&mut app, "goto John 8:12");
    command(&mut app, "bookmarks");
    assert_order(&mut app, &["Revelation 22:20"]);
    key(&mut app, KeyCode::Enter);
    let text = rows(&mut app, 80, 12).join("\n");
    assert!(text.contains("Not in SHORT"), "{text}");
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
}

#[test]
fn missing_translation_chapter_keeps_position_and_reports_status() {
    let mut app = seeded(vec![bookmark(0, 1, 1, 0)]);
    command(&mut app, "t nochapter");
    command(&mut app, "goto John 8:12");
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    command(&mut app, "bookmarks");
    assert_order(&mut app, &["Genesis 1:1"]);
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    let text = rows(&mut app, 80, 12).join("\n");
    assert!(text.contains("Not in NOCHAPTER"), "{text}");
}

#[test]
fn missing_translation_verse_keeps_position_and_reports_status() {
    let mut app = seeded(vec![bookmark(0, 1, 3, 0)]);
    command(&mut app, "t noverse");
    command(&mut app, "goto John 8:12");
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    command(&mut app, "bm");
    assert_order(&mut app, &["Genesis 1:3"]);
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    let text = rows(&mut app, 80, 12).join("\n");
    assert!(text.contains("Not in NOVERSE"), "{text}");
}

#[test]
fn opening_refreshes_after_toggle_including_no_persistence_session() {
    let mut app = App::new(Box::new(NoPersistence), Box::new(Provider));
    command(&mut app, "goto John 8:12");
    key(&mut app, KeyCode::Char('B'));
    command(&mut app, "bm");
    assert_order(&mut app, &["John 8:12"]);
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Char('B'));
    command(&mut app, "bm");
    assert!(rows(&mut app, 80, 12)
        .join("\n")
        .contains("No bookmarks yet: press B on a verse"));
    key(&mut app, KeyCode::Char('B'));
    command(&mut app, "goto Genesis 1:3");
    key(&mut app, KeyCode::Char('B'));
    command(&mut app, "bm");
    assert_order(&mut app, &["Genesis 1:3", "John 8:12"]);
}

#[test]
fn escape_closes_without_moving() {
    let mut app = seeded(vec![bookmark(0, 1, 3, 0), bookmark(42, 3, 16, 1)]);
    command(&mut app, "goto John 8:12");
    command(&mut app, "bm");
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.mode, Mode::Normal);
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
}

#[test]
fn narrow_jump_keeps_exact_verse_through_resize_and_placeholder_width() {
    for width in [28, 3] {
        let mut app = seeded(vec![bookmark(42, 8, 12, 0)]);
        rows(&mut app, width, 12);
        command(&mut app, "bm John 8:12");
        if width == 28 {
            at(&mut app, "John 8:12", "¹² Then spake Jesus", width);
        }
        app.update(&Action::Resize(80, 12));
        at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    }
}

#[test]
fn unmatched_filter_never_jumps_to_an_unsaved_reference() {
    let mut app = seeded(vec![bookmark(42, 8, 12, 0)]);
    command(&mut app, "goto Genesis 1:3");
    command(&mut app, "bm John 3:16");
    at(&mut app, "Genesis 1:3", "³  And God said", 80);
}

#[test]
fn bookmark_command_completion_and_tab_work_without_changing_goto() {
    let mut app = seeded(vec![bookmark(42, 8, 12, 0)]);
    type_command(&mut app, "book");
    assert!(rows(&mut app, 80, 12).join("\n").contains("bookmarks"));
    key(&mut app, KeyCode::Tab);
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:12", "¹² Then spake Jesus", 80);
    type_command(&mut app, "goto Joh");
    key(&mut app, KeyCode::Tab);
    type_text(&mut app, "8:16");
    key(&mut app, KeyCode::Enter);
    at(&mut app, "John 8:16", "¹⁶ And yet if I judge", 80);
    type_command(&mut app, "t sh");
    key(&mut app, KeyCode::Tab);
    key(&mut app, KeyCode::Enter);
    assert!(rows(&mut app, 80, 12)[11].contains("SHORT"));
}

#[test]
fn help_shows_the_approved_bookmark_interaction() {
    let mut app = seeded(vec![]);
    key(&mut app, KeyCode::Char('?'));
    let rendered = rows(&mut app, 80, 45);
    let start = rendered
        .iter()
        .position(|row| row.contains(":bookmarks / :bm"))
        .unwrap();
    // The 60-column help popup starts at x=10, with a 26-column key field.
    let description = rendered[start..]
        .iter()
        .enumerate()
        .take_while(|(i, row)| *i == 0 || row.chars().skip(11).take(26).all(|c| c == ' '))
        .map(|(_, row)| {
            row.chars()
                .skip(37)
                .take(32)
                .collect::<String>()
                .trim()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        description,
        "Browse bookmarks; type to filter, ↑/↓ select, Enter jumps"
    );
}
