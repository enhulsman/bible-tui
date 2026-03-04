use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppState {
    pub last_book: u8,
    pub last_chapter: u16,
    pub last_translation: Option<String>,
}

impl AppState {
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
}
