pub mod bookmark;
pub mod state;

use directories::ProjectDirs;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct Storage {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> color_eyre::Result<Self> {
        let dirs = ProjectDirs::from("", "", "bible-tui")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine home directory"))?;

        let config_dir = dirs.config_dir().to_path_buf();
        let data_dir = dirs.data_dir().to_path_buf();

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;

        // Create translations directory for user imports
        let translations_dir = data_dir.join("translations");
        std::fs::create_dir_all(&translations_dir)?;

        Ok(Self {
            config_dir,
            data_dir,
        })
    }

    #[allow(dead_code)]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    #[allow(dead_code)]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn bookmarks_path(&self) -> PathBuf {
        self.data_dir.join("bookmarks.toml")
    }

    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join("state.toml")
    }

    #[allow(dead_code)]
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn library_path(&self) -> PathBuf {
        self.data_dir.join("library.sqlite")
    }

    #[allow(dead_code)]
    pub fn translations_dir(&self) -> PathBuf {
        self.data_dir.join("translations")
    }
}
