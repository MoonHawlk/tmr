use crossterm::event::{KeyCode as CtCode, KeyEvent, KeyModifiers};
use tmr_core::input::{Key, KeyCode};

/// Translates a `crossterm` key event into the core's UI-independent [`Key`]
/// type, so `tmr-core`'s keymap never has to know `crossterm` exists.
pub fn to_core_key(ev: KeyEvent) -> Option<Key> {
    let code = match ev.code {
        CtCode::Char(c) => KeyCode::Char(c),
        CtCode::Enter => KeyCode::Enter,
        CtCode::Esc => KeyCode::Esc,
        CtCode::Backspace => KeyCode::Backspace,
        CtCode::Delete => KeyCode::Delete,
        CtCode::Tab => KeyCode::Tab,
        CtCode::Up => KeyCode::Up,
        CtCode::Down => KeyCode::Down,
        CtCode::Left => KeyCode::Left,
        CtCode::Right => KeyCode::Right,
        CtCode::Home => KeyCode::Home,
        CtCode::End => KeyCode::End,
        CtCode::PageUp => KeyCode::PageUp,
        CtCode::PageDown => KeyCode::PageDown,
        _ => return None,
    };
    Some(Key {
        code,
        ctrl: ev.modifiers.contains(KeyModifiers::CONTROL),
        alt: ev.modifiers.contains(KeyModifiers::ALT),
        shift: ev.modifiers.contains(KeyModifiers::SHIFT),
    })
}
