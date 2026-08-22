use std::path::PathBuf;

use crate::search::LineMatch;
use crate::workspace::Entry;

/// Describes what happened after a [`crate::command::Command`] was
/// dispatched. The TUI reacts to these (refresh a pane, show a status
/// message); widgets and addons observe them via their `on_event` hook.
#[derive(Debug, Clone)]
pub enum AppEvent {
    DirectoryListed {
        dir: PathBuf,
        entries: Vec<Entry>,
    },
    DocumentOpened {
        path: PathBuf,
    },
    DocumentSaved {
        path: PathBuf,
    },
    FileCreated {
        path: PathBuf,
    },
    FileDeleted {
        path: PathBuf,
    },
    FileRenamed {
        from: PathBuf,
        to: PathBuf,
    },
    TaskToggled {
        index: usize,
    },
    FilenameResults {
        query: String,
        matches: Vec<Entry>,
    },
    TextSearchResults {
        query: String,
        matches: Vec<LineMatch>,
    },
    Reloaded,
    /// A task was added, toggled, deleted, or reordered.
    TasksChanged,
    TasksExported {
        path: PathBuf,
    },
}
