use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub fn toggle(&mut self, vref: VerseRef) -> bool {
        if let Some(pos) = self.bookmarks.iter().position(|b| b.verse_ref() == vref) {
            self.bookmarks.remove(pos);
            false
        } else {
            self.bookmarks.push(Bookmark::from_ref(vref));
            true
        }
    }

    #[allow(dead_code)]
    pub fn is_bookmarked(&self, vref: &VerseRef) -> bool {
        self.bookmarks.iter().any(|b| b.verse_ref() == *vref)
    }
}
