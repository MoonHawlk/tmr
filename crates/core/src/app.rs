use std::path::{Path, PathBuf};

use crate::addon::AddonRegistry;
use crate::command::Command;
use crate::config::Config;
use crate::document::Document;
use crate::error::{AppError, Result};
use crate::events::AppEvent;
use crate::fs_ops;
use crate::keymap::Keymap;
use crate::search;
use crate::tasks::TaskStore;
use crate::theme::Theme;
use crate::widget::Widget;
use crate::workspace::{Entry, Workspace};

/// The engine: owns all application state and is the only thing allowed to
/// touch the filesystem or mutate the current document. Frontends drive it
/// exclusively through [`App::dispatch`] — see the module-level docs on
/// [`crate::command::Command`] for the intended flow.
pub struct App {
    workspace: Workspace,
    current_dir: PathBuf,
    entries: Vec<Entry>,
    document: Option<Document>,
    pub config: Config,
    pub keymap: Keymap,
    pub theme: Theme,
    widgets: Vec<Box<dyn Widget>>,
    addons: AddonRegistry,
    tasks: TaskStore,
    /// Where `persist_tasks` writes to; `None` means tasks stay
    /// session-only (no persistence). The caller resolves this — typically
    /// `tmr_core::config::default_tasks_path()`, mirroring how the
    /// workspace/config/theme are resolved by the caller too — rather than
    /// `App::new` reaching for the real filesystem itself, so constructing
    /// an `App` in a test never touches a real user's task file.
    tasks_path: Option<PathBuf>,
}

impl App {
    pub fn new(
        workspace: Workspace,
        config: Config,
        keymap: Keymap,
        theme: Theme,
        tasks_path: Option<PathBuf>,
    ) -> Self {
        let current_dir = workspace.root().to_path_buf();
        let tasks = tasks_path
            .as_deref()
            .and_then(|p| TaskStore::load(p).ok())
            .unwrap_or_default();
        App {
            workspace,
            current_dir,
            entries: Vec::new(),
            document: None,
            config,
            keymap,
            theme,
            widgets: Vec::new(),
            addons: AddonRegistry::new(),
            tasks,
            tasks_path,
        }
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn document(&self) -> Option<&Document> {
        self.document.as_ref()
    }

    pub fn tasks(&self) -> &TaskStore {
        &self.tasks
    }

    pub fn register_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    pub fn widgets(&self) -> &[Box<dyn Widget>] {
        &self.widgets
    }

    /// Advances every enabled widget by one tick (the frontend decides
    /// when/how often to call this, typically driven by each widget's
    /// `tick_interval`).
    pub fn tick_widgets(&mut self) {
        for widget in &mut self.widgets {
            if widget.is_enabled() {
                widget.tick();
            }
        }
    }

    pub fn register_addon(&mut self, addon: Box<dyn crate::addon::Addon>) {
        self.addons.register(addon);
    }

    pub fn load_addons(&mut self) {
        let ctx = crate::addon::AddonContext {
            workspace: &self.workspace,
        };
        self.addons.on_load(&ctx);
    }

    pub fn addon_status_texts(&self) -> Vec<String> {
        self.addons.status_texts()
    }

    /// The single entry point for performing an operation. Runs the
    /// command, then fans the resulting event out to widgets and addons.
    pub fn dispatch(&mut self, cmd: Command) -> Result<AppEvent> {
        let event = self.execute(cmd)?;
        for widget in &mut self.widgets {
            widget.on_event(&event);
        }
        self.addons.notify(&event);
        Ok(event)
    }

    fn execute(&mut self, cmd: Command) -> Result<AppEvent> {
        match cmd {
            Command::ListDir(dir) => {
                let entries = self.workspace.list_dir(&dir, self.config.ui.show_hidden)?;
                self.current_dir = self.workspace.guard(&dir)?;
                self.entries = entries.clone();
                Ok(AppEvent::DirectoryListed {
                    dir: self.current_dir.clone(),
                    entries,
                })
            }
            Command::OpenFile(path) => {
                let content = fs_ops::read_file(&self.workspace, &path)?;
                let guarded = self.workspace.guard(&path)?;
                self.document = Some(Document::new(guarded.clone(), content));
                Ok(AppEvent::DocumentOpened { path: guarded })
            }
            Command::Save(content) => {
                let doc = self.document.as_mut().ok_or(AppError::NoDocument)?;
                let path = fs_ops::save_file(&self.workspace, &doc.path, &content)?;
                doc.set_content(content);
                doc.mark_clean();
                Ok(AppEvent::DocumentSaved { path })
            }
            Command::CreateFile(path) => {
                let created = fs_ops::create_file(&self.workspace, &path)?;
                self.refresh_current_dir()?;
                Ok(AppEvent::FileCreated { path: created })
            }
            Command::DeleteFile(path) => {
                fs_ops::delete_file(&self.workspace, &path)?;
                let guarded = self
                    .workspace
                    .root()
                    .join(path.strip_prefix(self.workspace.root()).unwrap_or(&path));
                if self.document.as_ref().map(|d| &d.path) == Some(&guarded) {
                    self.document = None;
                }
                self.refresh_current_dir()?;
                Ok(AppEvent::FileDeleted { path: guarded })
            }
            Command::RenameFile { from, to } => {
                let dest = fs_ops::rename_file(&self.workspace, &from, &to)?;
                self.refresh_current_dir()?;
                Ok(AppEvent::FileRenamed { from, to: dest })
            }
            Command::ToggleTask(index) => {
                let doc = self.document.as_mut().ok_or(AppError::NoDocument)?;
                let updated = tmr_markdown::checkbox::toggle(&doc.content, index)
                    .ok_or(AppError::InvalidTaskIndex(index))?;
                fs_ops::save_file(&self.workspace, &doc.path, &updated)?;
                doc.set_content(updated);
                doc.mark_clean();
                Ok(AppEvent::TaskToggled { index })
            }
            Command::SearchFilenames(query) => {
                let matches: Vec<Entry> = search::search_filenames(&self.entries, &query)
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(AppEvent::FilenameResults { query, matches })
            }
            Command::SearchInDocument(query) => {
                let doc = self.document.as_ref().ok_or(AppError::NoDocument)?;
                let matches = search::search_in_text(&doc.content, &query);
                Ok(AppEvent::TextSearchResults { query, matches })
            }
            Command::Reload => {
                self.refresh_current_dir()?;
                Ok(AppEvent::Reloaded)
            }
            Command::AddTask(text) => {
                self.tasks.add(text);
                self.persist_tasks()?;
                Ok(AppEvent::TasksChanged)
            }
            Command::ToggleTaskDone(id) => {
                self.tasks.toggle_done(id);
                self.persist_tasks()?;
                Ok(AppEvent::TasksChanged)
            }
            Command::DeleteTask(id) => {
                self.tasks.delete(id);
                self.persist_tasks()?;
                Ok(AppEvent::TasksChanged)
            }
            Command::MoveTask { id, delta } => {
                self.tasks.move_visible(id, delta);
                self.persist_tasks()?;
                Ok(AppEvent::TasksChanged)
            }
            Command::ExportTasks(path) => {
                std::fs::write(&path, self.tasks.export_tsv())
                    .map_err(|e| AppError::from_io(&path, e))?;
                Ok(AppEvent::TasksExported { path })
            }
        }
    }

    /// Best-effort: if no config dir is resolvable (`tasks_path` is
    /// `None`), tasks just stay session-only rather than erroring every
    /// mutation.
    fn persist_tasks(&self) -> Result<()> {
        if let Some(path) = &self.tasks_path {
            self.tasks
                .save(path)
                .map_err(|e| AppError::from_io(path, e))?;
        }
        Ok(())
    }

    fn refresh_current_dir(&mut self) -> Result<()> {
        self.entries = self
            .workspace
            .list_dir(&self.current_dir, self.config.ui.show_hidden)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.md"), "# Hi\n\n- [ ] task\n").unwrap();
        let ws = Workspace::new(dir.path().to_path_buf()).unwrap();
        let app = App::new(
            ws,
            Config::default(),
            Keymap::default(),
            Theme::dark(),
            None,
        );
        (dir, app)
    }

    /// Like `make_app`, but with `tasks_path` pointed inside the same
    /// tempdir — for tests that exercise task persistence, so they never
    /// touch a real user's `tasks.tsv`.
    fn make_app_with_tasks() -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path().to_path_buf()).unwrap();
        let tasks_path = dir.path().join("tasks.tsv");
        let app = App::new(
            ws,
            Config::default(),
            Keymap::default(),
            Theme::dark(),
            Some(tasks_path),
        );
        (dir, app)
    }

    #[test]
    fn list_dir_updates_entries_and_current_dir() {
        let (dir, mut app) = make_app();
        let event = app
            .dispatch(Command::ListDir(dir.path().to_path_buf()))
            .unwrap();
        assert!(matches!(event, AppEvent::DirectoryListed { .. }));
        assert_eq!(app.entries().len(), 1);
    }

    #[test]
    fn open_save_roundtrip_updates_document() {
        let (dir, mut app) = make_app();
        app.dispatch(Command::OpenFile(dir.path().join("note.md")))
            .unwrap();
        assert_eq!(app.document().unwrap().content, "# Hi\n\n- [ ] task\n");
        app.dispatch(Command::Save("# Hi\n\nedited\n".to_string()))
            .unwrap();
        assert!(!app.document().unwrap().is_dirty());
        assert_eq!(
            fs_ops::read_file(app.workspace(), Path::new("note.md")).unwrap(),
            "# Hi\n\nedited\n"
        );
    }

    #[test]
    fn toggle_task_persists_to_disk() {
        let (dir, mut app) = make_app();
        app.dispatch(Command::OpenFile(dir.path().join("note.md")))
            .unwrap();
        app.dispatch(Command::ToggleTask(0)).unwrap();
        let on_disk = fs_ops::read_file(app.workspace(), Path::new("note.md")).unwrap();
        assert!(on_disk.contains("- [x] task"));
    }

    #[test]
    fn save_without_open_document_errors() {
        let (_dir, mut app) = make_app();
        let err = app.dispatch(Command::Save("x".into()));
        assert!(matches!(err, Err(AppError::NoDocument)));
    }

    #[test]
    fn create_then_delete_refreshes_listing() {
        let (dir, mut app) = make_app();
        app.dispatch(Command::ListDir(dir.path().to_path_buf()))
            .unwrap();
        app.dispatch(Command::CreateFile(dir.path().join("second.md")))
            .unwrap();
        assert_eq!(app.entries().len(), 2);
        app.dispatch(Command::DeleteFile(dir.path().join("second.md")))
            .unwrap();
        assert_eq!(app.entries().len(), 1);
    }

    #[test]
    fn addon_observes_document_opened_event() {
        let (dir, mut app) = make_app();
        app.register_addon(Box::new(crate::addon::StatsAddon::default()));
        app.load_addons();
        app.dispatch(Command::OpenFile(dir.path().join("note.md")))
            .unwrap();
        assert_eq!(
            app.addon_status_texts(),
            vec!["opened:1 saved:0 created:0 deleted:0"]
        );
    }

    #[test]
    fn add_task_appears_in_visible_tasks_and_persists() {
        let (_dir, mut app) = make_app_with_tasks();
        let event = app
            .dispatch(Command::AddTask("write the docs".to_string()))
            .unwrap();
        assert!(matches!(event, AppEvent::TasksChanged));
        let visible: Vec<_> = app.tasks().visible().collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].text, "write the docs");

        // A fresh App pointed at the same path picks up the persisted task.
        let tasks_path = app.tasks_path.clone().unwrap();
        let reloaded = crate::tasks::TaskStore::load(&tasks_path).unwrap();
        assert_eq!(reloaded.visible().count(), 1);
    }

    #[test]
    fn toggle_and_delete_task_round_trip_through_dispatch() {
        let (_dir, mut app) = make_app_with_tasks();
        app.dispatch(Command::AddTask("ship it".to_string()))
            .unwrap();
        let id = app.tasks().visible().next().unwrap().id;

        app.dispatch(Command::ToggleTaskDone(id)).unwrap();
        assert_eq!(
            app.tasks()
                .all()
                .iter()
                .find(|t| t.id == id)
                .unwrap()
                .status,
            crate::tasks::TaskStatus::Done
        );

        app.dispatch(Command::DeleteTask(id)).unwrap();
        assert_eq!(app.tasks().visible().count(), 0);
        assert_eq!(app.tasks().all().len(), 1); // soft-deleted, not erased
    }

    #[test]
    fn export_tasks_writes_a_tsv_file_with_a_header() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path().to_path_buf()).unwrap();
        let mut app = App::new(
            ws,
            Config::default(),
            Keymap::default(),
            Theme::dark(),
            None,
        );
        app.dispatch(Command::AddTask("exported task".to_string()))
            .unwrap();
        let export_path = dir.path().join("export.tsv");
        let event = app
            .dispatch(Command::ExportTasks(export_path.clone()))
            .unwrap();
        assert!(matches!(event, AppEvent::TasksExported { .. }));
        let contents = std::fs::read_to_string(&export_path).unwrap();
        assert!(contents.starts_with("id\tstatus\tcreated_at\tdone_at\ttext\n"));
        assert!(contents.contains("exported task"));
    }
}
