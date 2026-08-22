use std::path::PathBuf;

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
    pub show_help: bool,
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
            show_help: false,
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
