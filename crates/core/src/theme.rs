use std::path::Path;

use serde::Deserialize;

/// A color palette. Colors are kept as raw strings (hex like `"#1e1e2e"` or
/// ANSI names like `"blue"`) rather than a UI-toolkit color type, so this
/// crate never has to depend on ratatui. The TUI layer parses these into
/// whatever color type it needs.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub accent: String,
    pub border: String,
    pub muted: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Theme {
            name: "dark".into(),
            foreground: "#cdd6f4".into(),
            background: "#1e1e2e".into(),
            accent: "#89b4fa".into(),
            border: "#585b70".into(),
            muted: "#6c7086".into(),
            success: "#a6e3a1".into(),
            warning: "#f9e2af".into(),
            error: "#f38ba8".into(),
        }
    }

    pub fn light() -> Self {
        Theme {
            name: "light".into(),
            foreground: "#4c4f69".into(),
            background: "#eff1f5".into(),
            accent: "#1e66f5".into(),
            border: "#9ca0b0".into(),
            muted: "#8c8fa1".into(),
            success: "#40a02b".into(),
            warning: "#df8e1d".into(),
            error: "#d20f39".into(),
        }
    }

    /// Resolves a theme by name: tries `<config_dir>/themes/<name>.toml`
    /// first, then falls back to a built-in palette ("dark"/"light"), then
    /// finally to the built-in dark palette if the name is unrecognized.
    /// Returns the theme plus an optional warning describing what happened.
    pub fn resolve(config_dir: Option<&Path>, name: &str) -> (Theme, Option<String>) {
        if let Some(dir) = config_dir {
            let path = dir.join("themes").join(format!("{name}.toml"));
            if path.exists() {
                return match std::fs::read_to_string(&path) {
                    Ok(raw) => match toml::from_str::<Theme>(&raw) {
                        Ok(theme) => (theme, None),
                        Err(e) => (
                            Theme::built_in_or_dark(name),
                            Some(format!("invalid theme file {}: {e}", path.display())),
                        ),
                    },
                    Err(e) => (
                        Theme::built_in_or_dark(name),
                        Some(format!("could not read theme file {}: {e}", path.display())),
                    ),
                };
            }
        }
        match name {
            "dark" => (Theme::dark(), None),
            "light" => (Theme::light(), None),
            other => (
                Theme::dark(),
                Some(format!("unknown theme '{other}', falling back to 'dark'")),
            ),
        }
    }

    fn built_in_or_dark(name: &str) -> Theme {
        match name {
            "light" => Theme::light(),
            _ => Theme::dark(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_falls_back_to_dark_for_unknown_name() {
        let (theme, warning) = Theme::resolve(None, "does-not-exist");
        assert_eq!(theme.name, "dark");
        assert!(warning.is_some());
    }

    #[test]
    fn resolve_reads_theme_file_when_present() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("themes")).unwrap();
        std::fs::write(
            dir.path().join("themes/custom.toml"),
            r##"name = "custom"
foreground = "#ffffff"
background = "#000000"
accent = "#ff00ff"
border = "#333333"
muted = "#888888"
success = "#00ff00"
warning = "#ffff00"
error = "#ff0000"
"##,
        )
        .unwrap();
        let (theme, warning) = Theme::resolve(Some(dir.path()), "custom");
        assert!(warning.is_none());
        assert_eq!(theme.accent, "#ff00ff");
    }
}
