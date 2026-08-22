---
name: architecture
description: Crate layout and the Key -> Command -> App::dispatch -> AppEvent -> render data flow - read before changing code.
---

# Architecture

## Crate map

```
tmr/                      binary crate (src/main.rs)
crates/core/    tmr-core    the engine — no ratatui/crossterm dependency
crates/markdown/ tmr-markdown  Markdown text -> AST — no UI dependency either
crates/tui/     tmr-tui     ratatui/crossterm presentation layer
```

Dependency direction: `tmr` (bin) depends on `tmr-core` and `tmr-tui`.
`tmr-tui` depends on `tmr-core` and `tmr-markdown`. `tmr-core` depends on
`tmr-markdown` (only for `tmr_markdown::checkbox::toggle`, in
`App::execute`'s `Command::ToggleTask` arm — see below). **Nothing
depends on `tmr-tui` except the binary.** That's the whole point: swap
the presentation layer, or add a second one, without touching the engine.

## The core data flow

Every user action follows one path, deliberately:

```
crossterm KeyEvent
  -> crates/tui/src/keymap.rs::to_core_key            (crossterm -> tmr_core::input::Key)
  -> crates/tui/src/input.rs::handle_key               (mode-aware dispatch)
  -> tmr_core::app::App::dispatch(Command)             (the only fs/document mutator)
       -> App::execute matches the Command, does the work, returns AppEvent
       -> fans AppEvent out to every registered Widget::on_event and Addon::on_event
  -> crates/tui/src/input.rs reacts to the Result<AppEvent, AppError>
       (updates UiState: status message, re-renders the document, etc.)
  -> crates/tui/src/ui.rs::draw                         (next frame)
```

`tmr-tui` never calls `std::fs` directly and never mutates a `Document`
directly — every filesystem/document operation is a `Command` variant
(`crates/core/src/command.rs`) dispatched through `App::dispatch`. If
you're adding a new user-facing operation, add a `Command` variant and an
`App::execute` arm before touching any TUI code.

## Where things live, concretely

**`tmr-core`** (`crates/core/src/`):
- `app.rs` — `App`, the engine: owns `Workspace`, current directory +
  cached entry listing, the open `Document`, `Config`, `Keymap`, `Theme`,
  registered widgets and the `AddonRegistry`. `App::dispatch` is the only
  public mutator.
- `command.rs` — `Command` enum (the full vocabulary of operations).
- `events.rs` — `AppEvent` enum (what happened, returned by `dispatch`).
- `document.rs` — `Document` (path + raw text + dirty flag) and
  `DocumentFormat` (currently `Markdown` / `PlainText` / `Unknown`,
  detected by extension — see `extending-the-app.md` for adding formats).
- `workspace.rs` — `Workspace` (canonicalized root + `guard()` path
  containment check + `list_dir()`).
- `fs_ops.rs` — the actual `std::fs` calls: `read_file` (with a 10 MB
  size guard), `save_file` (atomic: write to a sibling temp file, then
  rename), `create_file`, `delete_file` (refuses symlink targets other
  than removing the link itself, refuses non-regular files), `rename_file`.
- `search.rs` — `search_filenames` / `search_in_text`, both plain
  case-insensitive substring matching, no index.
- `config.rs`, `theme.rs`, `keymap.rs`, `input.rs` — see `configuration.md`.
- `widget.rs`, `addon.rs` — the extensibility traits, see
  `extending-the-app.md`.
- `error.rs` — `AppError` (`thiserror`), the one error type the whole
  engine returns.

**`tmr-markdown`** (`crates/markdown/src/`):
- `ast.rs` — `Block` / `Inline` / `ListItem` / `Alignment`: a renderer-
  agnostic tree. No styling, no ratatui types.
- `parser.rs` — `parse(source: &str) -> Vec<Block>`, built on
  `pulldown-cmark` 0.10. Implemented as a hand-rolled recursive-descent
  walk over pulldown's flat `Event` stream (not pulldown's own AST — it
  doesn't build one). See the doc comment on `parse_inlines` for the one
  subtle bit: distinguishing "an `End` event closed *my* container" from
  "an `End` event closed the *caller's* container" when a tight list item
  has no `Paragraph` wrapper around its text.
- `checkbox.rs` — `toggle(source, task_index) -> Option<String>`:
  flips the `task_index`-th `[ ]`/`[x]` found by a plain line scan (not a
  re-parse), preserving every other byte of the file exactly. Task
  indices are assigned in the same document-order the parser uses
  (`ListItem::task_index`), so "the 3rd checkbox in the rendered view" and
  "the 3rd checkbox `toggle` finds" are always the same checkbox.

**`tmr-tui`** (`crates/tui/src/`):
- `lib.rs` — `run(App)`: terminal setup/teardown (a `Drop` guard
  restores the terminal even on panic/early-return), the event loop.
  Blocks on `event::read()` when no widget needs periodic ticks; polls
  with a short timeout only when one does (see `troubleshooting.md` on
  CPU usage).
- `state.rs` — `UiState`: everything that is *not* engine state — focus,
  `Mode` (Normal/Edit/Search/Prompt/Confirm), cursor/scroll positions, the
  cached rendered document, the in-progress edit buffer. `tmr-core` knows
  nothing about any of this.
- `input.rs` — the mode-aware key handler described above.
- `editor.rs` — `Editor`, a minimal hand-rolled multi-line text buffer
  (no external editor-widget dependency) used only while in Edit mode.
- `markdown_view.rs` — `Block` tree -> `Vec<RenderedLine>` (one entry per
  displayed line, each optionally tagged with a `task_index`). Explicitly
  does **not** word-wrap — see the module doc comment for why (wrapping
  would decouple "line index" from "terminal width", which the
  task-toggle-by-cursor and scroll-follows-cursor logic both depend on).
- `image_backend.rs` — capability detection (env vars only, never a
  blocking terminal query) + a Unicode half-block renderer + text
  fallback. See `extending-the-app.md` for adding a Kitty/Sixel backend.
- `theme.rs` — hex/name string -> `ratatui::style::Color`.
- `keymap.rs` — `crossterm::event::KeyEvent` -> `tmr_core::input::Key`.
- `layout.rs` — `compute_panes` (the one place pane geometry is computed;
  both `ui.rs` for drawing and `lib.rs` for sizing the document view call
  it, so they can never disagree) and `styled_block` (border style ->
  ratatui `Block`).
- `widgets/` — `file_list.rs`, `document_view.rs`, `status_bar.rs` (the
  three always-visible panes) and `side_panel.rs` (renders enabled
  `tmr_core::widget::Widget`s; only appears when at least one is enabled).

## Why word-wrap is deliberately absent

This is the one design choice worth understanding before touching
`markdown_view.rs` or `input.rs`'s document-navigation code: every
`Block` renders to a fixed, width-independent number of lines. That means
"line 7 of the rendered document" means the same thing regardless of
terminal width, which is what lets checkbox-toggle-under-cursor and
scroll-follows-cursor be simple index arithmetic instead of tracking
positions through a reflow. Long lines are clipped at the pane edge
instead. If you add word-wrap, you need a different addressing scheme for
"which line has the cursor" first.
