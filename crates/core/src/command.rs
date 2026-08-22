use std::path::PathBuf;

/// The full vocabulary of operations the engine can perform. The TUI never
/// touches the filesystem or document state directly — every action flows
/// through here: `Key -> Command -> App::dispatch -> AppEvent -> render`.
#[derive(Debug, Clone)]
pub enum Command {
    /// (Re-)lists a directory, updating the engine's navigation state.
    ListDir(PathBuf),
    /// Opens a file as the current document.
    OpenFile(PathBuf),
    /// Persists `content` to the current document's path.
    Save(String),
    /// Creates a new, empty file.
    CreateFile(PathBuf),
    /// Deletes a file. The TUI is responsible for confirming with the user
    /// first — this executes unconditionally once dispatched.
    DeleteFile(PathBuf),
    /// Renames/moves a file.
    RenameFile { from: PathBuf, to: PathBuf },
    /// Toggles the `index`-th task-list checkbox in the current document,
    /// persisting the change to disk immediately.
    ToggleTask(usize),
    /// Filters the current directory listing by filename.
    SearchFilenames(String),
    /// Finds matching lines within the current document's content.
    SearchInDocument(String),
    /// Re-reads the current directory listing from disk.
    Reload,
}
