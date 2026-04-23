use bible_core::keys::{KeyCode, KeyEvent};
use crossterm::event::{KeyCode as CxCode, KeyEvent as CxKeyEvent, KeyModifiers};

pub fn from_crossterm(e: CxKeyEvent) -> KeyEvent {
    let ctrl = e.modifiers.contains(KeyModifiers::CONTROL);
    let shift = e.modifiers.contains(KeyModifiers::SHIFT);
    let alt = e.modifiers.contains(KeyModifiers::ALT);
    let code = match e.code {
        CxCode::Char(c) => KeyCode::Char(c),
        CxCode::Enter => KeyCode::Enter,
        CxCode::Esc => KeyCode::Esc,
        CxCode::Tab => KeyCode::Tab,
        CxCode::BackTab => KeyCode::Tab, // shift+tab — treat as Tab with shift=true
        CxCode::Backspace => KeyCode::Backspace,
        CxCode::Left => KeyCode::Left,
        CxCode::Right => KeyCode::Right,
        CxCode::Up => KeyCode::Up,
        CxCode::Down => KeyCode::Down,
        CxCode::PageUp => KeyCode::PageUp,
        CxCode::PageDown => KeyCode::PageDown,
        CxCode::Home => KeyCode::Home,
        CxCode::End => KeyCode::End,
        CxCode::Delete => KeyCode::Delete,
        CxCode::F(n) => KeyCode::F(n),
        _ => KeyCode::Unidentified,
    };
    KeyEvent { code, ctrl, shift, alt }
}
