# Changelog

All notable changes to this project are documented here. Loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This file is
updated as part of every change that touches user-facing behavior — see
[docs/todo.md](docs/todo.md) for what's still open.

## [Unreleased]

### Changed

- `README.md` cut down to a minimal landing page — the animated `docs/`
  SVG banner, a one-line pitch, the ASCII preview, install/build/run
  commands, and a links list — with everything else (Features,
  Keybindings, Configuration, Architecture, Roadmap, TODO, Development,
  Tests) moved into its own file under `docs/`. Each doc links back to
  the README and to related docs, so nothing that was previously in the
  README is gone, just relocated.
- `docs/architecture.md` gained a "How it works" section: a Mermaid
  flowchart of the runtime data flow — `tmr` binary startup (CLI → config/
  theme/keymap load → widget/addon registration → TUI start), the
  `tmr-tui` event loop (key event → `Command` → dispatch), `tmr-core`'s
  `App::dispatch` fan-out (workspace/document ops, Widget/Addon hooks →
  `AppEvent`), and the three `DocumentFormat` rendering paths (`.md` via
  `tmr-markdown`'s AST, `.json` via `json_view`'s tokenizer when
  `json_highlight` is on, everything else via `render_plain_text`) — as a
  visual companion to the existing prose/crate-table explanation.
- Licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE)
  (was MIT): free for any noncommercial purpose, commercial use requires
  a separate agreement. `Cargo.toml`'s `license` field now reads
  `PolyForm-Noncommercial-1.0.0` (a valid SPDX identifier) across the
  whole workspace via `[workspace.package]`.

### Added

- A Quick-TODO window (`ctrl+t`, new default binding): a minimal task
  list — create, check off, reorder, and delete simple tasks — usable
  without opening or navigating to a Markdown document. Tasks persist to
  `~/.config/tmr/tasks.tsv`, independent of the current workspace, via a
  new `tmr_core::tasks::TaskStore` reached through the existing
  `Command → App::dispatch → AppEvent` flow (`AddTask`/`ToggleTaskDone`/
  `DeleteTask`/`MoveTask`). Deletion is soft (marked `deleted`, kept on
  disk rather than erased) so the full history stays available for later
  search/filtering and for export. `App::new` now takes an extra
  `tasks_path: Option<PathBuf>` argument (resolved by the caller, e.g.
  `main.rs` via `tmr_core::config::default_tasks_path`) rather than
  reaching for the real filesystem itself, so constructing an `App` in a
  test never touches a real user's task file. See
  `crates/tui/src/todo_view.rs`, `crates/tui/src/input.rs::handle_todo_key`,
  and `crates/core/src/tasks.rs`.
- An application-level task export: `ctrl+e` (new default binding,
  available regardless of focus) asks for confirmation, then writes every
  task ever recorded — open, done, and deleted — to
  `~/.config/tmr/tasks-export.tsv` as TSV with a header row. New
  `Command::ExportTasks`/`AppEvent::TasksExported` and
  `ConfirmAction::ExportTasks`, reusing the same Confirm-dialog machinery
  the file-delete flow already has.
- Optional JSON syntax highlighting: `[ui] json_highlight = true` (or a
  new "JSON highlighting" row in the Settings window) colors `.json`
  files' keys, strings, numbers, and `true`/`false`/`null` distinctly
  instead of showing them as plain text — off by default, so a `.json`
  file's rendering is unchanged unless you opt in. `DocumentFormat`
  gained a `Json` variant; the highlighter itself
  (`crates/tui/src/json_view.rs`) is a self-contained, dependency-free
  line-local tokenizer rather than a `tmr-markdown`-style AST crate —
  JSON's styling doesn't depend on block structure across lines the way
  Markdown's does, so a per-line token scan (preserving the source
  exactly, never reformatting) is enough. Malformed JSON degrades to
  approximate coloring rather than an error, matching the Markdown
  parser's tolerant style.
- The Settings window (`s`) can now toggle the Timer bar on/off live, as a
  third row alongside Theme and Line indicator — `left`/`right`/`enter`
  flips it, and the choice persists to `[ui] timer` in `config.toml` the
  same way the other two rows already did. Unlike Theme/Line indicator,
  the toggle updates `app.config.ui.timer` directly rather than a
  `UiState` mirror, since the TUI already reads that field live every
  frame. See `crates/tui/src/settings.rs` and
  `crates/tui/src/input.rs::handle_settings_key`;
  `tmr_core::config::persist_settings` now writes `ui.timer` too.
- An optional Timer bar: `[ui] timer = true` draws a thin strip at the top
  of the TUI, above the Files/Document panes, with the current time (UTC),
  updating live once a second. Backed by a new dependency-free calendar/
  clock module, `tmr_core::datetime` (Howard Hinnant's civil-calendar
  algorithm), also intended for the upcoming Calendar window. See
  `crates/tui/src/widgets/timer_bar.rs` and `layout::compute_panes`'s new
  `timer` slot.
- `setup.sh`: quick environment setup — checks for a Rust toolchain
  (offers to install via `rustup` if missing), fetches dependencies,
  builds the release binary, and optionally installs it onto `PATH` and
  seeds `~/.config/tmr/config.toml` from the example config.
- `debug.sh`: a dev-loop inspection helper — toolchain/config info,
  `cargo check`/`clippy`/`fmt --check`/`test`, a debug build, and
  (`./debug.sh run [DIR]`) launching tmr with `RUST_BACKTRACE=full`
  against `sandbox/` by default.
- Config/theme hot-reload: `ctrl+r` (`reload`) now re-reads `config.toml`
  and re-applies theme, keymap overrides, editor tab width, and the
  Settings window's `Default`/line-indicator baseline — in addition to
  its existing directory-listing refresh — instead of requiring a
  restart to pick up a config edit. Mirrors the same load/resolve
  sequence `main.rs` runs once at startup; deliberately doesn't touch the
  workspace directory or addon/widget registration, both startup-only
  concerns with no live re-init path. Border style and `show_hidden` were
  already effectively live (read fresh every frame, or via the paired
  directory re-list) — this just makes that consistent for everything
  else `config.toml` controls. See `input.rs::reload_config`.
- Copy/cut the current Edit-mode selection to the system clipboard —
  `ctrl+c`/`ctrl+x` (new default bindings, `crates/core/src/keymap.rs`).
  Implemented via an OSC 52 terminal escape sequence rather than a native
  clipboard crate: no new dependency (just a small hand-rolled base64
  encoder), and it works over SSH/tmux, where a native clipboard API has
  no path back to the user's desktop. Cut only removes the selection once
  the clipboard write actually succeeds, so a failure never silently
  destroys text the user couldn't retrieve. See
  `crates/tui/src/clipboard.rs` and `Editor::selected_text`
  (`crates/tui/src/editor.rs`).
- The Settings window's theme and line-indicator choices are now
  persisted to `config.toml` as you change them (`[theme] name` and a new
  `[ui] line_indicator` key), instead of resetting to whatever was
  configured at startup every time tmr restarts. Writes go through a new
  `tmr_core::config::persist_settings`, which edits just those two keys
  in place via `toml_edit::DocumentMut` — a format-preserving parser, so
  the rest of the file (comments, ordering, unrelated keys) survives
  untouched, unlike this module's existing `toml`-crate-based struct
  loading. A write failure (e.g. no writable config dir) surfaces as a
  status-bar error but doesn't block the live in-memory change. See
  `crates/core/src/config.rs` and `input.rs::persist_settings`.
- Horizontal scroll for the Edit-mode raw-source view: a cursor column
  past the pane's right edge now scrolls the view to follow it, instead
  of clamping the terminal cursor to the last visible column while the
  underlying text stayed put. See `UiState::doc_hscroll`/
  `ensure_doc_hscroll` (`crates/tui/src/state.rs`) and
  `document_view::skip_chars` (`crates/tui/src/widgets/document_view.rs`),
  which trims the horizontally-scrolled-past characters after any
  selection overlay is applied.
- Entering Edit mode now seeds the editor's cursor at the line you were
  viewing in Normal mode, for plain-text/unknown documents — previously it
  always started at `(0, 0)`. Markdown documents are unchanged for now:
  the Obsidian-style rendering doesn't keep a 1:1 row mapping back to the
  raw source, so there's no correct row to seed yet. See
  `Editor::set_cursor_row` (`crates/tui/src/editor.rs`) and
  `input.rs::viewed_source_row`.
- The `h` command-reference popup now supports `up`/`down` to move a
  highlighted row, which auto-scrolls into view via ratatui's `ListState`
  on terminals too short to fit every entry — previously the list just
  clipped silently past the pane's bottom edge. See
  `crates/tui/src/help.rs`.
- `ctrl+a` selects the entire document in Edit mode (new default binding,
  `crates/core/src/keymap.rs`) — same selection mechanism as Shift+
  navigation, just anchored at the start and cursor at the end in one
  step. See `Editor::select_all` (`crates/tui/src/editor.rs`).
- Text selection in Edit mode: holding `shift` with an arrow key, `home`,
  or `end` selects a range from the cursor, the way a shell prompt or a
  plain-text CLI editor does — the first Shift+move sets an anchor,
  further Shift+moves extend it, and the selected text is reverse-
  highlighted. `Backspace`/`Delete` removes the selection instead of one
  character; typing a character replaces it. Any non-Shift key collapses
  the selection. See `Editor::selection_range`/`start_or_keep_selection`/
  `delete_selection` (`crates/tui/src/editor.rs`) and
  `crates/tui/src/widgets/document_view.rs::overlay_style` for the
  rendering side (splices the highlight into the existing per-line
  spans, so it composes correctly with the current-line indicator).
- Line numbers: the Document pane now shows a numbered gutter for every
  line, in both the normal (Obsidian-style) view and the raw-source Edit
  view. Gutter width grows with the document's line count.
- A Settings window, opened with `s` (new default binding). Lets you
  switch the color theme live — `Default` (whatever `config.toml`
  selected at startup), `Dark`, or `Light (grey)`, a new neutral/
  monochrome light palette (`Theme::light_grey`, distinct from the
  existing blue-tinted `light`) — and choose how the Document pane marks
  its current line: `Highlight` (the original full-line reverse-video) or
  `Bar`, a single `▏` marker in the gutter next to the line number,
  closer to a plain terminal cursor. `Up`/`Down` selects a row, `Left`/
  `Right`/`Enter` cycles its value, `Esc` closes. Both settings apply
  immediately, no restart — but are session-only for now, not persisted
  to `config.toml` (see the new `README.md` TODO item). See
  `crates/tui/src/settings.rs`.
- A "where am I typing" locator for Edit mode: the document pane now
  shows the raw source text (instead of the Obsidian-style rendering)
  while editing, so line numbers match the built-in editor 1:1; the
  current line auto-scrolls into view, a real terminal cursor blinks at
  the exact row/column, and the status bar shows `Ln X, Col Y`. Leaving
  Edit mode (`esc` or `ctrl+s`) switches the pane back to the normal
  rendered view.
- A command-reference popup, opened with `h` (new default binding,
  `crates/core/src/keymap.rs`). Shows every action, the key actually
  bound to it (honors `[keys]` overrides, not just the built-in
  defaults), and a one-line description, filterable by typing — `esc`
  closes it. Purely visual; it doesn't dispatch any `Command`. See
  `crates/tui/src/help.rs`.
- Obsidian-style Markdown rendering: raw syntax markers (`#`, `**`,
  `` ` ``, `[]()`, `> `) are no longer shown — each element gets its own
  glyph/color/weight. Heading levels 1–6 get decreasing visual
  prominence (H1: bold + underlined + a ruled line below; H6: muted
  italic). Task checkboxes render as `☐`/`☑`. Unordered bullets vary by
  nesting depth (`•` / `◦` / `▪`). Blockquotes get a `▎` bar, code
  blocks a `▏` bar. Inline code renders as a padded pill. Links are
  colored and underlined. This applies to `.md` files only.
- Plain-text rendering path (`markdown_view::render_plain_text`) for
  every other format — `.txt` and unrecognized extensions now show
  exactly what's on disk, with no Markdown parsing at all. Previously
  every opened file was parsed as Markdown regardless of extension.
- `CHANGELOG.md` (this file).
- A `## TODO` section in `README.md`.
- `.llm/` — task-oriented documentation for AI agents working on or with
  this project (quickstart, keybindings/workflows, configuration,
  architecture, extending the app, development workflow,
  troubleshooting).
- `sandbox/` — a playground directory for trying tmr without touching
  real notes (`tmr sandbox`), with example files and a real embedded
  test image. Only the shipped example files are tracked in git;
  anything else created inside `sandbox/` during testing is ignored.

- A Calendar window: `alt+c` (new default binding) opens a small popup
  with a mini month-preview grid, weekday-aligned like a standard
  calendar, with today's day highlighted. `left`/`right` moves to the
  adjacent month; `esc` closes it. Purely visual, like the `h`/`s`
  windows — nothing here dispatches a `Command`. See
  `crates/tui/src/calendar.rs` and `tmr_core::datetime::CivilDate`.
- A `"double"` border style: `[ui] border = "double"` draws panes with
  double box-drawing lines (`╔═╗`/`║`/`╚═╝`) instead of the plain-ASCII
  `+---+` default — a denser, more application-panel look. See
  `BorderStyle::Double` (`crates/core/src/config.rs`) and
  `layout::DOUBLE_BORDER`/`styled_block`.

### Fixed

- Border styles (Ascii/Rounded/Double/None) were only reachable by editing
  `config.toml` directly — the Settings window had no row for them. Added
  a "Border" row (between Theme and Line indicator), cycled live with
  `left`/`right`/`enter` the same way Theme is, and persisted to
  `[ui] border`. `BorderStyle` gained `ALL`/`label`/`next`/`prev`/
  `config_str` helpers in `crates/core/src/config.rs`, mirroring
  `ThemeChoice`'s existing shape; `persist_settings` now writes `ui.border`
  too.
- Status bar no longer gets stuck on a transient message (e.g. "Saved",
  "Deleted") until the next command — it now reverts to the default helper
  bar on its own after a few seconds. `UiState::status` carries the
  `Instant` it was set, `UiState::expire_status` clears it once stale, and
  `lib.rs::run_loop` polls at a short interval while a status is pending so
  the revert happens close to on time even when idle. See
  `crates/tui/src/state.rs`.
- The `h` command-reference popup stopped opening for the rest of the
  session as soon as any file was opened once — it was gated to only
  fire when `app.document().is_none()`, and there's no "close document"
  action, so the gate could never turn back off. `h` now always works in
  Normal mode, matching the ungated `s` Settings binding.
- `[ui] show_hidden` was defined in `config.toml`'s schema but had no
  effect — `Workspace::list_dir` now takes and honors it.

## [0.1.0] - 2026-08-22

Initial implementation.

### Added

- Core engine (`tmr-core`): workspace/filesystem operations (list, open,
  save — atomic write, create, delete, rename), a `Document` model, the
  `Command → App::dispatch → AppEvent` flow, external TOML configuration
  (theme selection, UI/editor/keys/addons/widgets), a UI-independent
  keymap and key-parsing layer, filename and in-document search, and the
  `Widget`/`Addon` extension-point traits (with `ClockWidget` and
  `StatsAddon` as minimal working examples).
- Markdown parser (`tmr-markdown`): source text → a renderer-agnostic
  `Block`/`Inline` AST built on `pulldown-cmark`, supporting headings,
  paragraphs, ordered/unordered lists, task lists (with document-order
  indices), code blocks, blockquotes, tables, links, images, bold/
  italic/strikethrough, inline code, and thematic breaks; plus in-place,
  index-addressed checkbox toggling on raw source text.
- Terminal UI (`tmr-tui`, ratatui/crossterm): a Files/Document/Status
  three-pane layout with ASCII (or rounded/none) borders, keyboard-only
  navigation, a built-in multi-line text editor, filename/in-document
  search dialogs, create/rename/delete (with confirmation) flows,
  interactive checkbox toggling, and image rendering via a Unicode
  half-block renderer with environment-based truecolor capability
  detection and a text-placeholder fallback.
- CLI (`tmr [directory]`, `tmr --help`) and installable release binary.
- Unit test coverage across all three crates (filesystem ops, config/
  theme/keymap loading and parsing, the parser, checkbox toggling, the
  built-in editor buffer, and the app engine's command dispatch).
