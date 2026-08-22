use std::collections::HashMap;
use std::path::PathBuf;

use tmr_core::app::App;
use tmr_core::command::Command;
use tmr_core::document::DocumentFormat;
use tmr_core::events::AppEvent;
use tmr_core::input::{Key, KeyCode};

use crate::editor::Editor;
use crate::image_backend::ImageCapability;
use crate::markdown_view;
use crate::state::{ConfirmAction, Focus, Mode, PromptKind, SearchScope, StatusLevel, UiState};
use crate::theme::Palette;

pub enum ControlFlow {
    Continue,
    Quit,
}

/// Re-renders the document pane from the live edit buffer (if a document is
/// open), so the view always reflects the latest in-memory state — including
/// unsaved edits — rather than only what's on disk.
pub fn refresh_rendered(
    ui: &mut UiState,
    app: &App,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) {
    let Some(doc) = app.document() else {
        ui.rendered.clear();
        return;
    };
    let content = ui
        .editor
        .as_ref()
        .map(|e| e.to_content())
        .unwrap_or_else(|| doc.content.clone());
    // The Obsidian-style rich rendering (hidden syntax markers, per-element
    // styling, checkbox glyphs, ...) is Markdown-specific — every other
    // format is shown as plain, unparsed text.
    ui.rendered = match doc.format {
        DocumentFormat::Markdown => {
            let base_dir = doc
                .path
                .parent()
                .unwrap_or(app.workspace().root())
                .to_path_buf();
            let blocks = tmr_markdown::parse(&content);
            markdown_view::render(&blocks, palette, image_cap, &base_dir, width)
        }
        DocumentFormat::PlainText | DocumentFormat::Unknown => {
            markdown_view::render_plain_text(&content, palette)
        }
    };
    ui.doc_cursor = ui.doc_cursor.min(ui.rendered.len().saturating_sub(1));
}

#[allow(clippy::too_many_arguments)]
pub fn handle_key(
    ui: &mut UiState,
    app: &mut App,
    resolved_keymap: &HashMap<Key, String>,
    key: Key,
    tab_width: usize,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) -> ControlFlow {
    match &mut ui.mode {
        Mode::Search { .. } | Mode::Prompt { .. } => {
            handle_text_entry_key(ui, app, key, palette, image_cap, width);
        }
        Mode::Confirm { .. } => {
            handle_confirm_key(ui, app, resolved_keymap, key);
        }
        Mode::Edit => {
            if resolved_keymap.get(&key).map(String::as_str) == Some("save") {
                save_current(ui, app);
            } else if key.code == KeyCode::Esc {
                ui.mode = Mode::Normal;
            } else {
                handle_editor_key(ui, key, tab_width);
                refresh_rendered(ui, app, palette, image_cap, width);
            }
        }
        Mode::Normal => {
            let Some(action) = resolved_keymap.get(&key).cloned() else {
                return ControlFlow::Continue;
            };
            return handle_action(ui, app, &action, palette, image_cap, width);
        }
    }
    ControlFlow::Continue
}

fn handle_action(
    ui: &mut UiState,
    app: &mut App,
    action: &str,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) -> ControlFlow {
    match action {
        "quit" => return ControlFlow::Quit,
        "focus_files" => {
            ui.focus = match ui.focus {
                Focus::Files => Focus::Document,
                Focus::Document => Focus::Files,
            };
        }
        "nav_up" => match ui.focus {
            Focus::Files => ui.selected = ui.selected.saturating_sub(1),
            Focus::Document => {
                ui.doc_cursor = ui.doc_cursor.saturating_sub(1);
            }
        },
        "nav_down" => match ui.focus {
            Focus::Files => {
                if !app.entries().is_empty() {
                    ui.selected = (ui.selected + 1).min(app.entries().len() - 1);
                }
            }
            Focus::Document => {
                if !ui.rendered.is_empty() {
                    ui.doc_cursor = (ui.doc_cursor + 1).min(ui.rendered.len() - 1);
                }
            }
        },
        "nav_enter" | "edit" if ui.focus == Focus::Files => {
            activate_selected(ui, app, palette, image_cap, width);
        }
        "edit" if ui.focus == Focus::Document => {
            if app.document().is_some() {
                ui.mode = Mode::Edit;
            }
        }
        "nav_back" => {
            if ui.focus == Focus::Files && app.current_dir() != app.workspace().root() {
                if let Some(parent) = app.current_dir().parent().map(PathBuf::from) {
                    if let Ok(AppEvent::DirectoryListed { .. }) =
                        app.dispatch(Command::ListDir(parent))
                    {
                        ui.selected = 0;
                    }
                }
            }
        }
        "toggle_task" if ui.focus == Focus::Document => {
            if let Some(idx) = ui.rendered.get(ui.doc_cursor).and_then(|l| l.task_index) {
                match app.dispatch(Command::ToggleTask(idx)) {
                    Ok(_) => {
                        resync_editor(ui, app, tab_width_of(app));
                        refresh_rendered(ui, app, palette, image_cap, width);
                    }
                    Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
                }
            }
        }
        "save" if ui.focus == Focus::Document => save_current(ui, app),
        "new_file" => {
            ui.mode = Mode::Prompt {
                kind: PromptKind::NewFile,
                buffer: String::new(),
            };
        }
        "rename" if ui.focus == Focus::Files => {
            if let Some(entry) = app.entries().get(ui.selected) {
                ui.mode = Mode::Prompt {
                    kind: PromptKind::Rename {
                        from: entry.path.clone(),
                    },
                    buffer: entry.name.clone(),
                };
            }
        }
        "delete" if ui.focus == Focus::Files => {
            if let Some(entry) = app.entries().get(ui.selected) {
                if !entry.is_dir {
                    ui.mode = Mode::Confirm {
                        message: format!("Delete {}? (y/n)", entry.name),
                        action: ConfirmAction::Delete {
                            path: entry.path.clone(),
                        },
                    };
                }
            }
        }
        "search" => {
            let scope = match ui.focus {
                Focus::Files => SearchScope::Files,
                Focus::Document => SearchScope::Document,
            };
            ui.mode = Mode::Search {
                scope,
                buffer: String::new(),
            };
        }
        "reload" => match app.dispatch(Command::Reload) {
            Ok(_) => ui.set_status("Reloaded", StatusLevel::Info),
            Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
        },
        _ => {}
    }
    ControlFlow::Continue
}

fn tab_width_of(app: &App) -> usize {
    app.config.editor.tab_width
}

fn activate_selected(
    ui: &mut UiState,
    app: &mut App,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) {
    let Some(entry) = app.entries().get(ui.selected).cloned() else {
        return;
    };
    if entry.is_dir {
        if app.dispatch(Command::ListDir(entry.path)).is_ok() {
            ui.selected = 0;
        }
        return;
    }
    match app.dispatch(Command::OpenFile(entry.path)) {
        Ok(_) => {
            let tab_width = tab_width_of(app);
            resync_editor(ui, app, tab_width);
            ui.focus = Focus::Document;
            ui.doc_cursor = 0;
            ui.doc_scroll = 0;
            refresh_rendered(ui, app, palette, image_cap, width);
            ui.clear_status();
        }
        Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
    }
}

fn resync_editor(ui: &mut UiState, app: &App, tab_width: usize) {
    if let Some(doc) = app.document() {
        ui.editor = Some(Editor::new(&doc.content, tab_width));
    } else {
        ui.editor = None;
    }
}

fn save_current(ui: &mut UiState, app: &mut App) {
    let Some(editor) = ui.editor.as_ref() else {
        return;
    };
    let content = editor.to_content();
    match app.dispatch(Command::Save(content)) {
        Ok(_) => {
            if let Some(e) = ui.editor.as_mut() {
                e.mark_saved();
            }
            ui.mode = Mode::Normal;
            ui.set_status("Saved", StatusLevel::Success);
        }
        Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
    }
}

fn handle_editor_key(ui: &mut UiState, key: Key, tab_width: usize) {
    let Some(editor) = ui.editor.as_mut() else {
        return;
    };
    match key.code {
        KeyCode::Char(c) => editor.insert_char(c),
        KeyCode::Enter => editor.insert_newline(),
        KeyCode::Backspace => editor.backspace(),
        KeyCode::Delete => editor.delete_forward(),
        KeyCode::Tab => {
            let _ = tab_width;
            editor.insert_tab();
        }
        KeyCode::Left => editor.move_left(),
        KeyCode::Right => editor.move_right(),
        KeyCode::Up => editor.move_up(),
        KeyCode::Down => editor.move_down(),
        KeyCode::Home => editor.move_home(),
        KeyCode::End => editor.move_end(),
        _ => {}
    }
}

fn handle_text_entry_key(
    ui: &mut UiState,
    app: &mut App,
    key: Key,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) {
    match key.code {
        KeyCode::Esc => {
            ui.mode = Mode::Normal;
            return;
        }
        KeyCode::Enter => {
            submit_text_entry(ui, app, palette, image_cap, width);
            return;
        }
        _ => {}
    }
    let buffer = match &mut ui.mode {
        Mode::Search { buffer, .. } | Mode::Prompt { buffer, .. } => buffer,
        _ => return,
    };
    match key.code {
        KeyCode::Char(c) => buffer.push(c),
        KeyCode::Backspace => {
            buffer.pop();
        }
        _ => {}
    }
}

fn submit_text_entry(
    ui: &mut UiState,
    app: &mut App,
    palette: &Palette,
    image_cap: ImageCapability,
    width: u16,
) {
    match std::mem::replace(&mut ui.mode, Mode::Normal) {
        Mode::Search { scope, buffer } => match scope {
            SearchScope::Files => match app.dispatch(Command::SearchFilenames(buffer)) {
                Ok(AppEvent::FilenameResults { matches, .. }) => {
                    if let Some(first) = matches.first() {
                        if let Some(idx) = app.entries().iter().position(|e| e.path == first.path) {
                            ui.selected = idx;
                        }
                    }
                    ui.set_status(format!("{} match(es)", matches.len()), StatusLevel::Info);
                }
                Ok(_) => {}
                Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
            },
            SearchScope::Document => match app.dispatch(Command::SearchInDocument(buffer)) {
                Ok(AppEvent::TextSearchResults { matches, .. }) => {
                    if let Some(first) = matches.first() {
                        ui.doc_cursor = first
                            .line_number
                            .saturating_sub(1)
                            .min(ui.rendered.len().saturating_sub(1));
                    }
                    ui.set_status(format!("{} match(es)", matches.len()), StatusLevel::Info);
                }
                Ok(_) => {}
                Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
            },
        },
        Mode::Prompt { kind, buffer } => {
            if buffer.trim().is_empty() {
                return;
            }
            match kind {
                PromptKind::NewFile => {
                    let path = app.current_dir().join(buffer.trim());
                    match app.dispatch(Command::CreateFile(path)) {
                        Ok(_) => ui.set_status("Created", StatusLevel::Success),
                        Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
                    }
                }
                PromptKind::Rename { from } => {
                    let to = app.current_dir().join(buffer.trim());
                    match app.dispatch(Command::RenameFile { from, to }) {
                        Ok(_) => ui.set_status("Renamed", StatusLevel::Success),
                        Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
                    }
                }
            }
        }
        other => ui.mode = other,
    }
    let _ = (palette, image_cap, width);
}

fn handle_confirm_key(
    ui: &mut UiState,
    app: &mut App,
    resolved_keymap: &HashMap<Key, String>,
    key: Key,
) {
    let action = resolved_keymap.get(&key).cloned();
    let Mode::Confirm {
        action: pending, ..
    } = std::mem::replace(&mut ui.mode, Mode::Normal)
    else {
        return;
    };
    if action.as_deref() == Some("confirm") {
        match pending {
            ConfirmAction::Delete { path } => match app.dispatch(Command::DeleteFile(path)) {
                Ok(_) => {
                    ui.selected = ui.selected.min(app.entries().len().saturating_sub(1));
                    ui.set_status("Deleted", StatusLevel::Success);
                }
                Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
            },
        }
    }
}
