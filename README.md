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
- Obsidian-style Markdown rendering, **for `.md` files only** — raw syntax
  markers (`#`, `**`, `` ` ``, `[]()`, `> `) are never shown; each element
  gets its own glyph, color and weight instead: heading levels get
  decreasing prominence (H1 bold+underlined+ruled, down to muted italic at
  H6), checkboxes render as `☐`/`☑`, nested bullets vary (`•`/`◦`/`▪`),
  blockquotes get a `▎` bar, code blocks a `▏` bar, inline code a padded
  pill, links are colored+underlined. Any other format (`.txt`, unknown
  extensions) is shown as plain, untouched text — no parsing at all.
- Interactive task-list checkboxes (`- [ ]` / `- [x]`) — toggle with the
  cursor, persisted straight to the `.md` file.
- A live cursor locator while editing: raw source view, auto-scroll, a
  real blinking terminal caret, and a `Ln X, Col Y` status readout, so
  you always know where a keystroke will land.
- Text selection in Edit mode: hold `shift` with the arrow/`home`/`end`
  keys to select a range, the same way a shell prompt or a plain-text CLI
  editor does. `Backspace`/`Delete` removes the selection; typing a
  character replaces it. Selected text is visually reverse-highlighted.
  `ctrl+a` selects the entire document the same way.
- Line numbers in the Document pane's gutter (both the normal rendered
  view and the raw Edit-mode view).
- A searchable command-reference popup (`h`).
- A Settings window (`s`) for live interface customization: pick a color
  theme (`Default` / `Dark` / `Light (grey)`) and how the current line is
  marked (full-line `Highlight` or a `Bar` gutter marker) — no restart
  needed.
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

### Try it risk-free: `sandbox/`

The repo ships a [`sandbox/`](sandbox/) directory just for trying tmr out
— open/edit/create/delete files there and it can't touch your real notes:

```sh
tmr sandbox
```

Only the example files it ships with are tracked in git; anything else
you create inside `sandbox/` while testing is gitignored, so experimenting
never dirties `git status`. See [`sandbox/README.md`](sandbox/README.md).

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
| `shift+arrows`/`shift+home`/`shift+end` | Edit mode only: select text from the cursor; `backspace`/`delete` removes it, typing replaces it |
| `ctrl+a`    | Edit mode only: select the entire document              |
| `h`         | Open the command-reference popup; type to filter, `esc` to close |
| `s`         | Open the Settings window (theme, line indicator); `up`/`down` select, `left`/`right`/`enter` change, `esc` close |
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

`[theme] name = "dark"` (the default), `"light"`, or `"grey"` select a
built-in palette — `grey` is a neutral, monochrome light theme, distinct
from `light`'s blue/lavender tint. Any other name is looked up at
`~/.config/tmr/themes/<name>.toml` — see
[`config/themes/dark.toml`](config/themes/dark.toml),
[`config/themes/light.toml`](config/themes/light.toml), and
[`config/themes/grey.toml`](config/themes/grey.toml) for the format
(plain `foreground`/`background`/`accent`/`border`/`muted`/`success`/
`warning`/`error` hex colors). Copy one, tweak the colors, and point
`[theme] name` at your new file's name — no rebuild needed. To try
`Dark`/`Light (grey)` without touching `config.toml`, use the in-app
Settings window (`s`) instead — see [Keybindings](#keybindings); that
switch is live but session-only (not saved to disk).

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

**Formats.** `tmr_core::document::DocumentFormat` distinguishes Markdown /
plain text / unknown by extension, and the TUI already dispatches on it
(`crates/tui/src/input.rs::refresh_rendered`): `.md` gets the full
Obsidian-style rendering via `tmr-markdown`, everything else falls
through to `markdown_view::render_plain_text` — untouched text, no
parsing. Adding a third rendered format (TXT/JSON/YAML with its own
styling, rather than the current plain-text catch-all) means adding a
sibling to the `tmr-markdown` crate and another arm in that same `match`
— the core's document/save/open flow already doesn't care what format
it's holding.

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

## TODO

Tracked here as a plain checklist (open it in tmr itself — `tmr .` from
the repo root and open `README.md` — to see the checkbox rendering these
items describe). Update this list, and [`CHANGELOG.md`](CHANGELOG.md),
as part of any change that adds/fixes user-facing behavior.

- [x] Obsidian-style Markdown rendering, `.md`-only (per-element glyphs,
      hidden raw syntax, heading hierarchy)
- [x] Plain-text rendering path for non-Markdown files
- [x] `[ui] show_hidden` config option actually wired to the file listing
- [x] Edit-mode cursor locator: raw source view, auto-scroll, a real
      terminal caret, and a `Ln X, Col Y` status readout
- [x] `h` command-reference popup (searchable)
- [x] Line numbers in the Document pane's gutter
- [x] `s` Settings window: live theme switching (Default/Dark/Light-grey)
      and a Highlight-vs-Bar current-line indicator
- [x] Shift+navigation text selection in Edit mode (select, then
      Backspace/Delete/type to remove or replace it)
- [x] Select-all (`Ctrl+A`)
- [ ] Copy/cut the current selection to the system clipboard (selection
      exists now, but there's no clipboard integration yet — `Ctrl+C`/
      `Ctrl+X` are unbound)
- [ ] Double-click/word-level selection
- [ ] Persist Settings-window choices (theme, line indicator) to
      `config.toml` — they currently apply live but are session-only,
      reset to `[theme] name`'s configured value and `Highlight` on
      restart
- [ ] Word-wrap for long lines (currently clipped — see Roadmap above)
- [ ] Sync the editor's opening cursor position to the line you were viewing
- [ ] Horizontal scroll for the Edit-mode cursor locator: a column past
      the pane's right edge is currently clamped to the last visible
      column rather than scrolling the view (same root cause as the
      no-word-wrap limitation above) — also affects how far a selection
      can be seen once it runs past the right edge
- [ ] Scrolling/keyboard navigation inside the `h` command-reference
      popup for terminals too short to fit every entry at once (it
      currently just clips silently, relying on the search filter to
      narrow the list instead)
- [ ] Kitty/iTerm2/Sixel image backends (currently half-block only)
- [ ] Recursive/global search across the workspace
- [ ] Config/theme hot-reload without restarting tmr
- [ ] Syntax highlighting inside fenced code blocks
- [ ] Mouse support (click to select/open, scroll, drag-to-select text)
- [ ] Undo/redo in the built-in editor
- [ ] Rendering support for a second document format (TXT/JSON/YAML), to
      exercise the `DocumentFormat` dispatch point beyond Markdown-vs-plain

## Development

```sh
cargo build --workspace       # build everything
cargo test --workspace        # run all unit tests
cargo clippy --workspace --all-targets
cargo fmt --all
```

If you're an AI agent (or onboarding one) working on this repo, see
[`.llm/`](.llm/) — task-oriented docs on driving the TUI, the config/theme
schema, the crate architecture and data flow, how to extend it (widgets,
addons, formats), and known-limitation troubleshooting, written to be
read on demand rather than all at once.

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
- `tmr-tui`: the built-in editor buffer (insert/delete/UTF-8/scrolling,
  Shift-selection extend/normalize/delete), color parsing, and Markdown-
  AST-to-terminal-lines rendering (task index tracking, image fallback,
  gutter width, and the selection-highlight span-splitting logic).

Run everything with `cargo test --workspace`.
