use bible_core::bible::loader;
use bible_core::bible::model::{BibleData, BundledTranslation, TranslationId, TranslationInfo};
use bible_core::persistence::TranslationProvider;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct NativeTranslationProvider {
    library_path: PathBuf,
}

impl NativeTranslationProvider {
    pub fn new(library_path: PathBuf) -> Self {
        Self { library_path }
    }

    fn open_db(&self) -> Option<Connection> {
        Connection::open(&self.library_path).ok()
    }
}

impl TranslationProvider for NativeTranslationProvider {
    fn available_translations(&self) -> Vec<TranslationInfo> {
        let bundled = [BundledTranslation::Kjv, BundledTranslation::Web, BundledTranslation::Sv];
        let mut translations: Vec<TranslationInfo> = bundled.iter().map(|b| b.info()).collect();

        if let Some(db) = self.open_db() {
            translations.extend(crate::import::list_translations(&db));
        }

        translations
    }

    fn load_translation(&self, id: &TranslationId) -> Option<BibleData> {
        match id {
            TranslationId::Bundled(bt) => Some(loader::load_bundled(*bt)),
            TranslationId::Imported(name) => {
                let db = self.open_db()?;
                crate::import::load_full_bible(&db, name)
            }
        }
    }
}
