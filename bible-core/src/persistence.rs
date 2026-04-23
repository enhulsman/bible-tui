use crate::bible::model::{BibleData, TranslationId, TranslationInfo};
use crate::storage::bookmark::BookmarkStore;
use crate::storage::state::AppState;

pub trait Persistence {
    fn load_bookmarks(&self) -> BookmarkStore;
    fn save_bookmarks(&self, store: &BookmarkStore);
    fn load_state(&self) -> AppState;
    fn save_state(&self, state: &AppState);
}

pub trait TranslationProvider {
    fn available_translations(&self) -> Vec<TranslationInfo>;
    fn load_translation(&self, id: &TranslationId) -> Option<BibleData>;
}

/// No-op persistence used for WASM MVP (bookmarks/state deferred to Phase 6).
pub struct NoPersistence;

impl Persistence for NoPersistence {
    fn load_bookmarks(&self) -> BookmarkStore { BookmarkStore::default() }
    fn save_bookmarks(&self, _store: &BookmarkStore) {}
    fn load_state(&self) -> AppState { AppState::default() }
    fn save_state(&self, _state: &AppState) {}
}
