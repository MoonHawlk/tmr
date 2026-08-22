//! A tiny persistent task store for the Quick-TODO window
//! (`crates/tui/src/todo_view.rs`) — plain TSV on disk, independent of any
//! open document or workspace, so tasks survive across sessions and
//! directories and stay available to other tmr features later (search,
//! filtering, the `Ctrl+E` export). See `config::default_tasks_path`.
//!
//! Deletion is soft: a deleted task moves to `TaskStatus::Deleted` and
//! stays in the file rather than being erased, so the historical record
//! `export_tsv` produces is complete. `visible()` — what the Quick-TODO
//! window actually shows — filters `Deleted` out.

use std::io;
use std::path::Path;

use crate::datetime::now_unix_secs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Done,
    Deleted,
}

impl TaskStatus {
    fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::Done => "done",
            TaskStatus::Deleted => "deleted",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "open" => Some(TaskStatus::Open),
            "done" => Some(TaskStatus::Done),
            "deleted" => Some(TaskStatus::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: u64,
    pub status: TaskStatus,
    pub created_at: u64,
    pub done_at: Option<u64>,
    pub text: String,
}

/// Replaces any literal tab/newline in `text` with a space, so a task's
/// text can never break the one-task-per-line TSV format it's stored in.
/// Simple tasks are expected to be single-line; this just guarantees it
/// rather than erroring on input that isn't.
fn sanitize_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c == '\t' || c == '\n' || c == '\r' {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn format_line(task: &Task) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}",
        task.id,
        task.status.as_str(),
        task.created_at,
        task.done_at.map(|t| t.to_string()).unwrap_or_default(),
        sanitize_text(&task.text),
    )
}

fn parse_line(line: &str) -> Option<Task> {
    let mut parts = line.splitn(5, '\t');
    let id = parts.next()?.parse().ok()?;
    let status = TaskStatus::parse(parts.next()?)?;
    let created_at = parts.next()?.parse().ok()?;
    let done_at_raw = parts.next()?;
    let done_at = if done_at_raw.is_empty() {
        None
    } else {
        done_at_raw.parse().ok()
    };
    let text = parts.next().unwrap_or_default().to_string();
    Some(Task {
        id,
        status,
        created_at,
        done_at,
        text,
    })
}

/// In-memory task list, mirrored to a TSV file. Every mutating method just
/// updates `self.tasks`; callers (`App::execute`) are responsible for
/// calling `save` afterward — kept separate so a caller that wants to
/// batch several changes into one write can.
#[derive(Debug, Default)]
pub struct TaskStore {
    tasks: Vec<Task>,
}

impl TaskStore {
    /// Loads from `path`. A missing file is not an error — a fresh install
    /// has none — and just yields an empty store.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(raw) => Ok(TaskStore {
                tasks: raw.lines().filter_map(parse_line).collect(),
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(TaskStore::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut body = self
            .tasks
            .iter()
            .map(format_line)
            .collect::<Vec<_>>()
            .join("\n");
        if !body.is_empty() {
            body.push('\n');
        }
        std::fs::write(path, body)
    }

    /// Active tasks (Open/Done) in on-disk order — what the Quick-TODO
    /// window shows. `Deleted` tasks are hidden here but kept in the store
    /// for `export_tsv`'s historical record.
    pub fn visible(&self) -> impl Iterator<Item = &Task> {
        self.tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Deleted)
    }

    /// Adds a new `Open` task and returns its id.
    pub fn add(&mut self, text: String) -> u64 {
        let id = self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        self.tasks.push(Task {
            id,
            status: TaskStatus::Open,
            created_at: now_unix_secs(),
            done_at: None,
            text: sanitize_text(&text),
        });
        id
    }

    /// Flips a task between `Open` and `Done`; a no-op on an id that
    /// doesn't exist or is `Deleted`.
    pub fn toggle_done(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = match task.status {
                TaskStatus::Open => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Open,
                TaskStatus::Deleted => return,
            };
            task.done_at = (task.status == TaskStatus::Done).then(now_unix_secs);
        }
    }

    /// Soft-deletes a task: marks it `Deleted` rather than removing it, so
    /// `export_tsv` still has a full historical record.
    pub fn delete(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Deleted;
        }
    }

    /// Moves the task with `id` one slot earlier (`delta < 0`) or later
    /// (`delta > 0`) among the *visible* tasks — organizing. A no-op if
    /// `id` isn't visible or the move would run past either end.
    pub fn move_visible(&mut self, id: u64, delta: i32) {
        let visible_indices: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status != TaskStatus::Deleted)
            .map(|(i, _)| i)
            .collect();
        let Some(pos) = visible_indices.iter().position(|&i| self.tasks[i].id == id) else {
            return;
        };
        let new_pos = pos as i32 + delta;
        if new_pos < 0 || new_pos as usize >= visible_indices.len() {
            return;
        }
        let a = visible_indices[pos];
        let b = visible_indices[new_pos as usize];
        self.tasks.swap(a, b);
    }

    /// Every task ever recorded — open, done, and deleted — for the
    /// `Ctrl+E` export.
    pub fn all(&self) -> &[Task] {
        &self.tasks
    }

    /// Renders the full historical record (including `Deleted` tasks) as
    /// TSV, header row included.
    pub fn export_tsv(&self) -> String {
        let mut out = String::from("id\tstatus\tcreated_at\tdone_at\ttext\n");
        for task in &self.tasks {
            out.push_str(&format_line(task));
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_assigns_increasing_ids_and_defaults_to_open() {
        let mut store = TaskStore::default();
        let a = store.add("first".to_string());
        let b = store.add("second".to_string());
        assert!(b > a);
        let visible: Vec<_> = store.visible().collect();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].status, TaskStatus::Open);
    }

    #[test]
    fn toggle_done_flips_status_and_sets_done_at() {
        let mut store = TaskStore::default();
        let id = store.add("task".to_string());
        store.toggle_done(id);
        let task = store.all().iter().find(|t| t.id == id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.done_at.is_some());
        store.toggle_done(id);
        let task = store.all().iter().find(|t| t.id == id).unwrap();
        assert_eq!(task.status, TaskStatus::Open);
        assert!(task.done_at.is_none());
    }

    #[test]
    fn delete_hides_from_visible_but_keeps_in_all() {
        let mut store = TaskStore::default();
        let id = store.add("task".to_string());
        store.delete(id);
        assert_eq!(store.visible().count(), 0);
        assert_eq!(store.all().len(), 1);
        assert_eq!(store.all()[0].status, TaskStatus::Deleted);
    }

    #[test]
    fn toggle_done_is_a_no_op_on_a_deleted_task() {
        let mut store = TaskStore::default();
        let id = store.add("task".to_string());
        store.delete(id);
        store.toggle_done(id);
        assert_eq!(store.all()[0].status, TaskStatus::Deleted);
    }

    #[test]
    fn move_visible_reorders_among_visible_tasks_only() {
        let mut store = TaskStore::default();
        let a = store.add("a".to_string());
        let b = store.add("b".to_string());
        let c = store.add("c".to_string());
        store.move_visible(c, -1);
        let order: Vec<u64> = store.visible().map(|t| t.id).collect();
        assert_eq!(order, vec![a, c, b]);
    }

    #[test]
    fn move_visible_is_a_no_op_past_either_end() {
        let mut store = TaskStore::default();
        let a = store.add("a".to_string());
        let b = store.add("b".to_string());
        store.move_visible(a, -1); // already first
        store.move_visible(b, 1); // already last
        let order: Vec<u64> = store.visible().map(|t| t.id).collect();
        assert_eq!(order, vec![a, b]);
    }

    #[test]
    fn save_and_load_round_trips_tasks_including_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.tsv");
        let mut store = TaskStore::default();
        let a = store.add("write the docs".to_string());
        let b = store.add("ship it".to_string());
        store.toggle_done(a);
        store.delete(b);
        store.save(&path).unwrap();

        let loaded = TaskStore::load(&path).unwrap();
        assert_eq!(loaded.all().len(), 2);
        assert_eq!(loaded.visible().count(), 1);
        let reloaded_a = loaded.all().iter().find(|t| t.id == a).unwrap();
        assert_eq!(reloaded_a.status, TaskStatus::Done);
        assert_eq!(reloaded_a.text, "write the docs");
    }

    #[test]
    fn load_missing_file_yields_an_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.tsv");
        let store = TaskStore::load(&path).unwrap();
        assert_eq!(store.all().len(), 0);
    }

    #[test]
    fn sanitize_text_strips_tabs_and_newlines() {
        let mut store = TaskStore::default();
        let id = store.add("multi\tline\ntext".to_string());
        assert_eq!(
            store.all().iter().find(|t| t.id == id).unwrap().text,
            "multi line text"
        );
    }

    #[test]
    fn export_tsv_includes_deleted_tasks_and_a_header() {
        let mut store = TaskStore::default();
        let id = store.add("gone".to_string());
        store.delete(id);
        let tsv = store.export_tsv();
        assert!(tsv.starts_with("id\tstatus\tcreated_at\tdone_at\ttext\n"));
        assert!(tsv.contains("deleted"));
        assert!(tsv.contains("gone"));
    }
}
