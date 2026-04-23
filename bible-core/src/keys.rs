/// Platform-neutral key event, bridging crossterm (native) and Ratzilla (WASM).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyEvent {
    pub fn plain(code: KeyCode) -> Self {
        Self { code, ctrl: false, shift: false, alt: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    F(u8),
    Unidentified,
}
