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
    /// Adds a new open task to the persistent Quick-TODO store.
    AddTask(String),
    /// Flips a task between open and done.
    ToggleTaskDone(u64),
    /// Soft-deletes a task (kept for `ExportTasks`'s historical record).
    DeleteTask(u64),
    /// Reorders a task among the visible (non-deleted) ones.
    MoveTask { id: u64, delta: i32 },
    /// Writes the full task history (open, done, and deleted) to `path` as
    /// TSV.
    ExportTasks(PathBuf),
}
