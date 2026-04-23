use bible_core::persistence::Persistence;
use bible_core::storage::bookmark::BookmarkStore;
use bible_core::storage::state::AppState;
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

pub struct NativeStorage {
    data_dir: PathBuf,
    #[allow(dead_code)]
    config_dir: PathBuf,
}

impl NativeStorage {
    pub fn new() -> color_eyre::Result<Self> {
        let dirs = ProjectDirs::from("", "", "bible-tui")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine home directory"))?;

        let config_dir = dirs.config_dir().to_path_buf();
        let data_dir = dirs.data_dir().to_path_buf();

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(data_dir.join("translations"))?;

        Ok(Self { data_dir, config_dir })
    }

    pub fn bookmarks_path(&self) -> PathBuf {
        self.data_dir.join("bookmarks.toml")
    }

    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join("state.toml")
    }

    pub fn library_path(&self) -> PathBuf {
        self.data_dir.join("library.sqlite")
    }
}

impl Persistence for NativeStorage {
    fn load_bookmarks(&self) -> BookmarkStore {
        load_toml(&self.bookmarks_path())
    }

    fn save_bookmarks(&self, store: &BookmarkStore) {
        let _ = save_toml(&self.bookmarks_path(), store);
    }

    fn load_state(&self) -> AppState {
        load_toml(&self.state_path())
    }

    fn save_state(&self, state: &AppState) {
        let _ = save_toml(&self.state_path(), state);
    }
}

fn load_toml<T: Default + serde::de::DeserializeOwned>(path: &Path) -> T {
    if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        T::default()
    }
}

fn save_toml<T: serde::Serialize>(path: &Path, value: &T) -> color_eyre::Result<()> {
    let contents = toml::to_string_pretty(value)?;
    std::fs::write(path, contents)?;
    Ok(())
}
