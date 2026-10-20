mod keys;
mod persistence;

use std::cell::RefCell;
use std::rc::Rc;

use bible_core::app::App;
use bible_core::bible::loader;
use bible_core::bible::model::{BundledTranslation, TranslationId, TranslationInfo};
use bible_core::persistence::TranslationProvider;
use bible_core::bible::model::BibleData;
use ratzilla::DomBackend;
use ratzilla::ratatui::Terminal;
use ratzilla::WebRenderer;
use wasm_bindgen::prelude::*;

struct WasmTranslationProvider;

fn handle_event(app: &mut App, event: bible_core::keys::KeyEvent) {
    let bookmark_revision = app.bookmark_revision();
    let action = app.handle_key(event);
    app.update(&action);
    if app.bookmark_revision() != bookmark_revision {
        app.save_state();
    }
}

impl TranslationProvider for WasmTranslationProvider {
    fn available_translations(&self) -> Vec<TranslationInfo> {
        vec![
            BundledTranslation::Kjv.info(),
            BundledTranslation::Web.info(),
            BundledTranslation::Sv.info(),
        ]
    }

    fn load_translation(&self, id: &TranslationId) -> Option<BibleData> {
        match id {
            TranslationId::Bundled(bt) => Some(loader::load_bundled(*bt)),
            TranslationId::Imported(_) => None,
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let backend = DomBackend::new_by_id("terminal-body")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut terminal = Terminal::new(backend)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let events: Rc<RefCell<Vec<bible_core::keys::KeyEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let app = Rc::new(RefCell::new(App::new(
        Box::new(persistence::BrowserPersistence),
        Box::new(WasmTranslationProvider),
    )));

    terminal.on_key_event({
        let events = events.clone();
        move |key_event| {
            events.borrow_mut().push(keys::from_ratzilla(key_event));
        }
    }).map_err(|e| JsValue::from_str(&e.to_string()))?;

    terminal.draw_web(move |frame| {
        let pending: Vec<_> = events.borrow_mut().drain(..).collect();
        let mut app = app.borrow_mut();

        let current_width = frame.area().width;
        if current_width != app.last_width {
            app.update(&bible_core::action::Action::Resize(
                current_width,
                frame.area().height,
            ));
        }

        for event in pending {
            handle_event(&mut app, event);
        }

        app.render(frame);
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{handle_event, App, WasmTranslationProvider};
    use bible_core::keys::{KeyCode, KeyEvent};
    use bible_core::persistence::Persistence;
    use bible_core::storage::bookmark::{Bookmark, BookmarkStore};
    use bible_core::storage::state::AppState;
    use std::cell::RefCell;
    use std::rc::Rc;

    type SavedBookmarks = Rc<RefCell<Vec<Vec<Bookmark>>>>;

    struct RecordingPersistence(SavedBookmarks);

    impl Persistence for RecordingPersistence {
        fn load_bookmarks(&self) -> BookmarkStore {
            BookmarkStore {
                bookmarks: self.0.borrow().last().cloned().unwrap_or_default(),
            }
        }

        fn save_bookmarks(&self, store: &BookmarkStore) {
            self.0.borrow_mut().push(store.bookmarks.clone());
        }

        fn load_state(&self) -> AppState {
            AppState::default()
        }

        fn save_state(&self, _: &AppState) {}
    }

    fn app(saved: &SavedBookmarks) -> App {
        App::new(
            Box::new(RecordingPersistence(Rc::clone(saved))),
            Box::new(WasmTranslationProvider),
        )
    }

    fn key(app: &mut App, code: KeyCode) {
        handle_event(app, KeyEvent::plain(code));
    }

    #[test]
    fn web_events_save_each_bookmark_addition_and_removal_before_quit() {
        let saved = SavedBookmarks::default();
        let mut app = app(&saved);
        assert!(saved.borrow().is_empty());
        let initial_revision = app.bookmark_revision();

        key(&mut app, KeyCode::Char('B'));
        assert_ne!(app.bookmark_revision(), initial_revision);
        assert_eq!(saved.borrow().len(), 1, "addition must save immediately");
        assert_eq!(saved.borrow()[0].len(), 1);
        let added_revision = app.bookmark_revision();

        key(&mut app, KeyCode::Char('B'));
        assert_ne!(app.bookmark_revision(), added_revision);
        assert_eq!(saved.borrow().len(), 2, "removal must save immediately");
        assert!(saved.borrow()[1].is_empty());
        assert!(app.running);
    }

    #[test]
    fn web_events_save_removal_of_a_reloaded_bookmark() {
        let saved = SavedBookmarks::default();
        let mut original = app(&saved);
        key(&mut original, KeyCode::Char('B'));
        assert_eq!(saved.borrow().len(), 1);

        let mut reloaded = app(&saved);
        key(&mut reloaded, KeyCode::Char('B'));
        assert_eq!(saved.borrow().len(), 2);
        assert!(saved.borrow()[1].is_empty());

        let mut reloaded_again = app(&saved);
        key(&mut reloaded_again, KeyCode::Char('B'));
        assert_eq!(saved.borrow().len(), 3);
        assert_eq!(saved.borrow()[2].len(), 1);
    }

    #[test]
    fn web_navigation_and_bookmark_commands_do_not_save_unchanged_bookmarks() {
        let saved = SavedBookmarks::default();
        let mut app = app(&saved);
        key(&mut app, KeyCode::Char('j'));
        for command in ["bm", "bookmarks"] {
            key(&mut app, KeyCode::Char(':'));
            for c in command.chars() {
                key(&mut app, KeyCode::Char(c));
            }
            key(&mut app, KeyCode::Enter);
        }
        assert!(saved.borrow().is_empty());
    }

    #[test]
    fn core_updates_mark_bookmark_changes_without_saving_automatically() {
        use bible_core::action::Action;

        let saved = SavedBookmarks::default();
        let mut app = app(&saved);
        let initial_revision = app.bookmark_revision();
        app.update(&Action::BookmarkCurrent);
        let added_revision = app.bookmark_revision();
        assert_ne!(added_revision, initial_revision);
        app.update(&Action::None);
        assert_eq!(app.bookmark_revision(), added_revision);
        app.update(&Action::BookmarkCurrent);
        assert_ne!(app.bookmark_revision(), added_revision);
        assert!(saved.borrow().is_empty(), "native updates must not autosave");
    }
}
