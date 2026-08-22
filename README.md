# tmr

I'm the worst person to create good names... So, TMR ( Terminal Markdown Reader )

A fast, low-footprint terminal UI for browsing, reading and editing
Markdown notes — a small productivity engine for text in the terminal,
with Markdown as its first supported format.

```
tmr ~/notes
```

```
+ FILES ---------------------++ DOCUMENT ------------------------------------+
| sub/                       || # Welcome                                   |
| note1.md                   || This is a test note with emphasis, inline   |
| note2.md                   || code, and a link.                          |
|                            || ## Tasks                                    |
|                            || [ ] first task                              |
|                            || [x] second task                             |
+----------------------------++----------------------------------------------+
+ STATUS -----------------------------------------------------------------------+
| note1.md  tab focus · enter open/edit · space toggle · / search · q quit      |
+---------------------------------------------------------------------------------+
```

## Features (v1)

- Navigate directories and list `.md` files.
- Open, view, edit, save, create and delete files (delete asks to confirm).
- Markdown rendering: headings, paragraphs, lists, ordered lists, code
  spans/blocks, blockquotes, tables, links, bold/italic, thematic breaks,
  and images.
- Interactive task-list checkboxes (`- [ ]` / `- [x]`) — toggle with the
  cursor, persisted straight to the `.md` file.
- Filename search and in-document text search.
- Image rendering when the terminal supports truecolor (Unicode half-block
  approximation), with an elegant `[image: name.png]` fallback otherwise.
- Fully external configuration: theme colors, borders, keybindings,
  workspace default, addons/widgets to enable — nothing is hardcoded.
- Architecture prepared for more document formats, TUI widgets and addons,
  without those being fully built out in v1 (see Roadmap).

## Install / Build

Requires a Rust toolchain (stable, 2021 edition; developed against 1.98).

```sh
git clone <this repo> tmr
cd tmr
cargo build --release
# binary at target/release/tmr
```

Optionally put it on your `PATH`:

```sh
install -m 755 target/release/tmr ~/.local/bin/tmr
```

## Run

```sh
tmr              # opens the current working directory
tmr ~/notes       # opens a specific directory
tmr --help
```

If no directory is given, tmr uses `[workspace] default_dir` from
`config.toml` if set, otherwise the current working directory — the same
convention most terminal file/note browsers use.

## Keybindings

All of these are remappable — see [Configuration](#configuration). Defaults:

| Key         | Action                                            |
|-------------|----------------------------------------------------|
| `tab`       | Switch focus between Files and Document panes      |
| `up`/`down` | Move selection (files) or cursor (document)        |
| `right`/`enter` | Open selected entry (dir → enter it, file → open); in the document pane, `enter` starts editing |
| `left`      | Go to parent directory (Files pane)                |
| `space`     | Toggle the task checkbox under the cursor           |
| `ctrl+s`    | Save (in Edit mode)                                 |
| `esc`       | Cancel dialog / leave Edit mode                     |
| `/`         | Search (filenames if Files is focused, in-document text if Document is focused) |
| `ctrl+n`    | New file (prompts for a name)                       |
| `r`         | Rename selected file                                |
| `d`         | Delete selected file (asks to confirm with `y`)      |
| `ctrl+r`    | Reload the current directory listing                |
| `q`         | Quit                                                 |

## Configuration

tmr reads `~/.config/tmr/config.toml` (or `$XDG_CONFIG_HOME/tmr/config.toml`).
A missing file is not an error — every setting has a built-in default. See
[`config/config.example.toml`](config/config.example.toml) for a fully
annotated copy; copy it to get started:

```sh
mkdir -p ~/.config/tmr
cp config/config.example.toml ~/.config/tmr/config.toml
```

Key sections: `[workspace]` (default directory), `[theme]` (which palette
to use), `[ui]` (border style, hidden files), `[editor]` (tab width),
`[keys]` (any keybinding override), `[addons]` / `[widgets]` (which
compiled-in addons/widgets to enable by id).

### Themes

`[theme] name = "dark"` (the default) or `"light"` select a built-in
palette. Any other name is looked up at
`~/.config/tmr/themes/<name>.toml` — see
[`config/themes/dark.toml`](config/themes/dark.toml) and
[`config/themes/light.toml`](config/themes/light.toml) for the format
(plain `foreground`/`background`/`accent`/`border`/`muted`/`success`/
`warning`/`error` hex colors). Copy one, tweak the colors, and point
`[theme] name` at your new file's name — no rebuild needed.

## Architecture

```
tmr/                    binary crate: CLI parsing, config/theme loading,
                         wiring addons/widgets, handing off to tmr-tui.
crates/core/  (tmr-core) the engine: workspace/filesystem ops, the
                         Document model, config, theme, keymap, the
                         Command → App::dispatch → AppEvent flow, and the
                         Widget/Addon trait abstractions. No UI dependency.
crates/markdown/         (tmr-markdown) Markdown source → a renderer-
              (tmr-markdown) agnostic Block/Inline AST (pulldown-cmark
                         under the hood), plus in-place checkbox toggling
                         on raw source text. No UI dependency either.
crates/tui/   (tmr-tui)  ratatui/crossterm presentation layer: converts
                         the AST to styled terminal lines, owns the
                         interaction/dialog state machine, the built-in
                         editor, image rendering, and the event loop.
```

The core never imports ratatui or crossterm — every operation flows
`Key event → Command → App::dispatch → AppEvent → redraw`, matching the
brief: the engine owns state and operations, the TUI only owns
presentation and translates raw terminal events into that engine's
vocabulary. A different frontend could be built against `tmr-core` and
`tmr-markdown` without touching either crate.

**Widgets.** `tmr_core::widget::Widget` is a small trait (enable/disable,
configure, tick, receive events, render as plain text lines) that a side
panel in the TUI draws generically. One example ships (`ClockWidget`) to
prove the trait works end to end — enable it with `[widgets] enabled =
["clock"]`. Building a real widget (a quick TODO list, a calendar) means
implementing the trait and registering it in `main.rs`; no TUI changes
required.

**Addons.** `tmr_core::addon::Addon` is a trait (load hook, event hook,
optional status-bar text) with **no dynamic loading** in this version —
addons are Rust structs compiled into the binary and enabled via
`[addons] enabled = [...]`. Rust's ABI instability makes `.so`-based
plugins a poor fit for a v1; this trait is the seam a future dynamic- or
WASM-based loader could sit behind without changing how addons are
written. One example ships (`StatsAddon`, a session file-op counter).

**Formats.** `tmr_core::document::DocumentFormat` currently distinguishes
Markdown / plain text / unknown by extension; only Markdown is actually
rendered (via `tmr-markdown`). Adding TXT/JSON/YAML rendering later means
adding a sibling to the `tmr-markdown` crate and a dispatch arm in the
TUI's rendering — the core's document/save/open flow already doesn't care
what format it's holding.

## Roadmap / known limitations

- **No word-wrap**: long lines are clipped at the pane edge rather than
  wrapped, so the document cursor can address a stable line index for
  checkbox toggling. Word-wrap is a reasonable follow-up but needs a
  cursor model that survives reflow.
- **Editor starts at the top of the file**, not at the line you were
  viewing — the built-in editor is intentionally minimal (see
  `crates/tui/src/editor.rs`); an external-editor integration is a
  plausible alternative for users who want more.
- **Images**: only local (non-`http`) images are rendered, as
  colored-halfblock approximations, gated on detecting truecolor support
  via environment variables (no blocking terminal queries, to avoid any
  risk of hanging on an unusual terminal). Kitty/iTerm2/Sixel protocol
  support is a natural next backend behind the existing `ImageBackend`
  seam (`crates/tui/src/image_backend.rs`).
- **Search** is filename substring / in-document line substring only, in
  the current directory / current document — no recursive global search
  or indexing (deliberately: the brief calls for not indexing the whole
  workspace up front).
- **No file-system watching** — the listing refreshes on navigation and
  on the explicit `reload` action, not via polling or `inotify`.
- Widgets/addons are compiled-in only (see Architecture above).

## Development

```sh
cargo build --workspace       # build everything
cargo test --workspace        # run all unit tests
cargo clippy --workspace --all-targets
cargo fmt --all
```

## Tests

Unit tests live next to the code they cover (`#[cfg(test)]` modules), no
TUI initialization required — the engine, parser and rendering-to-lines
logic are all plain functions/structs testable in isolation:

- `tmr-core`: filesystem ops (create/save/delete/rename, size guard,
  workspace containment), config loading (missing/partial/invalid file),
  theme resolution, keymap parsing/overrides, search, the `App` engine's
  command dispatch (open/save/toggle-task/create/delete + addon/widget
  event fan-out).
- `tmr-markdown`: the Markdown parser (headings, lists, nested task lists,
  code blocks, blockquotes, tables, links, images, thematic breaks) and
  checkbox toggling (index-based, nested lists, no-trailing-newline files).
- `tmr-tui`: the built-in editor buffer (insert/delete/UTF-8/scrolling),
  color parsing, and Markdown-AST-to-terminal-lines rendering (task index
  tracking, image fallback).

Run everything with `cargo test --workspace`.
