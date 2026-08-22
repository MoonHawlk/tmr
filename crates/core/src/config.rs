use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Which directory to browse when `tmr` is launched with no path argument.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
    /// Falls back to the current working directory when unset.
    pub default_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ThemeSelection {
    pub name: String,
}

impl Default for ThemeSelection {
    fn default() -> Self {
        ThemeSelection {
            name: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    Ascii,
    Rounded,
    /// Double-line box-drawing borders (`╔ ═ ╗ / ║ ║ / ╚ ═ ╝`) — a denser,
    /// more app-like panel look than the plain-terminal `+---+` default,
    /// closer to a classic boxed TUI (Turbo Vision, Norton Commander) than
    /// a raw ASCII sketch.
    Double,
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub border: BorderStyle,
    pub show_hidden: bool,
    /// How the Document pane marks its current line: `"highlight"` (full-
    /// line reverse-video) or `"bar"` (a gutter marker). Mirrors the TUI's
    /// `LineIndicatorStyle` — kept as a plain string here so `tmr-core`
    /// doesn't need to know that type exists. An unrecognized value falls
    /// back to `"highlight"`, the same as an absent one.
    pub line_indicator: String,
    /// Shows a thin bar with the current time (UTC) at the very top of the
    /// TUI, above the Files/Document panes and below the terminal's top
    /// edge — sitting between those and the Status bar. Off by default so
    /// the default screen matches the README's layout diagram exactly.
    pub timer: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            border: BorderStyle::Ascii,
            show_hidden: false,
            line_indicator: "highlight".to_string(),
            timer: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub tab_width: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        EditorConfig { tab_width: 4 }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AddonsConfig {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WidgetsConfig {
    pub enabled: Vec<String>,
}

/// Top-level configuration, loaded from `~/.config/tmr/config.toml` (or a
/// path given via `--config`). Every field has a sensible built-in default,
/// used whenever the file is absent, partially specified, or invalid.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub workspace: WorkspaceConfig,
    pub theme: ThemeSelection,
    pub ui: UiConfig,
    pub editor: EditorConfig,
    pub keys: HashMap<String, String>,
    pub addons: AddonsConfig,
    pub widgets: WidgetsConfig,
}

/// Result of loading configuration: the resolved config plus any warnings
/// that should be surfaced to the user (invalid file, unreadable path...).
/// Loading never fails outright — it always falls back to defaults.
pub struct LoadResult {
    pub config: Config,
    pub warnings: Vec<String>,
}

/// Returns `~/.config/tmr` (or the platform's XDG-equivalent config dir).
pub fn default_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tmr"))
}

/// Returns the `config.toml` path `Config::load(None)` reads — the only
/// path the running app currently knows how to write back to (there's no
/// `--config` CLI flag wired up yet, so this always agrees with what was
/// loaded at startup).
pub fn default_config_path() -> Option<PathBuf> {
    default_config_dir().map(|d| d.join("config.toml"))
}

/// Writes `theme.name` and `ui.line_indicator` into the config file at
/// `path`, creating it (and its parent directory) if it doesn't exist yet,
/// while leaving every other key's value, formatting and comments
/// untouched — used by the Settings window to persist its two
/// live-editable choices without clobbering the rest of a hand-edited
/// `config.toml`. A `toml_edit::DocumentMut` (format-preserving, unlike
/// this module's `toml::from_str`/plain-struct-based loading) is what
/// makes that possible.
pub fn persist_settings(path: &Path, theme_name: &str, line_indicator: &str) -> io::Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut doc = existing
        .parse::<toml_edit::DocumentMut>()
        .unwrap_or_default();
    doc["theme"]["name"] = toml_edit::value(theme_name);
    doc["ui"]["line_indicator"] = toml_edit::value(line_indicator);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, doc.to_string())
}

impl Config {
    /// Loads configuration from `explicit_path` if given, otherwise from
    /// the default config dir's `config.toml`. Missing files are not an
    /// error (a fresh install has none); malformed files fall back to
    /// defaults with a warning.
    pub fn load(explicit_path: Option<&Path>) -> LoadResult {
        let path = match explicit_path {
            Some(p) => Some(p.to_path_buf()),
            None => default_config_dir().map(|d| d.join("config.toml")),
        };

        let mut warnings = Vec::new();
        let config = match path {
            Some(path) if path.exists() => match std::fs::read_to_string(&path) {
                Ok(raw) => match toml::from_str::<Config>(&raw) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        warnings.push(format!(
                            "invalid config at {}: {e} (using defaults)",
                            path.display()
                        ));
                        Config::default()
                    }
                },
                Err(e) => {
                    warnings.push(format!(
                        "could not read config at {}: {e} (using defaults)",
                        path.display()
                    ));
                    Config::default()
                }
            },
            Some(path) if explicit_path.is_some() => {
                warnings.push(format!(
                    "config file not found at {} (using defaults)",
                    path.display()
                ));
                Config::default()
            }
            _ => Config::default(),
        };

        LoadResult { config, warnings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_settings_creates_a_missing_file_and_its_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        persist_settings(&path, "dark", "bar").unwrap();
        let result = Config::load(Some(&path));
        assert_eq!(result.config.theme.name, "dark");
        assert_eq!(result.config.ui.line_indicator, "bar");
    }

    #[test]
    fn persist_settings_preserves_untouched_keys_and_comments() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "# a comment worth keeping\n[theme]\nname = \"grey\"\n\n[editor]\ntab_width = 8\n",
        )
        .unwrap();

        persist_settings(&path, "dark", "bar").unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("# a comment worth keeping"));
        let result = Config::load(Some(&path));
        assert_eq!(result.config.theme.name, "dark");
        assert_eq!(result.config.ui.line_indicator, "bar");
        assert_eq!(result.config.editor.tab_width, 8);
    }

    #[test]
    fn missing_file_yields_defaults_without_warning() {
        let dir = tempfile::tempdir().unwrap();
        let result = Config::load(Some(&dir.path().join("nope.toml")));
        assert_eq!(result.warnings.len(), 1); // explicit path missing -> warn
        assert_eq!(result.config.theme.name, "dark");
    }

    #[test]
    fn valid_partial_file_merges_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[theme]\nname = \"light\"\n").unwrap();
        let result = Config::load(Some(&path));
        assert!(result.warnings.is_empty());
        assert_eq!(result.config.theme.name, "light");
        assert_eq!(result.config.editor.tab_width, 4);
    }

    #[test]
    fn invalid_toml_falls_back_to_defaults_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not [ valid toml").unwrap();
        let result = Config::load(Some(&path));
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.config.theme.name, "dark");
    }
}
