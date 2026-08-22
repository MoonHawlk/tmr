use std::collections::HashMap;

use serde::Deserialize;

use crate::input::{parse_key, Key};

/// Default action -> key-spec bindings. These are the only keys tmr ships
/// with hardcoded; everything else flows through this table so behavior can
/// be remapped entirely from `config.toml`'s `[keys]` section.
const DEFAULTS: &[(&str, &str)] = &[
    ("quit", "q"),
    ("save", "ctrl+s"),
    ("search", "/"),
    ("new_file", "ctrl+n"),
    ("delete", "d"),
    ("rename", "r"),
    ("reload", "ctrl+r"),
    ("toggle_task", "space"),
    ("edit", "enter"),
    ("cancel", "esc"),
    ("confirm", "y"),
    ("nav_up", "up"),
    ("nav_down", "down"),
    ("nav_enter", "right"),
    ("nav_back", "left"),
    ("focus_files", "tab"),
    ("help", "h"),
    ("settings", "s"),
    ("select_all", "ctrl+a"),
    ("copy", "ctrl+c"),
    ("cut", "ctrl+x"),
];

/// Action name -> key-spec string, as configured by the user (or defaults).
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct Keymap(HashMap<String, String>);

impl Default for Keymap {
    fn default() -> Self {
        Keymap(
            DEFAULTS
                .iter()
                .map(|(a, k)| (a.to_string(), k.to_string()))
                .collect(),
        )
    }
}

impl Keymap {
    /// Merges user-provided overrides on top of the defaults; unknown
    /// action names are kept too, so addons can register their own actions
    /// with default bindings supplied at registration time.
    pub fn with_overrides(overrides: HashMap<String, String>) -> Self {
        let mut map = Keymap::default().0;
        map.extend(overrides);
        Keymap(map)
    }

    /// Builds a reverse lookup table (parsed `Key` -> action name). Specs
    /// that fail to parse are skipped rather than causing a startup error.
    pub fn resolve(&self) -> HashMap<Key, String> {
        self.0
            .iter()
            .filter_map(|(action, spec)| parse_key(spec).map(|key| (key, action.clone())))
            .collect()
    }

    pub fn key_for(&self, action: &str) -> Option<Key> {
        self.0.get(action).and_then(|spec| parse_key(spec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyCode;

    #[test]
    fn defaults_resolve_quit_to_q() {
        let km = Keymap::default();
        assert_eq!(km.key_for("quit"), Some(Key::plain(KeyCode::Char('q'))));
    }

    #[test]
    fn overrides_replace_only_given_actions() {
        let mut overrides = HashMap::new();
        overrides.insert("quit".to_string(), "ctrl+q".to_string());
        let km = Keymap::with_overrides(overrides);
        assert_eq!(km.key_for("quit"), Some(Key::ctrl(KeyCode::Char('q'))));
        // untouched default still present
        assert_eq!(km.key_for("save"), Some(Key::ctrl(KeyCode::Char('s'))));
    }

    #[test]
    fn resolve_builds_reverse_map() {
        let km = Keymap::default();
        let reverse = km.resolve();
        assert_eq!(
            reverse
                .get(&Key::plain(KeyCode::Char('q')))
                .map(|s| s.as_str()),
            Some("quit")
        );
    }
}
