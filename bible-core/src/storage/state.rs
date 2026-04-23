use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppState {
    pub last_book: u8,
    pub last_chapter: u16,
    pub last_translation: Option<String>,
}
