# Changelog

All notable changes to this project are documented here. Loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This file is
updated as part of every change that touches user-facing behavior — see
the README's [TODO section](README.md#todo) for what's still open.

## [Unreleased]

### Added

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

### Fixed

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
