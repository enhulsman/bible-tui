use bible_core::persistence::Persistence;
use bible_core::storage::bookmark::BookmarkStore;
use bible_core::storage::state::AppState;

const BOOKMARKS_KEY: &str = "bible-tui.bookmarks.v1";

pub struct BrowserPersistence;

impl Persistence for BrowserPersistence {
    fn load_bookmarks(&self) -> BookmarkStore {
        let contents =
            local_storage().and_then(|storage| storage.get_item(BOOKMARKS_KEY).ok().flatten());
        parse_bookmarks(contents.as_deref())
    }

    fn save_bookmarks(&self, store: &BookmarkStore) {
        if let (Some(storage), Ok(contents)) = (local_storage(), serialize_bookmarks(store)) {
            let _ = storage.set_item(BOOKMARKS_KEY, &contents);
        }
    }

    fn load_state(&self) -> AppState {
        AppState::default()
    }

    fn save_state(&self, _state: &AppState) {}
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn parse_bookmarks(contents: Option<&str>) -> BookmarkStore {
    contents
        .and_then(|contents| serde_json::from_str(contents).ok())
        .unwrap_or_default()
}

fn serialize_bookmarks(store: &BookmarkStore) -> Result<String, serde_json::Error> {
    serde_json::to_string(store)
}

#[cfg(test)]
mod tests {
    use super::{parse_bookmarks, serialize_bookmarks};
    use bible_core::bible::model::VerseRef;
    use bible_core::storage::bookmark::{Bookmark, BookmarkStore};

    #[test]
    fn bookmarks_round_trip_preserves_references_notes_and_timestamps() {
        let mut first = Bookmark::from_ref(VerseRef {
            book_index: 42,
            chapter: 8,
            verse: 12,
        });
        first.note = Some("Light: \"licht\"\nLumière, φῶς".to_owned());
        let second = Bookmark::from_ref(VerseRef {
            book_index: 0,
            chapter: 1,
            verse: 1,
        });
        let store = BookmarkStore {
            bookmarks: vec![first, second],
        };

        let serialized = serialize_bookmarks(&store).expect("serialize bookmarks");
        let loaded = parse_bookmarks(Some(&serialized));

        assert_eq!(loaded.bookmarks.len(), store.bookmarks.len());
        for (actual, expected) in loaded.bookmarks.iter().zip(&store.bookmarks) {
            assert_eq!(actual.verse_ref(), expected.verse_ref());
            assert_eq!(actual.note, expected.note);
            assert_eq!(actual.created_at, expected.created_at);
        }
    }

    #[test]
    fn empty_store_round_trips_after_removing_all_bookmarks() {
        let serialized = serialize_bookmarks(&BookmarkStore::default())
            .expect("serialize an empty bookmark store");
        assert!(!serialized.is_empty());
        assert!(parse_bookmarks(Some(&serialized)).bookmarks.is_empty());
    }

    #[test]
    fn missing_or_empty_input_loads_an_empty_store() {
        for input in [None, Some(""), Some(" \n\t ")] {
            assert!(parse_bookmarks(input).bookmarks.is_empty(), "{input:?}");
        }
    }

    #[test]
    fn corrupt_or_wrong_schema_input_loads_an_empty_store() {
        for input in [
            "not JSON",
            "{",
            "null",
            "{}",
            r#"{"bookmarks":"invalid"}"#,
            r#"{"bookmarks":[{"book_index":42}]}"#,
        ] {
            assert!(parse_bookmarks(Some(input)).bookmarks.is_empty(), "{input}");
        }
    }
}
