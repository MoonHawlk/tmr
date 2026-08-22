# Changelog

All notable changes to this project are documented here. Loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This file is
updated as part of every change that touches user-facing behavior — see
the README's [TODO section](README.md#todo) for what's still open.

## [Unreleased]

### Added

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
