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
- **Confirm** — a yes/no gate before a destructive action (currently only
  delete).

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
| `y` | `confirm` | Confirms a pending delete (Confirm mode only) |
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
viewing — see `troubleshooting.md`). Type normally; arrow keys, `Home`/
`End`, `Backspace`/`Delete`, `Tab` (inserts `tab_width` spaces) all work.
`Ctrl+S` saves and returns to Normal mode; `Esc` returns to Normal mode
without saving (the buffer is *not* discarded — pressing `Enter` again
resumes editing with your unsaved changes still there).

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

**Navigate directories**: Files focused, `Enter` on a directory entry
(shown with a trailing `/`) enters it and resets selection to the top;
`←` goes back up one level (a no-op at the workspace root — you cannot
navigate above it, see `crates/core/src/workspace.rs::Workspace::guard`).
