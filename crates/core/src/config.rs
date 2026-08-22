use std::collections::HashMap;
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
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub border: BorderStyle,
    pub show_hidden: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            border: BorderStyle::Ascii,
            show_hidden: false,
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
