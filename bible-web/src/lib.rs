mod keys;

use std::cell::RefCell;
use std::rc::Rc;

use bible_core::app::App;
use bible_core::bible::loader;
use bible_core::bible::model::{BundledTranslation, TranslationId, TranslationInfo};
use bible_core::persistence::{NoPersistence, TranslationProvider};
use bible_core::bible::model::BibleData;
use ratzilla::DomBackend;
use ratzilla::ratatui::Terminal;
use ratzilla::WebRenderer;
use wasm_bindgen::prelude::*;

struct WasmTranslationProvider;

impl TranslationProvider for WasmTranslationProvider {
    fn available_translations(&self) -> Vec<TranslationInfo> {
        vec![
            BundledTranslation::Kjv.info(),
            BundledTranslation::Web.info(),
            BundledTranslation::Sv.info(),
        ]
    }

    fn load_translation(&self, id: &TranslationId) -> Option<BibleData> {
        match id {
            TranslationId::Bundled(bt) => Some(loader::load_bundled(*bt)),
            TranslationId::Imported(_) => None,
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let backend = DomBackend::new_by_id("terminal-body")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut terminal = Terminal::new(backend)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let events: Rc<RefCell<Vec<bible_core::keys::KeyEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let app = Rc::new(RefCell::new(App::new(
        Box::new(NoPersistence),
        Box::new(WasmTranslationProvider),
    )));

    terminal.on_key_event({
        let events = events.clone();
        move |key_event| {
            events.borrow_mut().push(keys::from_ratzilla(key_event));
        }
    }).map_err(|e| JsValue::from_str(&e.to_string()))?;

    terminal.draw_web(move |frame| {
        let pending: Vec<_> = events.borrow_mut().drain(..).collect();
        let mut app = app.borrow_mut();

        let current_width = frame.area().width;
        if current_width != app.last_width {
            app.update(&bible_core::action::Action::Resize(
                current_width,
                frame.area().height,
            ));
        }

        for event in pending {
            let action = app.handle_key(event);
            app.update(&action);
        }

        app.render(frame);
    });

    Ok(())
}
