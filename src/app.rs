use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

use crate::action::Action;
use crate::bible::loader;
use crate::bible::model::{BibleData, BundledTranslation, VerseRef};
use crate::bible::reference;
use crate::components::command_palette::CommandPalette;
use crate::components::help_overlay::HelpOverlay;
use crate::components::nav_panel::NavPanel;
use crate::components::reading_pane::ReadingPane;
use crate::components::search_bar::SearchBar;
use crate::components::status_bar::StatusBar;
use crate::event::Event;
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
    current_translation: BundledTranslation,
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
        let current_translation = BundledTranslation::Kjv;
        let bible = loader::load_bundled(current_translation);
        let search_index = SearchIndex::build(&bible);
        let mut reading_pane = ReadingPane::new();

        let storage = Storage::new().ok();
        let bookmarks = storage
            .as_ref()
            .map(|s| BookmarkStore::load(&s.bookmarks_path()))
            .unwrap_or_default();
        let app_state = storage
            .as_ref()
            .map(|s| AppState::load(&s.state_path()))
            .unwrap_or_default();

        let start_book = app_state.last_book;
        let start_chapter = app_state.last_chapter.max(1);

        if let Some(book) = bible.books.get(start_book as usize) {
            let ch_idx = (start_chapter as usize).saturating_sub(1);
            if let Some(chapter) = book.chapters.get(ch_idx) {
                reading_pane.set_chapter(start_book, chapter, 80);
            } else if let Some(chapter) = book.chapters.first() {
                reading_pane.set_chapter(start_book, chapter, 80);
            }
        } else if let Some(chapter) = bible.books.first().and_then(|b| b.chapters.first()) {
            reading_pane.set_chapter(0, chapter, 80);
        }

        Self {
            running: true,
            mode: Mode::Normal,
            bible,
            current_translation,
            reading_pane,
            nav_panel: NavPanel::new(),
            search_bar: SearchBar::new(),
            search_index,
            command_palette: CommandPalette::new(),
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
                last_translation: Some(self.bible.translation.to_string()),
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
        let new_translation = match name.to_lowercase().as_str() {
            "kjv" => Some(BundledTranslation::Kjv),
            "web" => Some(BundledTranslation::Web),
            "sv" => Some(BundledTranslation::Sv),
            _ => None,
        };

        if let Some(t) = new_translation {
            if t != self.current_translation {
                let book_idx = self.reading_pane.book_index();
                let ch_num = self.reading_pane.chapter_num();

                self.current_translation = t;
                self.bible = loader::load_bundled(t);
                self.search_index = SearchIndex::build(&self.bible);

                // Navigate to the same position in the new translation
                self.navigate_to(book_idx, ch_num);
            }
        }
    }

    fn navigate_to(&mut self, book_index: u8, chapter: u16) {
        if let Some(book) = self.bible.books.get(book_index as usize) {
            let ch_idx = (chapter as usize).saturating_sub(1);
            if let Some(ch) = book.chapters.get(ch_idx) {
                self.reading_pane
                    .set_chapter(book_index, ch, self.content_width());
            }
        }
    }

    fn navigate_chapter(&mut self, delta: i32) {
        let book_idx = self.reading_pane.book_index() as usize;
        let ch_num = self.reading_pane.chapter_num();

        if let Some(book) = self.bible.books.get(book_idx) {
            let ch_idx = ch_num as i32 - 1 + delta;

            if ch_idx >= 0 && (ch_idx as usize) < book.chapters.len() {
                let chapter = &book.chapters[ch_idx as usize];
                self.reading_pane
                    .set_chapter(book_idx as u8, chapter, self.content_width());
            } else if delta > 0 && book_idx + 1 < self.bible.books.len() {
                let next_book = &self.bible.books[book_idx + 1];
                if let Some(chapter) = next_book.chapters.first() {
                    self.reading_pane
                        .set_chapter((book_idx + 1) as u8, chapter, self.content_width());
                }
            } else if delta < 0 && book_idx > 0 {
                let prev_book = &self.bible.books[book_idx - 1];
                if let Some(chapter) = prev_book.chapters.last() {
                    self.reading_pane
                        .set_chapter((book_idx - 1) as u8, chapter, self.content_width());
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
            self.nav_panel.render(frame, nav_area);
        }

        self.reading_pane.render(frame, app_layout.reading_pane);

        let translation = self.current_translation.abbreviation();
        StatusBar::render(
            frame,
            app_layout.status_bar,
            self.reading_pane.book_index(),
            self.reading_pane.chapter_num(),
            self.reading_pane.current_verse_approx(),
            translation,
            self.error_message.as_deref(),
        );

        // Overlays
        if self.mode == Mode::Search {
            self.search_bar.render(frame, area);
        }

        if self.mode == Mode::Command {
            self.command_palette.render(frame, area);
        }

        if self.show_help {
            HelpOverlay::render(frame, area);
        }
    }
}
