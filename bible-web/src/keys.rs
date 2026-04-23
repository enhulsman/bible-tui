use bible_core::keys::{KeyCode, KeyEvent};
use ratzilla::event::{KeyCode as RzCode, KeyEvent as RzKeyEvent};

pub fn from_ratzilla(e: RzKeyEvent) -> KeyEvent {
    let code = match e.code {
        RzCode::Char(c) => KeyCode::Char(c),
        RzCode::Enter => KeyCode::Enter,
        RzCode::Esc => KeyCode::Esc,
        RzCode::Tab => KeyCode::Tab,
        RzCode::Backspace => KeyCode::Backspace,
        RzCode::Left => KeyCode::Left,
        RzCode::Right => KeyCode::Right,
        RzCode::Up => KeyCode::Up,
        RzCode::Down => KeyCode::Down,
        RzCode::PageUp => KeyCode::PageUp,
        RzCode::PageDown => KeyCode::PageDown,
        RzCode::Home => KeyCode::Home,
        RzCode::End => KeyCode::End,
        RzCode::Delete => KeyCode::Delete,
        RzCode::F(n) => KeyCode::F(n),
        RzCode::Unidentified => KeyCode::Unidentified,
    };
    KeyEvent { code, ctrl: e.ctrl, shift: e.shift, alt: e.alt }
}
