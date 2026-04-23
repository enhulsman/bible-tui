use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::bible::model::VerseRef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub book_index: u8,
    pub chapter: u16,
    pub verse: u8,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn from_ref(vref: VerseRef) -> Self {
        Self {
            book_index: vref.book_index,
            chapter: vref.chapter,
            verse: vref.verse,
            note: None,
            created_at: Utc::now(),
        }
    }

    pub fn verse_ref(&self) -> VerseRef {
        VerseRef {
            book_index: self.book_index,
            chapter: self.chapter,
            verse: self.verse,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BookmarkStore {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkStore {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            let contents = std::fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> color_eyre::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn toggle(&mut self, vref: VerseRef) -> bool {
        if let Some(pos) = self
            .bookmarks
            .iter()
            .position(|b| b.verse_ref() == vref)
        {
            self.bookmarks.remove(pos);
            false // removed
        } else {
            self.bookmarks.push(Bookmark::from_ref(vref));
            true // added
        }
    }

    #[allow(dead_code)]
    pub fn is_bookmarked(&self, vref: &VerseRef) -> bool {
        self.bookmarks.iter().any(|b| b.verse_ref() == *vref)
    }
}
