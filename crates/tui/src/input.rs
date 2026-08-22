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
    // While actively editing, show the raw source one-line-per-line instead
    // of the Obsidian-style rendering: that keeps rendered line indices in
    // exact lockstep with `Editor`'s row indices (Markdown block rendering
    // doesn't preserve a 1:1 source-line mapping — see markdown_view's
    // module doc), which is what lets the terminal cursor and the
    // highlighted line track the *actual* typing position rather than
    // wherever the cursor happened to be when Edit mode was entered.
    let editing = matches!(ui.mode, Mode::Edit);
    ui.rendered = if editing {
        markdown_view::render_plain_text(&content, palette)
    } else {
        match doc.format {
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
        }
    };
    if editing {
        if let Some(editor) = &ui.editor {
            ui.doc_cursor = editor.cursor().0;
        }
    }
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
        Mode::Help { .. } => {
            handle_help_key(ui, key, &app.keymap);
        }
        Mode::Settings { .. } => {
            handle_settings_key(ui, app, key, image_cap, width);
        }
        Mode::Calendar { .. } => {
            handle_calendar_key(ui, key);
        }
        Mode::Edit => {
            let action = resolved_keymap.get(&key).map(String::as_str);
            if action == Some("save") {
                save_current(ui, app);
                refresh_rendered(ui, app, palette, image_cap, width);
            } else if key.code == KeyCode::Esc {
                ui.mode = Mode::Normal;
                refresh_rendered(ui, app, palette, image_cap, width);
            } else if action == Some("select_all") {
                if let Some(editor) = ui.editor.as_mut() {
                    editor.select_all();
                }
            } else if action == Some("copy") {
                copy_selection(ui);
            } else if action == Some("cut") {
                cut_selection(ui);
                refresh_rendered(ui, app, palette, image_cap, width);
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
                if let Some(row) = viewed_source_row(app, ui) {
                    if let Some(editor) = ui.editor.as_mut() {
                        editor.set_cursor_row(row);
                    }
                }
                ui.mode = Mode::Edit;
                refresh_rendered(ui, app, palette, image_cap, width);
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
        "save" if ui.focus == Focus::Document => {
            save_current(ui, app);
            refresh_rendered(ui, app, palette, image_cap, width);
        }
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
        "reload" => {
            let config_warning = reload_config(app, ui);
            let live_palette = Palette::from_theme(&app.theme);
            refresh_rendered(ui, app, &live_palette, image_cap, width);
            match app.dispatch(Command::Reload) {
                Ok(_) => match config_warning {
                    Some(w) => ui.set_status(
                        format!("Reloaded; config warning: {w}"),
                        StatusLevel::Warning,
                    ),
                    None => ui.set_status("Reloaded", StatusLevel::Info),
                },
                Err(e) => ui.set_status(e.to_string(), StatusLevel::Error),
            }
        }
        "help" => {
            ui.mode = Mode::Help {
                query: String::new(),
                selected: 0,
            };
        }
        "settings" => {
            ui.mode = Mode::Settings { row: 0 };
        }
        "calendar" => {
            ui.mode = Mode::Calendar { month_offset: 0 };
        }
        _ => {}
    }
    ControlFlow::Continue
}

fn tab_width_of(app: &App) -> usize {
    app.config.editor.tab_width
}

/// Re-reads `config.toml` from disk and re-applies everything it's safe to
/// change on a running session: theme, keymap overrides, the Settings
/// window's `Default`/line-indicator baseline, and (implicitly, since
/// `app.config` is read fresh every frame — see `ui.rs::draw`) border
/// style. Bundled into the same `ctrl+r` binding as the existing directory
/// refresh, so `show_hidden` picks up a config change too via that
/// re-list. Deliberately narrow: it does *not* touch the workspace
/// directory or which addons/widgets are registered — both startup-only
/// concerns this function has no business owning (there's no live
/// register/unregister path for either). Mirrors the same
/// load-config/resolve-theme/build-keymap sequence `main.rs` runs once at
/// startup. Returns the first warning `Config::load`/`Theme::resolve`
/// produced, if any (loading never hard-fails — see `Config::load`'s doc
/// comment — so the (possibly-defaulted) config is applied either way).
fn reload_config(app: &mut App, ui: &mut UiState) -> Option<String> {
    let load_result = tmr_core::config::Config::load(None);
    let config = load_result.config;
    let config_dir = tmr_core::config::default_config_dir();
    let (theme, theme_warning) =
        tmr_core::theme::Theme::resolve(config_dir.as_deref(), &config.theme.name);

    app.keymap = tmr_core::keymap::Keymap::with_overrides(config.keys.clone());
    ui.default_theme_name = config.theme.name.clone();
    ui.line_indicator =
        crate::state::LineIndicatorStyle::from_config_str(&config.ui.line_indicator);
    ui.theme_choice = crate::state::ThemeChoice::Default;
    ui.default_theme = theme.clone();
    app.theme = theme;
    app.config = config;

    load_result.warnings.into_iter().next().or(theme_warning)
}

/// `ui.doc_cursor` only maps 1:1 onto a raw source line for formats whose
/// Normal-mode view *is* the raw source, one line per `RenderedLine`
/// (`render_plain_text` — see `refresh_rendered`'s doc comment). Markdown's
/// Obsidian-style rendering doesn't preserve that mapping (headings,
/// blank-line handling, etc. can shift rows), so there's no correct row to
/// seed the editor with there yet; `None` leaves the editor's cursor where
/// it already was, matching prior behavior for that format.
fn viewed_source_row(app: &App, ui: &UiState) -> Option<usize> {
    match app.document()?.format {
        DocumentFormat::PlainText | DocumentFormat::Unknown => Some(ui.doc_cursor),
        DocumentFormat::Markdown => None,
    }
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

/// Copies the current selection (if any) to the system clipboard via OSC
/// 52 — see `crate::clipboard::set_clipboard`. A no-op, silent, if there's
/// no selection; a write failure surfaces as a status-bar error.
fn copy_selection(ui: &mut UiState) {
    let Some(text) = ui.editor.as_ref().and_then(|e| e.selected_text()) else {
        return;
    };
    match crate::clipboard::set_clipboard(&text) {
        Ok(()) => ui.set_status("Copied selection", StatusLevel::Success),
        Err(e) => ui.set_status(format!("Copy failed: {e}"), StatusLevel::Error),
    }
}

/// Like `copy_selection`, but also removes the selection from the buffer
/// — and only if the clipboard write actually succeeded, so a failed Cut
/// never silently destroys text the user couldn't retrieve.
fn cut_selection(ui: &mut UiState) {
    let Some(text) = ui.editor.as_ref().and_then(|e| e.selected_text()) else {
        return;
    };
    match crate::clipboard::set_clipboard(&text) {
        Ok(()) => {
            if let Some(editor) = ui.editor.as_mut() {
                editor.delete_selection();
            }
            ui.set_status("Cut selection", StatusLevel::Success);
        }
        Err(e) => ui.set_status(
            format!("Cut failed (nothing removed): {e}"),
            StatusLevel::Error,
        ),
    }
}

fn handle_editor_key(ui: &mut UiState, key: Key, tab_width: usize) {
    let Some(editor) = ui.editor.as_mut() else {
        return;
    };
    // Shift+navigation selects text, the same way a terminal readline
    // prompt or a plain-text CLI editor does: the first Shift+move sets an
    // anchor at the pre-move cursor position, further Shift+moves just
    // extend it. Anything else collapses the selection — except typing a
    // character or Backspace/Delete while one is active, which (also
    // matching normal editor behavior) replaces/removes the selected text
    // instead of acting on a single character.
    let is_nav = matches!(
        key.code,
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End
    );
    if key.shift && is_nav {
        editor.start_or_keep_selection();
    } else {
        match key.code {
            KeyCode::Backspace | KeyCode::Delete if editor.has_selection() => {
                editor.delete_selection();
                return;
            }
            KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab if editor.has_selection() => {
                editor.delete_selection();
            }
            _ => editor.clear_selection(),
        }
    }
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

fn handle_help_key(ui: &mut UiState, key: Key, keymap: &tmr_core::keymap::Keymap) {
    let Mode::Help { query, selected } = &mut ui.mode else {
        return;
    };
    match key.code {
        KeyCode::Esc => ui.mode = Mode::Normal,
        KeyCode::Up => *selected = selected.saturating_sub(1),
        KeyCode::Down => {
            let count = crate::help::visible_count(keymap, query);
            *selected = (*selected + 1).min(count.saturating_sub(1));
        }
        KeyCode::Char(c) => {
            query.push(c);
            *selected = 0;
        }
        KeyCode::Backspace => {
            query.pop();
            *selected = 0;
        }
        _ => {}
    }
}

/// Row 0 is the Theme picker, row 1 the Line-indicator picker. Changing the
/// theme takes effect immediately: it updates `app.theme` and re-renders
/// the cached document view (`ui.rendered` bakes in colors at render time,
/// unlike the borders/status-bar/popups, which read the palette live every
/// frame — see `lib.rs::run_loop`'s per-iteration `Palette::from_theme`).
fn handle_settings_key(
    ui: &mut UiState,
    app: &mut App,
    key: Key,
    image_cap: ImageCapability,
    width: u16,
) {
    let Mode::Settings { row } = std::mem::replace(&mut ui.mode, Mode::Normal) else {
        return;
    };
    match key.code {
        KeyCode::Esc => {}
        KeyCode::Up => {
            ui.mode = Mode::Settings {
                row: row.saturating_sub(1),
            };
        }
        KeyCode::Down => {
            ui.mode = Mode::Settings {
                row: (row + 1).min(1),
            };
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
            ui.mode = Mode::Settings { row };
            if row == 0 {
                ui.theme_choice = if key.code == KeyCode::Left {
                    ui.theme_choice.prev()
                } else {
                    ui.theme_choice.next()
                };
                app.theme = ui.theme_choice.resolve(&ui.default_theme);
                let live_palette = Palette::from_theme(&app.theme);
                refresh_rendered(ui, app, &live_palette, image_cap, width);
            } else {
                ui.line_indicator = ui.line_indicator.toggled();
            }
            persist_settings(ui);
        }
        _ => {
            ui.mode = Mode::Settings { row };
        }
    }
}

/// Writes the Settings window's current choices to `config.toml`, so they
/// survive a restart instead of resetting to whatever was configured at
/// startup. Best-effort: a write failure (no config dir resolvable, no
/// permission, etc.) just surfaces a status message — the choice still
/// applies live either way, it just won't persist this time.
fn persist_settings(ui: &mut UiState) {
    let Some(path) = tmr_core::config::default_config_path() else {
        return;
    };
    let theme_name = ui.theme_choice.persisted_name(&ui.default_theme_name);
    let indicator = ui.line_indicator.config_str();
    if let Err(e) = tmr_core::config::persist_settings(&path, &theme_name, indicator) {
        ui.set_status(
            format!("Settings applied but not saved: {e}"),
            StatusLevel::Error,
        );
    }
}

/// `left`/`right` moves `month_offset` to the adjacent month; any other key
/// leaves it unchanged. `esc` closes the window (handled implicitly: the
/// `std::mem::replace` below already defaults to `Mode::Normal`, and no
/// arm re-enters `Calendar` for it).
fn handle_calendar_key(ui: &mut UiState, key: Key) {
    let Mode::Calendar { month_offset } = std::mem::replace(&mut ui.mode, Mode::Normal) else {
        return;
    };
    match key.code {
        KeyCode::Esc => {}
        KeyCode::Left => {
            ui.mode = Mode::Calendar {
                month_offset: month_offset - 1,
            }
        }
        KeyCode::Right => {
            ui.mode = Mode::Calendar {
                month_offset: month_offset + 1,
            }
        }
        _ => ui.mode = Mode::Calendar { month_offset },
    }
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
