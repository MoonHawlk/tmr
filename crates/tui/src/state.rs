use std::path::PathBuf;
use std::time::{Duration, Instant};

use tmr_core::theme::Theme;

/// How long a transient status message (e.g. "Saved", "Deleted") stays on
/// screen before the status bar reverts to the default helper text on its
/// own, with no further keypress required. See `UiState::status_expired`
/// and `lib.rs::run_loop`, which polls at a short interval while a status
/// is pending so the revert happens close to on time.
pub const STATUS_TTL: Duration = Duration::from_secs(4);

use crate::editor::Editor;
use crate::markdown_view::RenderedLine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Files,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    Files,
    Document,
}

#[derive(Debug, Clone)]
pub enum PromptKind {
    NewFile,
    Rename { from: PathBuf },
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete { path: PathBuf },
}

pub enum Mode {
    Normal,
    Edit,
    Search {
        scope: SearchScope,
        buffer: String,
    },
    Prompt {
        kind: PromptKind,
        buffer: String,
    },
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    /// A visual-only command reference, opened with `h` (default binding),
    /// available any time in Normal mode regardless of focus or whether a
    /// document is open (Edit mode intercepts `h` as a literal character
    /// instead, so it never fires mid-edit). `query` filters the list as
    /// the user types; `selected` is the highlighted row within the
    /// filtered list (reset to 0 whenever `query` changes), used both to
    /// mark a row and to let ratatui's `ListState` auto-scroll the list
    /// into view on terminals too short to fit every entry at once.
    /// Nothing in this mode dispatches a `Command`.
    Help {
        query: String,
        selected: usize,
    },
    /// The interface-customization window, opened with `s` (default
    /// binding). `row` is the currently highlighted setting (0 = Theme,
    /// 1 = Line indicator); Up/Down moves between rows, Left/Right/Enter
    /// cycles the highlighted row's value, applied live. Esc closes it.
    Settings {
        row: usize,
    },
}

/// The three theme options the Settings window offers. `Default` is
/// whatever theme tmr loaded at startup (from `config.toml`) — captured
/// once in `UiState::default_theme` — so switching to `Dark` or `Light`
/// and back to `Default` always returns to the user's configured theme,
/// not to `Theme::dark()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Default,
    Dark,
    Light,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [ThemeChoice::Default, ThemeChoice::Dark, ThemeChoice::Light];

    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::Default => "Default",
            ThemeChoice::Dark => "Dark",
            ThemeChoice::Light => "Light (grey)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ThemeChoice::Default => ThemeChoice::Dark,
            ThemeChoice::Dark => ThemeChoice::Light,
            ThemeChoice::Light => ThemeChoice::Default,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ThemeChoice::Default => ThemeChoice::Light,
            ThemeChoice::Dark => ThemeChoice::Default,
            ThemeChoice::Light => ThemeChoice::Dark,
        }
    }

    /// The `[theme] name` value to persist for this choice — `Default`
    /// round-trips whatever `config.toml` already had (`default_name`,
    /// snapshotted at startup in `UiState::default_theme_name`), since it
    /// has no single canonical name of its own (it could be a custom
    /// theme file). `Dark`/`Light` map to the same built-in names
    /// `Theme::resolve` understands (`Light` here is the `grey` palette —
    /// see `resolve`'s `ThemeChoice::Light` arm).
    pub fn persisted_name(self, default_name: &str) -> String {
        match self {
            ThemeChoice::Default => default_name.to_string(),
            ThemeChoice::Dark => "dark".to_string(),
            ThemeChoice::Light => "grey".to_string(),
        }
    }

    /// Resolves this choice to an actual [`Theme`], given the theme tmr
    /// loaded at startup (what `Default` means).
    pub fn resolve(self, default_theme: &Theme) -> Theme {
        match self {
            ThemeChoice::Default => default_theme.clone(),
            ThemeChoice::Dark => Theme::dark(),
            ThemeChoice::Light => Theme::light_grey(),
        }
    }
}

/// How the Document pane marks the cursor's current line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineIndicatorStyle {
    /// Reverse-video the whole line (the original/default behavior).
    Highlight,
    /// A single `▏` marker in the gutter, next to the line number — closer
    /// to how a plain terminal cursor marks a position.
    Bar,
}

impl LineIndicatorStyle {
    pub fn label(self) -> &'static str {
        match self {
            LineIndicatorStyle::Highlight => "Highlight",
            LineIndicatorStyle::Bar => "Bar (\u{258F})",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            LineIndicatorStyle::Highlight => LineIndicatorStyle::Bar,
            LineIndicatorStyle::Bar => LineIndicatorStyle::Highlight,
        }
    }

    /// The `[ui] line_indicator` string to persist/read for this style.
    pub fn config_str(self) -> &'static str {
        match self {
            LineIndicatorStyle::Highlight => "highlight",
            LineIndicatorStyle::Bar => "bar",
        }
    }

    /// Parses a `[ui] line_indicator` config value, falling back to
    /// `Highlight` for anything unrecognized (including an absent key,
    /// which deserializes to `UiConfig::default`'s `"highlight"`).
    pub fn from_config_str(s: &str) -> Self {
        match s {
            "bar" => LineIndicatorStyle::Bar,
            _ => LineIndicatorStyle::Highlight,
        }
    }
}

/// All interaction/presentation state that isn't part of the core engine:
/// which pane has focus, dialog state, cursor/scroll positions, the cached
/// rendered document, and the in-progress edit buffer. `tmr-core` knows
/// nothing about any of this.
pub struct UiState {
    pub focus: Focus,
    pub mode: Mode,
    pub selected: usize,
    pub doc_cursor: usize,
    pub doc_scroll: usize,
    /// Horizontal scroll offset (in characters) for the Edit-mode raw-
    /// source view — mirrors `doc_scroll`'s vertical role, but for
    /// columns. Always `0` outside Edit mode, where there's no per-line
    /// column position to track (see `lib.rs::run_loop`).
    pub doc_hscroll: usize,
    pub editor: Option<Editor>,
    /// A transient status message, its severity, and when it was set — the
    /// timestamp lets `status_expired` revert to the default helper bar
    /// after `STATUS_TTL` even if the user issues no further command (see
    /// `lib.rs::run_loop`).
    pub status: Option<(String, StatusLevel, Instant)>,
    pub rendered: Vec<RenderedLine>,
    pub search_matches: Vec<usize>,
    pub theme_choice: ThemeChoice,
    pub line_indicator: LineIndicatorStyle,
    /// The theme tmr loaded at startup, snapshotted once so the Settings
    /// window's `Default` option always means "what config.toml selected",
    /// even after switching to `Dark`/`Light` and back. Set right after
    /// construction in `lib.rs::run_loop`, once `App`'s theme is known —
    /// `UiState::default()` can't see it, so this starts as a placeholder.
    pub default_theme: Theme,
    /// The exact `config.toml` `[theme] name` string tmr started with —
    /// distinct from `default_theme.name`, which can be empty for a custom
    /// theme file that doesn't set its own `name` key. Used by
    /// `ThemeChoice::persisted_name` to round-trip `Default` back to disk
    /// without guessing. Also set in `lib.rs::run_loop`.
    pub default_theme_name: String,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            focus: Focus::Files,
            mode: Mode::Normal,
            selected: 0,
            doc_cursor: 0,
            doc_scroll: 0,
            doc_hscroll: 0,
            editor: None,
            status: None,
            rendered: Vec::new(),
            search_matches: Vec::new(),
            theme_choice: ThemeChoice::Default,
            line_indicator: LineIndicatorStyle::Highlight,
            default_theme: Theme::default(),
            default_theme_name: String::new(),
        }
    }
}

impl UiState {
    pub fn set_status(&mut self, msg: impl Into<String>, level: StatusLevel) {
        self.status = Some((msg.into(), level, Instant::now()));
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    /// Clears a status message once it's older than `STATUS_TTL`, so the
    /// status bar reverts to the default helper text on its own rather than
    /// staying stuck until the next command. Called every loop iteration in
    /// `lib.rs::run_loop`, including on the idle-poll timeout that fires
    /// while a status is pending.
    pub fn expire_status(&mut self) {
        if let Some((_, _, set_at)) = &self.status {
            if set_at.elapsed() >= STATUS_TTL {
                self.status = None;
            }
        }
    }

    /// Keeps `doc_cursor` within the visible window, scrolling as needed.
    pub fn ensure_doc_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.doc_cursor < self.doc_scroll {
            self.doc_scroll = self.doc_cursor;
        } else if self.doc_cursor >= self.doc_scroll + height {
            self.doc_scroll = self.doc_cursor - height + 1;
        }
    }

    /// Keeps a live cursor column within a viewport of `width` visible
    /// columns, the same way `ensure_doc_visible` does for rows. Called
    /// only while in Edit mode (see `lib.rs::run_loop`); `col` is the
    /// editor's live cursor column.
    pub fn ensure_doc_hscroll(&mut self, col: usize, width: usize) {
        if width == 0 {
            return;
        }
        if col < self.doc_hscroll {
            self.doc_hscroll = col;
        } else if col >= self.doc_hscroll + width {
            self.doc_hscroll = col - width + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expire_status_clears_a_message_older_than_the_ttl() {
        let mut ui = UiState {
            status: Some((
                "Saved".to_string(),
                StatusLevel::Success,
                Instant::now() - STATUS_TTL - Duration::from_millis(1),
            )),
            ..UiState::default()
        };
        ui.expire_status();
        assert!(ui.status.is_none());
    }

    #[test]
    fn expire_status_keeps_a_fresh_message() {
        let mut ui = UiState::default();
        ui.set_status("Saved", StatusLevel::Success);
        ui.expire_status();
        assert!(ui.status.is_some());
    }

    #[test]
    fn ensure_doc_hscroll_scrolls_right_when_cursor_passes_the_edge() {
        let mut ui = UiState::default();
        ui.ensure_doc_hscroll(25, 20);
        assert_eq!(ui.doc_hscroll, 6);
        assert!(25 >= ui.doc_hscroll && 25 < ui.doc_hscroll + 20);
    }

    #[test]
    fn ensure_doc_hscroll_scrolls_left_when_cursor_moves_before_the_view() {
        let mut ui = UiState {
            doc_hscroll: 10,
            ..UiState::default()
        };
        ui.ensure_doc_hscroll(3, 20);
        assert_eq!(ui.doc_hscroll, 3);
    }

    #[test]
    fn ensure_doc_hscroll_is_noop_for_a_column_already_in_view() {
        let mut ui = UiState {
            doc_hscroll: 5,
            ..UiState::default()
        };
        ui.ensure_doc_hscroll(10, 20);
        assert_eq!(ui.doc_hscroll, 5);
    }
}
