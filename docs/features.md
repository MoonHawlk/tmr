# Features (v1)

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
  `ctrl+a` selects the entire document the same way. `ctrl+c`/`ctrl+x`
  copy/cut the current selection to the system clipboard via an OSC 52
  terminal escape sequence — no native clipboard dependency, and it works
  over SSH/tmux, where a native clipboard API has no path back to the
  desktop.
- Line numbers in the Document pane's gutter (both the normal rendered
  view and the raw Edit-mode view).
- A searchable command-reference popup (`h`), with `up`/`down` to move the
  highlighted row and auto-scroll on terminals too short to fit every
  entry at once.
- A Settings window (`s`) for live interface customization: pick a color
  theme (`Default` / `Dark` / `Light (grey)`), a pane border style
  (`ASCII` / `Rounded` / `Double` / `None`), how the current line is
  marked (full-line `Highlight` or a `Bar` gutter marker), whether the
  Timer bar is shown (`On`/`Off`), and whether `.json` files get syntax
  highlighting (`On`/`Off`) — no restart needed, and all five choices are
  persisted to `config.toml` as you change them, so they survive a
  restart too.
- An optional Timer bar (`[ui] timer = true`, or toggle it live from the
  Settings window): a thin strip at the very top of the TUI, above the
  Files/Document panes, showing the current time (UTC), updated live once
  a second. Off by default.
- A `"double"` border style (`[ui] border = "double"`) alongside the
  default `"ascii"`, `"rounded"`, and `"none"` — double box-drawing lines
  (`╔═╗`/`║`/`╚═╝`) for a denser, more application-panel look than the
  plain `+---+` terminal sketch.
- Optional JSON syntax highlighting (`[ui] json_highlight = true`, or
  toggle it live from the Settings window): `.json` files get keys,
  strings, numbers, and `true`/`false`/`null` colored distinctly instead
  of being shown as plain text. Off by default, so a `.json` file renders
  exactly as it always has unless you opt in — see
  `crates/tui/src/json_view.rs`.
- A Calendar window (`alt+c`): a small popup with a mini month-preview
  grid, aligned like a standard calendar (weekday columns, today's day
  highlighted). `left`/`right` moves to the adjacent month, `esc` closes.
- A Quick-TODO window (`ctrl+t`): a minimal task list, independent of any
  open document — create, check off, reorder and (soft-)delete simple
  tasks without navigating to or opening a Markdown file. `ctrl+n` starts
  a new task, `space`/`enter` toggles it done, `shift+↑`/`shift+↓`
  reorders it, `d` deletes it (recoverably — see below), `esc` closes.
  Tasks persist to `~/.config/tmr/tasks.tsv`, independent of the current
  workspace, so they're available across sessions and directories. A
  deleted task is soft-deleted (kept, marked `deleted`) rather than
  erased, so the full history stays available to `ctrl+e` (below) and to
  future features (search, filtering). See
  `crates/tui/src/todo_view.rs` and `tmr_core::tasks`.
- An application-level task export (`ctrl+e`, any time, regardless of
  focus): asks for confirmation, then writes every task ever recorded —
  open, done, and deleted — to `~/.config/tmr/tasks-export.tsv` as TSV
  with a header row.
- Filename search and in-document text search.
- Image rendering when the terminal supports truecolor (Unicode half-block
  approximation), with an elegant `[image: name.png]` fallback otherwise.
- Fully external configuration: theme colors, borders, keybindings,
  workspace default, addons/widgets to enable — nothing is hardcoded.
- Architecture prepared for more document formats, TUI widgets and addons,
  without those being fully built out in v1 (see [Roadmap](roadmap.md)).

[← Back to README](../README.md)
