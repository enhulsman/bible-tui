mod event;
mod import;
mod keys;
mod storage;
mod translation;
mod tui;

use std::path::Path;
use std::time::Duration;

use bible_core::app::App;
use event::{Event, EventHandler};
use storage::NativeStorage;
use translation::NativeTranslationProvider;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "import" {
        return run_import(&args[2]);
    }

    let storage = NativeStorage::new()?;
    let library_path = storage.library_path();

    // Ensure library DB is initialized
    if let Ok(db) = rusqlite::Connection::open(&library_path) {
        let _ = import::init_library(&db);
    }

    let translation_provider = NativeTranslationProvider::new(library_path);

    let mut terminal = tui::init()?;
    let events = EventHandler::new(Duration::from_millis(250));
    let mut app = App::new(Box::new(storage), Box::new(translation_provider));

    while app.running {
        terminal.draw(|frame| app.render(frame))?;

        let event = events.next()?;
        match event {
            Event::Key(key) => {
                let action = app.handle_key(key);
                app.update(&action);
            }
            Event::Resize(w, h) => {
                app.update(&bible_core::action::Action::Resize(w, h));
            }
            Event::Tick => {
                app.update(&bible_core::action::Action::Tick);
            }
        }
    }

    tui::restore()?;
    Ok(())
}

fn run_import(file_path: &str) -> color_eyre::Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("File not found: {file_path}");
        std::process::exit(1);
    }

    let storage = NativeStorage::new()?;
    let db = rusqlite::Connection::open(storage.library_path())?;
    import::init_library(&db)?;

    match import::import_file(path, &db) {
        Ok(id) => {
            println!("Imported translation: {id}");
            println!("Library: {}", storage.library_path().display());
        }
        Err(e) => {
            eprintln!("Import failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
