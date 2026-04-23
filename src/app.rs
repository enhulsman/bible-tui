use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

use crate::action::Action;
use crate::bible::loader;
use crate::bible::model::{BibleData, BundledTranslation, TranslationId, TranslationInfo, VerseRef};
use crate::bible::reference;
use crate::components::command_palette::CommandPalette;
use crate::components::help_overlay::HelpOverlay;
use crate::components::nav_panel::NavPanel;
use crate::components::reading_pane::ReadingPane;
use crate::components::search_bar::SearchBar;
use crate::components::status_bar::StatusBar;
use crate::event::Event;
use crate::import;
use crate::search::index::SearchIndex;
use crate::storage::bookmark::BookmarkStore;
use crate::storage::state::AppState;
use crate::storage::Storage;
use crate::ui::layout;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Navigation,
    Search,
    Command,
}

pub struct App {
    pub running: bool,
    pub mode: Mode,
    bible: BibleData,
    current_translation: TranslationInfo,
    available_translations: Vec<TranslationInfo>,
    library_db: Option<rusqlite::Connection>,
    reading_pane: ReadingPane,
    nav_panel: NavPanel,
    search_bar: SearchBar,
    search_index: SearchIndex,
    command_palette: CommandPalette,
    show_nav: bool,
    show_help: bool,
    last_width: u16,
    storage: Option<Storage>,
    bookmarks: BookmarkStore,
    error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let storage = Storage::new().ok();
        let bookmarks = storage
            .as_ref()
            .map(|s| BookmarkStore::load(&s.bookmarks_path()))
            .unwrap_or_default();
        let app_state = storage
            .as_ref()
            .map(|s| AppState::load(&s.state_path()))
            .unwrap_or_default();

        // Open library database (non-fatal)
        let library_db: Option<rusqlite::Connection> = storage
            .as_ref()
            .and_then(|s| rusqlite::Connection::open(s.library_path()).ok());
        if let Some(ref db) = library_db {
            let _ = import::init_library(db);
        }

        // Build available translations: bundled + imported
        let bundled = [BundledTranslation::Kjv, BundledTranslation::Web, BundledTranslation::Sv];
        let mut available_translations: Vec<TranslationInfo> =
            bundled.iter().map(|b| b.info()).collect();
        if let Some(ref db) = library_db {
            available_translations.extend(import::list_translations(db));
        }

        // Restore saved translation or default to KJV
        let (bible, current_translation) = if let Some(ref saved_id) = app_state.last_translation {
            // Try bundled first
            if let Some(bt) = bundled
                .iter()
                .find(|b| b.abbreviation().eq_ignore_ascii_case(saved_id))
            {
                let bible = loader::load_bundled(*bt);
                (bible, bt.info())
            } else if let Some(ref db) = library_db {
                // Try imported
                if let Some(bible) = import::load_full_bible(db, saved_id) {
                    let info = available_translations
                        .iter()
                        .find(|t| t.id == TranslationId::Imported(saved_id.clone()))
                        .cloned()
                        .unwrap_or_else(|| TranslationInfo {
                            id: TranslationId::Imported(saved_id.clone()),
                            name: saved_id.clone(),
                            abbreviation: saved_id.to_uppercase(),
                        });
                    (bible, info)
                } else {
                    let bible = loader::load_bundled(BundledTranslation::Kjv);
                    (bible, BundledTranslation::Kjv.info())
                }
            } else {
                let bible = loader::load_bundled(BundledTranslation::Kjv);
                (bible, BundledTranslation::Kjv.info())
            }
        } else {
            let bible = loader::load_bundled(BundledTranslation::Kjv);
            (bible, BundledTranslation::Kjv.info())
        };

        let search_index = SearchIndex::build(&bible);
        let mut reading_pane = ReadingPane::new();

        let start_book = app_state.last_book;
        let start_chapter = app_state.last_chapter.max(1);

        if let Some(book) = bible.books.get(start_book as usize) {
            let name = book.name.clone();
            let ch_idx = (start_chapter as usize).saturating_sub(1);
            if let Some(chapter) = book.chapters.get(ch_idx) {
                reading_pane.set_chapter(start_book, &name, chapter, 80);
            } else if let Some(chapter) = book.chapters.first() {
                reading_pane.set_chapter(start_book, &name, chapter, 80);
            }
        } else if let Some(book) = bible.books.first() {
            let name = book.name.clone();
            if let Some(chapter) = book.chapters.first() {
                reading_pane.set_chapter(0, &name, chapter, 80);
            }
        }

        let mut command_palette = CommandPalette::new();
        command_palette.set_translations(
            available_translations
                .iter()
                .map(|t| t.abbreviation.to_lowercase())
                .collect(),
        );

        Self {
            running: true,
            mode: Mode::Normal,
            bible,
            current_translation,
            available_translations,
            library_db,
            reading_pane,
            nav_panel: NavPanel::new(),
            search_bar: SearchBar::new(),
            search_index,
            command_palette,
            show_nav: false,
            show_help: false,
            last_width: 80,
            storage,
            bookmarks,
            error_message: None,
        }
    }

    pub fn save_state(&self) {
        if let Some(storage) = &self.storage {
            let state = AppState {
                last_book: self.reading_pane.book_index(),
                last_chapter: self.reading_pane.chapter_num(),
                last_translation: Some(self.current_translation.id.to_string()),
            };
            let _ = state.save(&storage.state_path());
            let _ = self.bookmarks.save(&storage.bookmarks_path());
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Action {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize(w, h) => Action::Resize(w, h),
            Event::Tick => Action::Tick,
            Event::Mouse(_) => Action::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }

        // Help overlay captures Esc and ?
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                    return Action::ToggleHelp;
                }
                _ => return Action::None,
            }
        }

        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown(1),
                KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp(1),
                KeyCode::Char('f') | KeyCode::PageDown => Action::PageDown,
                KeyCode::Char('b') | KeyCode::PageUp => Action::PageUp,
                KeyCode::Char('G') => Action::ScrollToBottom,
                KeyCode::Char('g') => Action::ScrollToTop,
                KeyCode::Char(' ') => Action::NextChapter,
                KeyCode::Backspace => Action::PrevChapter,
                KeyCode::Tab => Action::ToggleNavPanel,
                KeyCode::Char('/') => Action::EnterSearchMode,
                KeyCode::Char(':') => Action::EnterCommandMode,
                KeyCode::Char('B') => Action::BookmarkCurrent,
                KeyCode::Char('?') => Action::ToggleHelp,
                KeyCode::Char('n') => Action::SearchNext,
                KeyCode::Char('N') => Action::SearchPrev,
                _ => Action::None,
            },
            Mode::Navigation => self.nav_panel.handle_key(key),
            Mode::Search => self.search_bar.handle_key(key, &self.search_index),
            Mode::Command => self.command_palette.handle_key(key),
        }
    }

    pub fn update(&mut self, action: &Action) {
        // Clear error on meaningful user actions
        match action {
            Action::Tick | Action::None | Action::Resize(_, _) | Action::ExecuteCommand(_) => {}
            _ => {
                self.error_message = None;
            }
        }
        match action {
            Action::Quit => {
                self.save_state();
                self.running = false;
            }
            Action::ScrollDown(n) => self.reading_pane.scroll_down(*n),
            Action::ScrollUp(n) => self.reading_pane.scroll_up(*n),
            Action::ScrollToTop => self.reading_pane.scroll_to_top(),
            Action::ScrollToBottom => self.reading_pane.scroll_to_bottom(),
            Action::PageDown => self.reading_pane.page_down(),
            Action::PageUp => self.reading_pane.page_up(),
            Action::NextChapter => self.navigate_chapter(1),
            Action::PrevChapter => self.navigate_chapter(-1),
            Action::ToggleNavPanel => {
                if self.mode == Mode::Navigation {
                    self.mode = Mode::Normal;
                    self.show_nav = false;
                } else {
                    self.mode = Mode::Navigation;
                    self.show_nav = true;
                    self.nav_panel.sync_to(
                        self.reading_pane.book_index(),
                        self.reading_pane.chapter_num(),
                    );
                }
                self.rewrap_current_chapter();
            }
            Action::NavigateToRef { book, chapter } => {
                self.navigate_to(*book, *chapter);
                self.mode = Mode::Normal;
                self.show_nav = false;
                self.rewrap_current_chapter();
            }
            Action::EnterSearchMode => {
                self.mode = Mode::Search;
                self.search_bar.open();
            }
            Action::ExitSearchMode => {
                self.mode = Mode::Normal;
            }
            Action::SearchNext => {
                if let Some(action) = self.search_bar.next_result() {
                    self.update(&action);
                }
            }
            Action::SearchPrev => {
                if let Some(action) = self.search_bar.prev_result() {
                    self.update(&action);
                }
            }
            Action::EnterCommandMode => {
                self.mode = Mode::Command;
                self.command_palette.open();
            }
            Action::ExitCommandMode => {
                self.mode = Mode::Normal;
            }
            Action::ExecuteCommand(cmd) => {
                self.execute_command(cmd);
                self.mode = Mode::Normal;
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::BookmarkCurrent => {
                let vref = VerseRef {
                    book_index: self.reading_pane.book_index(),
                    chapter: self.reading_pane.chapter_num(),
                    verse: self.reading_pane.current_verse_approx(),
                };
                self.bookmarks.toggle(vref);
            }
            Action::Resize(w, _) => {
                self.last_width = *w;
                self.rewrap_current_chapter();
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        match parts[0] {
            "q" | "quit" => {
                self.save_state();
                self.running = false;
            }
            "t" | "translation" => {
                if let Some(name) = parts.get(1) {
                    self.switch_translation(name.trim());
                }
            }
            "goto" | "g" => {
                if let Some(ref_str) = parts.get(1) {
                    let context_book = Some(self.reading_pane.book_index());
                    let context_chapter = Some(self.reading_pane.chapter_num());
                    if let Some(vref) =
                        reference::parse_reference(ref_str.trim(), context_book, context_chapter)
                    {
                        self.navigate_to(vref.book_index, vref.chapter);
                        self.error_message = None;
                    } else {
                        self.error_message = Some(format!("Unknown reference: {}", ref_str.trim()));
                    }
                }
            }
            _ => {} // Unknown command, silently ignore
        }
    }

    fn switch_translation(&mut self, name: &str) {
        let name_lower = name.to_lowercase();

        // Find matching translation in available list
        let info = self
            .available_translations
            .iter()
            .find(|t| t.abbreviation.to_lowercase() == name_lower)
            .cloned();

        let Some(info) = info else {
            self.error_message = Some(format!("Unknown translation: {name}"));
            return;
        };

        if info.id == self.current_translation.id {
            return;
        }

        let book_idx = self.reading_pane.book_index();
        let ch_num = self.reading_pane.chapter_num();

        match &info.id {
            TranslationId::Bundled(bt) => {
                self.bible = loader::load_bundled(*bt);
            }
            TranslationId::Imported(id) => {
                if let Some(ref db) = self.library_db {
                    if let Some(bible) = import::load_full_bible(db, id) {
                        self.bible = bible;
                    } else {
                        self.error_message = Some(format!("Failed to load: {name}"));
                        return;
                    }
                } else {
                    self.error_message = Some("No library database".to_string());
                    return;
                }
            }
        }

        self.current_translation = info;
        self.search_index = SearchIndex::build(&self.bible);
        self.navigate_to(book_idx, ch_num);
    }

    fn navigate_to(&mut self, book_index: u8, chapter: u16) {
        if let Some(book) = self.bible.books.get(book_index as usize) {
            let name = book.name.clone();
            let ch_idx = (chapter as usize).saturating_sub(1);
            if let Some(ch) = book.chapters.get(ch_idx) {
                self.reading_pane
                    .set_chapter(book_index, &name, ch, self.content_width());
            }
        }
    }

    fn navigate_chapter(&mut self, delta: i32) {
        let book_idx = self.reading_pane.book_index() as usize;
        let ch_num = self.reading_pane.chapter_num();

        if let Some(book) = self.bible.books.get(book_idx) {
            let ch_idx = ch_num as i32 - 1 + delta;

            if ch_idx >= 0 && (ch_idx as usize) < book.chapters.len() {
                let name = book.name.clone();
                let chapter = &book.chapters[ch_idx as usize];
                self.reading_pane
                    .set_chapter(book_idx as u8, &name, chapter, self.content_width());
            } else if delta > 0 && book_idx + 1 < self.bible.books.len() {
                let next_book = &self.bible.books[book_idx + 1];
                let name = next_book.name.clone();
                if let Some(chapter) = next_book.chapters.first() {
                    self.reading_pane
                        .set_chapter((book_idx + 1) as u8, &name, chapter, self.content_width());
                }
            } else if delta < 0 && book_idx > 0 {
                let prev_book = &self.bible.books[book_idx - 1];
                let name = prev_book.name.clone();
                if let Some(chapter) = prev_book.chapters.last() {
                    self.reading_pane
                        .set_chapter((book_idx - 1) as u8, &name, chapter, self.content_width());
                }
            }
        }
    }

    fn rewrap_current_chapter(&mut self) {
        let book_idx = self.reading_pane.book_index() as usize;
        let ch_num = self.reading_pane.chapter_num();

        if let Some(book) = self.bible.books.get(book_idx) {
            if let Some(chapter) = book.chapters.iter().find(|c| c.number == ch_num) {
                self.reading_pane
                    .rebuild_lines(chapter, self.content_width());
            }
        }
    }

    fn content_width(&self) -> u16 {
        if self.show_nav && self.last_width >= layout::NAV_PANEL_MIN_WIDTH {
            self.last_width - layout::NAV_PANEL_WIDTH
        } else {
            self.last_width
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if area.width != self.last_width {
            self.last_width = area.width;
            self.rewrap_current_chapter();
        }

        let app_layout = layout::compute_layout(area, self.show_nav);

        if let Some(nav_area) = app_layout.nav_panel {
            self.nav_panel.render(frame, nav_area, &self.bible.books);
        }

        self.reading_pane.render(frame, app_layout.reading_pane);

        let translation = &self.current_translation.abbreviation;
        StatusBar::render(
            frame,
            app_layout.status_bar,
            self.reading_pane.book_name(),
            self.reading_pane.chapter_num(),
            self.reading_pane.current_verse_approx(),
            translation,
            self.error_message.as_deref(),
        );

        // Overlays
        if self.mode == Mode::Search {
            self.search_bar.render(frame, area, &self.bible.books);
        }

        if self.mode == Mode::Command {
            self.command_palette.render(frame, area);
        }

        if self.show_help {
            HelpOverlay::render(frame, area);
        }
    }
}
