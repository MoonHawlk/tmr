---
name: keybindings-and-workflows
description: Drive the running TUI - open/edit/save a file, toggle a checkbox, search, create/delete/rename, and how focus/modes work.
---

# Keybindings and workflows

All bindings below are the **defaults**; every one is remappable via
`config.toml`'s `[keys]` table (see `configuration.md`). The source of
truth for the default table is `crates/core/src/keymap.rs`'s `DEFAULTS`
constant — check there if this doc and the code ever disagree.

## Panes and focus

The screen has two focusable panes (`Tab` switches between them) plus a
status/command bar at the bottom:

- **Files** (left) — a directory listing: subdirectories first, then
  files, alphabetically, dotfiles hidden by default.
- **Document** (right) — the rendered Markdown of whichever file is open.

Whichever pane has focus determines what `up`/`down`/`enter`/`space`/`/`
do — see the table below.

## Modes

The app is a small state machine (`crates/tui/src/state.rs::Mode`):

- **Normal** — browsing/viewing; the default.
- **Edit** — the built-in text editor is active over the open document.
- **Search** — typing a filename or in-document search query.
- **Prompt** — typing a new filename or a rename target.
- **Confirm** — a yes/no gate before a destructive/notable action
  (currently file delete and the `ctrl+e` task export).
- **Help** — a visual-only, filterable command-reference popup, entered
  with `h` any time in Normal mode (Edit mode intercepts `h` as a literal
  character, so this never fires mid-edit — that's the *only* gate; it
  used to also require no document be open, which was a bug, see
  `CHANGELOG.md`). Typing filters the list; `Esc` closes it; nothing in
  this mode dispatches a `Command`.
- **Settings** — the interface-customization window, entered with `s`
  (available regardless of focus or whether a document is open — Edit
  mode intercepts `s` as a literal character the same way it does `h`).
  `Up`/`Down` moves between its five rows (Theme, Border, Line indicator,
  Timer bar, JSON highlighting); `Left`/`Right`/`Enter` cycles the
  highlighted row's value, applying it immediately (the Border, Timer and
  JSON rows flip `app.config.ui.border`/`.timer`/`.json_highlight`
  directly, since the TUI already reads those fields live every frame —
  no separate `UiState` mirror needed, unlike Theme/Line indicator); `Esc`
  closes. See `crates/tui/src/settings.rs` and
  `crates/tui/src/input.rs::handle_settings_key`.
- **Calendar** — a mini month-preview popup, entered with `Alt+C`. Shows a
  weekday-aligned grid for the current month with today's day highlighted;
  `Left`/`Right` moves to the adjacent month (tracked as `month_offset`,
  relative to the current month); `Esc` closes. Purely visual, nothing in
  this mode dispatches a `Command`. See `crates/tui/src/calendar.rs`,
  `tmr_core::datetime::CivilDate`, and
  `crates/tui/src/input.rs::handle_calendar_key`.
- **Todo** — the Quick-TODO window, entered with `Ctrl+T`, independent of
  any open document. `selected` indexes the *visible* (non-`Deleted`)
  tasks; `new_task: Option<String>` is `Some` while composing a new task's
  text (`Ctrl+N` starts composing, `Enter` submits a non-empty trimmed
  value via `Command::AddTask`, `Esc` cancels composing) and `None` while
  just navigating (`Up`/`Down` moves selection, `Shift+Up`/`Shift+Down`
  reorders via `Command::MoveTask`, `Space`/`Enter` toggles done via
  `Command::ToggleTaskDone`, `d` soft-deletes via `Command::DeleteTask`
  with no confirmation dialog — deliberately, to keep this "quick", and
  safe since deletion is soft; `Esc` closes the window). See
  `crates/tui/src/todo_view.rs`, `tmr_core::tasks::TaskStore`, and
  `crates/tui/src/input.rs::handle_todo_key`.

In **Search**/**Prompt** modes, every typed character is appended to the
buffer (not looked up in the keymap) — so typing `q` while naming a new
file inserts the letter `q`, it does not quit. Only `Enter` (submit) and
`Esc` (cancel) are special in these modes. **Confirm** mode is the
opposite: it does *not* accept free text, only the `confirm` action
(`y` by default) confirms, anything else cancels.

## Default keybindings

| Key | Action id | Effect |
|-----|-----------|--------|
| `Tab` | `focus_files` | Switch focus between Files and Document |
| `↑` / `↓` | `nav_up` / `nav_down` | Move selection (Files) or cursor (Document) |
| `→` or `Enter` | `nav_enter` / `edit` | Files: open the selected entry (directory → enter it, file → open it, focus moves to Document). Document: `Enter` starts Edit mode |
| `←` | `nav_back` | Files: go to the parent directory (stops at the workspace root) |
| `Space` | `toggle_task` | Document only: toggle the checkbox on the cursor's line, if it is a task-list item; persists to disk immediately |
| `Ctrl+S` | `save` | Document only, while in Edit mode: write the buffer to disk |
| `Esc` | `cancel` | Leave Edit mode (without discarding the in-memory buffer) / cancel a Search, Prompt or Confirm dialog |
| `/` | `search` | Files focused → filename search; Document focused → in-document text search |
| `Ctrl+N` | `new_file` | Prompt for a filename, create it (empty) in the current directory |
| `r` | `rename` | Files only: prompt to rename the selected entry |
| `d` | `delete` | Files only, non-directories: ask for confirmation, then delete |
| `Ctrl+R` | `reload` | Re-list the current directory from disk |
| `y` | `confirm` | Confirms a pending delete or task export (Confirm mode only) |
| `Shift+↑↓←→`/`Shift+Home`/`Shift+End` | *(none — handled inside Edit mode, not the action keymap)* | Edit mode only: extend a text selection from the cursor |
| `h` | `help` | Opens the command-reference popup |
| `s` | `settings` | Opens the Settings window (theme, border style, line indicator, timer bar, JSON highlighting) |
| `Alt+C` | `calendar` | Opens the Calendar window (mini month preview) |
| `Ctrl+T` | `todo` | Opens the Quick-TODO window |
| `Ctrl+E` | `export_tasks` | Exports all tasks (current + historical) to `.tsv`, after confirmation |
| `q` | `quit` | Exit tmr |

## Step-by-step workflows

**Open and read a note**: `Tab` until Files has focus (it does by
default on startup) → `↑`/`↓` to the file → `Enter`. Focus moves to
Document automatically.

**Toggle a task checkbox**: with a document open and Document focused,
`↑`/`↓` to the line showing `[ ]`/`[x]`, then `Space`. This writes the
file immediately — there is no separate "save" step for checkbox toggles.

**Edit and save**: Document focused, `Enter` to start editing (note: the
editor cursor starts at the top of the file, not at the line you were
viewing — see `troubleshooting.md`). While editing, the pane switches to
raw source text (not the Obsidian-style rendering) and a real terminal
cursor tracks your position, with `Ln X, Col Y` shown in the status bar
— see `crates/tui/src/input.rs::refresh_rendered` and
`crates/tui/src/ui.rs::draw`'s Edit-mode cursor placement. Type normally;
arrow keys, `Home`/`End`, `Backspace`/`Delete`, `Tab` (inserts
`tab_width` spaces) all work. `Ctrl+S` saves and returns to Normal mode;
`Esc` returns to Normal mode without saving (the buffer is *not*
discarded — pressing `Enter` again resumes editing with your unsaved
changes still there). Either way, the pane switches back to the
Obsidian-style rendering.

**Select text while editing**: hold `Shift` with `←`/`→`/`↑`/`↓`/`Home`/
`End` — the first Shift+move records an anchor at the pre-move cursor
position (`Editor::start_or_keep_selection`), later ones just move the
cursor and extend the highlighted range. `Backspace`/`Delete` deletes the
selection (`Editor::delete_selection`) instead of one character; typing
a character replaces it. Any key that isn't Shift+navigation collapses
the selection first. See `crates/tui/src/input.rs::handle_editor_key`
for the exact precedence, and
`crates/tui/src/widgets/document_view.rs::overlay_style` for how it's
rendered (splices the highlight into whatever spans are already there,
so it composes with the current-line indicator instead of fighting it).

**Look up a command**: any pane, `h` → type to filter → `Esc` to close.
Read-only; it doesn't run anything.

**Change a Settings-window value**: any pane, any time, `s` → `Up`/`Down`
to the row you want (Theme, Border, Line indicator, Timer bar, JSON
highlighting) → `Left`/`Right`/`Enter` to cycle its value → `Esc` to
close. Every row applies live *and* is written straight back to the
matching `[ui]`/`[theme]` key in `config.toml`, so the choice survives a
restart too — see `configuration.md`'s Themes section and
`tmr_core::config::persist_settings`.

**Create a note**: any pane, `Ctrl+N` → type a filename (e.g.
`idea.md`) → `Enter`. Created empty in the *current* directory (the one
the Files pane is showing, not necessarily the workspace root).

**Delete a note**: Files focused, select a file (not a directory — v1
only deletes files) → `d` → confirm with `y` (any other key cancels).

**Rename a note**: Files focused, select it → `r` → edit the pre-filled
name → `Enter`.

**Search filenames**: Files focused → `/` → type → `Enter`. Jumps
selection to the first match and reports the match count in the status
bar.

**Search inside the open document**: Document focused → `/` → type →
`Enter`. Jumps the cursor to the first matching line.

**Capture a quick task**: any pane, any time, `Ctrl+T` → `Ctrl+N` → type
the task text → `Enter`. No document needs to be open. `Space`/`Enter`
on a selected task toggles it done; `Shift+↑`/`Shift+↓` reorders it; `d`
deletes it (soft — see `tmr_core::tasks::TaskStore::delete`, no confirm
dialog); `Esc` closes the window.

**Export tasks**: any pane, any time, `Ctrl+E` → confirm with `y` (any
other key cancels). Writes every task ever recorded — open, done, and
soft-deleted — to `~/.config/tmr/tasks-export.tsv`.

**Navigate directories**: Files focused, `Enter` on a directory entry
(shown with a trailing `/`) enters it and resets selection to the top;
`←` goes back up one level (a no-op at the workspace root — you cannot
navigate above it, see `crates/core/src/workspace.rs::Workspace::guard`).
