use bible_core::action::Action;
use bible_core::app::App;
use bible_core::bible::loader;
use bible_core::bible::model::{BibleData, BundledTranslation, TranslationId, TranslationInfo};
use bible_core::keys::{KeyCode, KeyEvent};
use bible_core::search::index::SearchIndex;
use bible_core::persistence::{NoPersistence, TranslationProvider};
use ratatui::{backend::TestBackend, Terminal};

struct BundledProvider;

impl TranslationProvider for BundledProvider {
    fn available_translations(&self) -> Vec<TranslationInfo> {
        vec![BundledTranslation::Kjv.info()]
    }

    fn load_translation(&self, _: &TranslationId) -> Option<BibleData> {
        Some(loader::load_bundled(BundledTranslation::Kjv))
    }
}

fn app() -> App {
    App::new(Box::new(NoPersistence), Box::new(BundledProvider))
}

fn key(app: &mut App, code: KeyCode) {
    let action = app.handle_key(KeyEvent::plain(code));
    app.update(&action);
}

fn goto(app: &mut App, reference: &str) {
    key(app, KeyCode::Char(':'));
    for c in format!("goto {reference}").chars() {
        key(app, KeyCode::Char(c));
    }
    key(app, KeyCode::Enter);
}

fn render(app: &mut App, width: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, 12)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..12)
        .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn assert_at(app: &mut App, width: u16, verse: u8, first_line: &str) {
    let rows = render(app, width);
    assert!(rows[11].contains(&format!("John 8:{verse}")), "{rows:?}");
    assert!(rows[0].starts_with(first_line), "{rows:?}");
}

#[test]
fn search_selection_opens_john_8_12() {
    let mut app = app();
    render(&mut app, 80);
    key(&mut app, KeyCode::Char('/'));
    for c in "light of the world".chars() {
        key(&mut app, KeyCode::Char(c));
    }
    let bible = loader::load_bundled(BundledTranslation::Kjv);
    let results = SearchIndex::build(&bible).search("light of the world");
    let selected = results.iter().position(|r| {
        r.verse_ref.book_index == 42 && r.verse_ref.chapter == 8 && r.verse_ref.verse == 12
    }).unwrap();
    for _ in 0..selected {
        key(&mut app, KeyCode::Down);
    }
    key(&mut app, KeyCode::Enter);
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
    app.update(&Action::SearchNext);
    app.update(&Action::SearchPrev);
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
}

#[test]
fn goto_with_verse_opens_its_first_line() {
    let mut app = app();
    render(&mut app, 80);
    goto(&mut app, "John 8:12");
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
}

#[test]
fn goto_without_verse_starts_at_one() {
    let mut app = app();
    goto(&mut app, "John 8:12");
    goto(&mut app, "John 8");
    let rows = render(&mut app, 80);
    assert!(rows[11].contains("John 8:1 "), "{rows:?}");
    // Unchanged from before the fix: the chapter opens at the top, title visible.
    assert!(rows[0].starts_with("John 8"), "{rows:?}");
}

#[test]
fn goto_first_verse_keeps_chapter_title_visible() {
    let mut app = app();
    render(&mut app, 80);
    goto(&mut app, "John 8:1");
    let rows = render(&mut app, 80);
    assert!(rows[11].contains("John 8:1 "), "{rows:?}");
    assert!(rows[0].starts_with("John 8"), "{rows:?}");
}

#[test]
fn navigation_panel_starts_at_one() {
    let mut app = app();
    goto(&mut app, "John 8:12");
    key(&mut app, KeyCode::Tab);
    key(&mut app, KeyCode::Right);
    for _ in 0..7 {
        key(&mut app, KeyCode::Down);
    }
    key(&mut app, KeyCode::Enter);
    let rows = render(&mut app, 80);
    assert!(rows[11].contains("John 8:1 "), "{rows:?}");
    assert!(rows[0].starts_with("John 8"), "{rows:?}");
}

#[test]
fn out_of_range_verse_clamps_to_last_verse() {
    let mut app = app();
    render(&mut app, 80);
    goto(&mut app, "John 8:99");
    assert_at(&mut app, 80, 59, "⁵⁹ Then took they up stones");
}

#[test]
fn narrow_width_and_resize_keep_target_on_its_first_wrapped_line() {
    let mut app = app();
    render(&mut app, 28);
    goto(&mut app, "John 8:12");
    assert_at(&mut app, 28, 12, "¹² Then spake Jesus");
    // Explicit resize and the render fallback must both re-anchor the verse.
    app.update(&Action::Resize(45, 12));
    assert_at(&mut app, 45, 12, "¹² Then spake Jesus");
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
    assert_at(&mut app, 18, 12, "¹² Then spake");
}

#[test]
fn section_heading_does_not_replace_the_target_verse_text() {
    use bible_core::bible::model::{Chapter, SectionHeading, Verse, VerseSpan};
    use bible_core::components::reading_pane::ReadingPane;

    let chapter = Chapter {
        number: 1,
        verses: (1..=3).map(|number| Verse {
            number,
            spans: vec![VerseSpan::Plain("Verse text that wraps at a narrow width.".into())],
            paragraph_break: false,
        }).collect(),
        headings: vec![SectionHeading { text: "Section title".into(), before_verse: 2 }],
    };
    let mut pane = ReadingPane::new();
    pane.set_chapter(0, "Test", &chapter, 24);
    pane.scroll_to_verse(2);
    for width in [24, 48] {
        pane.rebuild_lines(&chapter, width);
        let mut terminal = Terminal::new(TestBackend::new(width, 10)).unwrap();
        terminal.draw(|frame| pane.render(frame, frame.area())).unwrap();
        let row: String = (0..width).map(|x| terminal.backend().buffer()[(x, 0)].symbol()).collect();
        assert!(row.starts_with("² Verse text"), "{row:?}");
        assert_eq!(pane.current_verse_approx(), 2);
    }
}

#[test]
fn last_verse_survives_resize_and_down_does_not_jump_backwards() {
    let mut app = app();
    render(&mut app, 28);
    goto(&mut app, "John 8:99");
    assert_at(&mut app, 28, 59, "⁵⁹ Then took they up");
    assert_at(&mut app, 80, 59, "⁵⁹ Then took they up stones");
    key(&mut app, KeyCode::Down);
    assert_at(&mut app, 80, 59, "⁵⁹ Then took they up stones");
}
