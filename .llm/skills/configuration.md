---
name: configuration
description: Read or write config.toml, a theme file, or remap a keybinding - schema, defaults, and where each setting is resolved in code.
---

# Configuration

## File location

`~/.config/tmr/config.toml`, or more precisely whatever
`dirs::config_dir()` resolves to on the current platform, joined with
`tmr/config.toml` (see `crates/core/src/config.rs::default_config_dir`).
On Linux that's `$XDG_CONFIG_HOME/tmr/config.toml`, falling back to
`~/.config/tmr/config.toml`.

A missing file is **not an error** — every field has a built-in default
(`Config::default()`, spread across each sub-struct's own `Default` impl).
A present-but-invalid file (bad TOML syntax, wrong types) also does not
crash tmr: `Config::load` catches the parse error, falls back to
`Config::default()`, and returns a warning string that `main.rs` prints
to stderr with a `tmr: warning:` prefix. A *partially* specified file
(only some sections/keys present) is fine — every struct has
`#[serde(default)]`, so unset keys just take their defaults.

The annotated template to copy is
[`config/config.example.toml`](../../config/config.example.toml).

Next to `config.toml`, in the same resolved config dir, lives
`tasks.tsv` — the Quick-TODO window's persistent task store
(`crates/core/src/tasks.rs::TaskStore`, path from
`config::default_tasks_path`). It's a separate plain-TSV file, not part
of `config.toml`'s schema below, since tasks are an application-level
concern independent of any one workspace. `ctrl+e` exports the full
task history to a sibling `tasks-export.tsv` in the same directory
(`config::default_tasks_export_path`), after a confirm dialog.

## Schema

```toml
[workspace]
default_dir = "~/notes"   # used only when tmr is launched with no path argument

[theme]
name = "dark"              # "dark", "light", or a name looked up in themes/<name>.toml

[ui]
border = "ascii"           # "ascii" | "rounded" | "double" | "none"
show_hidden = false        # include dotfiles in the Files pane
timer = false               # show a live UTC clock bar above the Files/Document panes

[editor]
tab_width = 4               # spaces inserted by Tab in the built-in editor

[keys]
quit = "q"                  # any action name -> key spec string; see below

[addons]
enabled = []                 # ids of compiled-in addons to activate, e.g. ["stats"]

[widgets]
enabled = []                 # ids of compiled-in widgets to activate, e.g. ["clock"]
```

Types live in `crates/core/src/config.rs`: `Config` is the root, with
`WorkspaceConfig`, `ThemeSelection`, `UiConfig` (+ `BorderStyle` enum),
`EditorConfig`, `AddonsConfig`, `WidgetsConfig` as fields, plus `keys:
HashMap<String, String>` directly (no wrapper struct — see
`crates/core/src/keymap.rs::Keymap`).

## Key specs (the `[keys]` table)

A key spec string is parsed by `crates/core/src/input.rs::parse_key`.
Grammar: zero or more of `ctrl` / `alt` / `shift` joined with `+`, then
exactly one of: a single character (`q`, `n`, ...), `space`, or a named
key (`enter`/`return`, `esc`/`escape`, `backspace`, `delete`/`del`, `tab`,
`up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`).
Examples: `"q"`, `"ctrl+s"`, `"ctrl+shift+n"`, `"space"`. An unparseable
spec is silently dropped (that action keeps no binding — it's simply
unreachable, not a crash) rather than failing config load.

Action names are open — `crates/core/src/keymap.rs::DEFAULTS` lists the
built-in ones tmr's own input handling recognizes
(`crates/tui/src/input.rs::handle_action`), but the `Keymap` type itself
will happily carry unrecognized action names too (relevant for an addon
that wants to claim its own binding in the future — see
`extending-the-app.md`).

To remap: put only the actions you want to change under `[keys]`; every
other action keeps its default (`Keymap::with_overrides` layers your
table on top of `Keymap::default()`, not a full replacement).

## Themes

`[theme] name = "dark"` (default), `"light"`, or `"grey"` select a
palette that's built into the binary — no file needed
(`crates/core/src/theme.rs::Theme::dark`/`light`/`light_grey`). `grey` is
a neutral, monochrome light palette — distinct from `light`, which is
blue/lavender-tinted — with only the semantic colors (success/warning/
error) keeping their hue. Any other name is looked up at
`<config_dir>/themes/<name>.toml`; if that file is missing or invalid,
tmr falls back to the built-in dark palette and prints a warning
(`Theme::resolve`).

Separately, the in-app Settings window (`s`, `crates/tui/src/settings.rs`)
lets a user switch between `Default` (whatever the above resolved to at
startup — snapshotted in `UiState::default_theme`), `Dark`, and
`Light (grey)` live, without hand-editing `config.toml` — see
`crates/tui/src/input.rs::handle_settings_key`. It mutates `App::theme`
directly (and `lib.rs::run_loop` recomputes the `Palette` every frame
instead of once, to pick it up) *and* writes the choice straight back to
`[theme] name` via `tmr_core::config::persist_settings` (a
`toml_edit::DocumentMut`-based, format-preserving edit — comments and
unrelated keys survive), so it's still in effect on the next launch. The
same function also persists the Settings window's other rows (`[ui]
border`/`line_indicator`/`timer`/`json_highlight`).

Theme file schema — flat key/value, all colors as `"#rrggbb"` hex or one
of a small set of ANSI names (`black red green yellow blue magenta cyan
white gray/grey darkgray/darkgrey`), parsed by
`crates/tui/src/theme.rs::parse_color` (unrecognized values fall back to
the terminal's default color, never panic):

```toml
name = "my-theme"
foreground = "#cdd6f4"
background = "#1e1e2e"
accent = "#89b4fa"      # headings, links, focused-pane border, prompts
border = "#585b70"      # unfocused pane borders, table rules, thematic breaks
muted = "#6c7086"       # code text, blockquote marker, hints
success = "#a6e3a1"     # checked task-list items, "Saved" status
warning = "#f9e2af"     # delete-confirmation prompt
error = "#f38ba8"       # error status messages
```

There is no `[theme]`-inline-colors form (the brief's early sketch showed
colors directly under `[theme]` in `config.toml`); the shipped design
resolved that ambiguity by keeping `config.toml`'s `[theme]` to just a
`name` selector and putting the palette itself in a separate theme file —
see `config/themes/dark.toml` / `light.toml` for working examples to copy.

Changing the active theme requires restarting tmr (no runtime hot-reload
in v1 — `Theme` is resolved once in `main.rs` and handed to `App::new`).
