mod action;
mod app;
mod bible;
mod components;
mod event;
mod import;
mod search;
mod storage;
mod tui;
mod ui;

use app::App;
use event::EventHandler;
use std::path::Path;
use std::time::Duration;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();

    // CLI subcommand: bible import <file>
    if args.len() >= 3 && args[1] == "import" {
        return run_import(&args[2]);
    }

    let mut terminal = tui::init()?;
    let events = EventHandler::new(Duration::from_millis(250));
    let mut app = App::new();

    while app.running {
        terminal.draw(|frame| app.render(frame))?;

        let event = events.next()?;
        let action = app.handle_event(event);
        app.update(&action);
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

    let storage = storage::Storage::new()?;
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
