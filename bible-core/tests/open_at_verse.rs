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
fn target_survives_placeholder_width_and_restores_on_resize() {
    let mut app = app();
    render(&mut app, 80);
    goto(&mut app, "John 8:12");
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
    app.update(&Action::Resize(3, 12));
    render(&mut app, 3);
    app.update(&Action::Resize(80, 12));
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
}

#[test]
fn navigation_at_placeholder_width_restores_target_when_widened() {
    let mut app = app();
    render(&mut app, 3);
    goto(&mut app, "John 8:12");
    render(&mut app, 3);
    assert_at(&mut app, 80, 12, "¹² Then spake Jesus");
}

#[test]
fn placeholder_navigation_preserves_first_verse_and_clamping() {
    for verse in [1, 99] {
        let mut app = app();
        render(&mut app, 3);
        goto(&mut app, &format!("John 8:{verse}"));
        if verse == 1 {
            assert_at(&mut app, 80, 1, "John 8");
        } else {
            assert_at(&mut app, 80, 59, "⁵⁹ Then took they up stones");
        }
    }
}

#[test]
fn chapter_navigation_clears_the_deferred_verse() {
    let mut app = app();
    render(&mut app, 3);
    goto(&mut app, "John 8:12");
    goto(&mut app, "John 8");
    assert_at(&mut app, 80, 1, "John 8");
}

#[test]
fn placeholder_tracks_logical_verse_after_returning_to_top() {
    use bible_core::components::reading_pane::ReadingPane;

    let bible = loader::load_bundled(BundledTranslation::Kjv);
    let chapter = &bible.books[42].chapters[7];
    let mut pane = ReadingPane::new();
    pane.set_chapter(42, "John", chapter, 80);
    pane.scroll_to_verse(12);
    for width in [3, 2, 3] {
        pane.rebuild_lines(chapter, width);
        assert_eq!(pane.current_verse_approx(), 12);
    }
    pane.scroll_to_top();
    assert_eq!(pane.current_verse_approx(), 1);
    pane.scroll_to_verse(12);
    assert_eq!(pane.current_verse_approx(), 12);
    pane.rebuild_lines(chapter, 80);
    assert_eq!(pane.current_verse_approx(), 12);
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

#[test]
fn bottom_key_at_placeholder_width_restores_bottom_when_widened() {
    let mut expected = app();
    render(&mut expected, 80);
    goto(&mut expected, "John 8:12");
    key(&mut expected, KeyCode::Char('G'));
    let expected_rows = render(&mut expected, 80);
    assert!(expected_rows.iter().any(|row| row.contains("⁵⁹ Then took")));

    let mut actual = app();
    render(&mut actual, 80);
    goto(&mut actual, "John 8:12");
    actual.update(&Action::Resize(3, 12));
    render(&mut actual, 3);
    key(&mut actual, KeyCode::Char('G'));
    actual.update(&Action::Resize(80, 12));
    assert_eq!(render(&mut actual, 80), expected_rows);
}

use bible_core::components::reading_pane::ReadingPane;

fn pane_rows(pane: &mut ReadingPane, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| pane.render(frame, frame.area())).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn assert_deferred_navigation(actions: &[fn(&mut ReadingPane)]) {
    let bible = loader::load_bundled(BundledTranslation::Kjv);
    let chapter = &bible.books[42].chapters[7];
    let mut expected = ReadingPane::new();
    expected.set_chapter(42, "John", chapter, 44);
    pane_rows(&mut expected, 44, 16);
    expected.scroll_to_verse(12);
    let unchanged_rows = pane_rows(&mut expected, 44, 16);
    for action in actions {
        action(&mut expected);
    }
    let expected_rows = pane_rows(&mut expected, 44, 16);
    assert_ne!(expected_rows, unchanged_rows, "action must move the viewport");

    let mut actual = ReadingPane::new();
    actual.set_chapter(42, "John", chapter, 80);
    pane_rows(&mut actual, 80, 12);
    actual.scroll_to_verse(12);
    actual.rebuild_lines(chapter, 3);
    pane_rows(&mut actual, 3, 7);
    for action in actions {
        action(&mut actual);
        // Repeated narrow rebuilds and renders must neither lose nor consume intent.
        actual.rebuild_lines(chapter, 2);
        pane_rows(&mut actual, 2, 7);
    }
    actual.rebuild_lines(chapter, 44);
    assert_eq!(pane_rows(&mut actual, 44, 16), expected_rows);
    assert_eq!(actual.current_verse_approx(), expected.current_verse_approx());
    assert_eq!(pane_rows(&mut actual, 44, 16), expected_rows, "replay only once");
}

#[test]
fn deferred_scroll_down_survives_widening() {
    assert_deferred_navigation(&[|pane| pane.scroll_down(2)]);
}

#[test]
fn deferred_scroll_up_survives_widening() {
    assert_deferred_navigation(&[|pane| pane.scroll_up(2)]);
}

#[test]
fn deferred_page_down_uses_restored_viewport() {
    assert_deferred_navigation(&[ReadingPane::page_down]);
}

#[test]
fn deferred_page_up_uses_restored_viewport() {
    assert_deferred_navigation(&[ReadingPane::page_up]);
}

#[test]
fn deferred_top_survives_widening() {
    assert_deferred_navigation(&[ReadingPane::scroll_to_top]);
}

#[test]
fn deferred_bottom_uses_rebuilt_chapter_and_restored_viewport() {
    assert_deferred_navigation(&[ReadingPane::scroll_to_bottom]);
}

#[test]
fn deferred_verse_jump_survives_widening() {
    assert_deferred_navigation(&[|pane| pane.scroll_to_verse(20)]);
}

#[test]
fn deferred_navigation_preserves_action_order() {
    assert_deferred_navigation(&[
        ReadingPane::scroll_to_bottom,
        ReadingPane::page_up,
        |pane| pane.scroll_up(2),
        |pane| pane.scroll_down(1),
    ]);
    assert_deferred_navigation(&[
        ReadingPane::scroll_to_top,
        ReadingPane::page_down,
    ]);
}

#[test]
fn deferred_absolute_jumps_replace_earlier_navigation() {
    assert_deferred_navigation(&[
        ReadingPane::scroll_to_bottom,
        |pane| pane.scroll_to_verse(20),
        |pane| pane.scroll_down(1),
    ]);
    assert_deferred_navigation(&[
        ReadingPane::page_down,
        ReadingPane::scroll_to_top,
    ]);
}

#[test]
fn deferred_line_navigation_keeps_scroll_bounds() {
    assert_deferred_navigation(&[|pane| pane.scroll_up(u16::MAX)]);
    assert_deferred_navigation(&[|pane| pane.scroll_down(u16::MAX)]);
    assert_deferred_navigation(&[
        |pane| pane.scroll_to_verse(59),
        |pane| pane.scroll_down(1),
    ]);
}

#[test]
fn navigation_between_rebuild_and_render_keeps_action_order() {
    let bible = loader::load_bundled(BundledTranslation::Kjv);
    let chapter = &bible.books[42].chapters[7];
    let mut expected = ReadingPane::new();
    expected.set_chapter(42, "John", chapter, 44);
    pane_rows(&mut expected, 44, 16);
    expected.scroll_to_bottom();
    expected.page_up();
    let expected_rows = pane_rows(&mut expected, 44, 16);

    let mut actual = ReadingPane::new();
    actual.set_chapter(42, "John", chapter, 3);
    actual.scroll_to_verse(12);
    actual.scroll_to_bottom();
    actual.rebuild_lines(chapter, 44);
    actual.page_up();
    assert_eq!(pane_rows(&mut actual, 44, 16), expected_rows);
}

#[test]
fn chapter_change_discards_deferred_navigation() {
    let bible = loader::load_bundled(BundledTranslation::Kjv);
    let chapter = &bible.books[42].chapters[7];
    let mut pane = ReadingPane::new();
    pane.set_chapter(42, "John", chapter, 80);
    pane_rows(&mut pane, 80, 12);
    pane.scroll_to_verse(12);
    pane.rebuild_lines(chapter, 3);
    pane.scroll_to_bottom();
    pane.page_up();
    pane.set_chapter(42, "John", chapter, 3);
    pane.rebuild_lines(chapter, 80);
    assert!(pane_rows(&mut pane, 80, 12)[0].starts_with("John 8"));
    assert_eq!(pane.current_verse_approx(), 1);
}
