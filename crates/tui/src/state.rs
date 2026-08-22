use std::path::PathBuf;

use tmr_core::theme::Theme;

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
    pub editor: Option<Editor>,
    pub status: Option<(String, StatusLevel)>,
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
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            focus: Focus::Files,
            mode: Mode::Normal,
            selected: 0,
            doc_cursor: 0,
            doc_scroll: 0,
            editor: None,
            status: None,
            rendered: Vec::new(),
            search_matches: Vec::new(),
            theme_choice: ThemeChoice::Default,
            line_indicator: LineIndicatorStyle::Highlight,
            default_theme: Theme::default(),
        }
    }
}

impl UiState {
    pub fn set_status(&mut self, msg: impl Into<String>, level: StatusLevel) {
        self.status = Some((msg.into(), level));
    }

    pub fn clear_status(&mut self) {
        self.status = None;
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
}
