//! A UI-toolkit-independent representation of a key press.
//!
//! The core must not depend on `crossterm` (or any other terminal crate), so
//! keymaps are matched against this small, stable vocabulary. The TUI layer
//! is responsible for translating real terminal events into a [`Key`].

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key {
    pub fn plain(code: KeyCode) -> Self {
        Key {
            code,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub fn ctrl(code: KeyCode) -> Self {
        Key {
            code,
            ctrl: true,
            alt: false,
            shift: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Delete,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

/// Parses a keybinding string like `"ctrl+s"`, `"q"`, `"ctrl+shift+n"` into a [`Key`].
/// Returns `None` if the string cannot be parsed; callers should fall back to
/// a built-in default and surface a warning rather than fail hard.
pub fn parse_key(spec: &str) -> Option<Key> {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut code: Option<KeyCode> = None;

    for part in spec.split('+') {
        let part = part.trim();
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            "enter" | "return" => code = Some(KeyCode::Enter),
            "esc" | "escape" => code = Some(KeyCode::Esc),
            "backspace" => code = Some(KeyCode::Backspace),
            "delete" | "del" => code = Some(KeyCode::Delete),
            "tab" => code = Some(KeyCode::Tab),
            "up" => code = Some(KeyCode::Up),
            "down" => code = Some(KeyCode::Down),
            "left" => code = Some(KeyCode::Left),
            "right" => code = Some(KeyCode::Right),
            "home" => code = Some(KeyCode::Home),
            "end" => code = Some(KeyCode::End),
            "pageup" => code = Some(KeyCode::PageUp),
            "pagedown" => code = Some(KeyCode::PageDown),
            "space" => code = Some(KeyCode::Char(' ')),
            other => {
                let mut chars = other.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    code = Some(KeyCode::Char(c));
                } else {
                    return None;
                }
            }
        }
    }

    code.map(|code| Key {
        code,
        ctrl,
        alt,
        shift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_char() {
        assert_eq!(parse_key("q"), Some(Key::plain(KeyCode::Char('q'))));
    }

    #[test]
    fn parses_ctrl_combo() {
        assert_eq!(parse_key("ctrl+s"), Some(Key::ctrl(KeyCode::Char('s'))));
    }

    #[test]
    fn parses_named_key() {
        assert_eq!(
            parse_key("ctrl+shift+enter"),
            Some(Key {
                code: KeyCode::Enter,
                ctrl: true,
                alt: false,
                shift: true,
            })
        );
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_key("notakey"), None);
    }
}
