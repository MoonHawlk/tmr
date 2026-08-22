# Configuration

tmr reads `~/.config/tmr/config.toml` (or `$XDG_CONFIG_HOME/tmr/config.toml`).
A missing file is not an error — every setting has a built-in default. See
[`config/config.example.toml`](../config/config.example.toml) for a fully
annotated copy; copy it to get started:

```sh
mkdir -p ~/.config/tmr
cp config/config.example.toml ~/.config/tmr/config.toml
```

Key sections: `[workspace]` (default directory), `[theme]` (which palette
to use), `[ui]` (border style, hidden files, current-line indicator, the
`timer` bar, JSON highlighting),
`[editor]` (tab width), `[keys]` (any keybinding override), `[addons]` /
`[widgets]` (which compiled-in addons/widgets to enable by id).

The Quick-TODO window's tasks aren't part of `config.toml` — they're
their own file, `~/.config/tmr/tasks.tsv` (next to `config.toml`, since
tasks are an application-level concern independent of any one workspace).
`ctrl+e` exports the full history to a sibling `tasks-export.tsv` in the
same directory.

## Themes

`[theme] name = "dark"` (the default), `"light"`, or `"grey"` select a
built-in palette — `grey` is a neutral, monochrome light theme, distinct
from `light`'s blue/lavender tint. Any other name is looked up at
`~/.config/tmr/themes/<name>.toml` — see
[`config/themes/dark.toml`](../config/themes/dark.toml),
[`config/themes/light.toml`](../config/themes/light.toml), and
[`config/themes/grey.toml`](../config/themes/grey.toml) for the format
(plain `foreground`/`background`/`accent`/`border`/`muted`/`success`/
`warning`/`error` hex colors). Copy one, tweak the colors, and point
`[theme] name` at your new file's name — no rebuild needed. You can also
pick `Dark`/`Light (grey)` from the in-app Settings window (`s`) instead —
see [Keybindings](keybindings.md); that switch is live and, as of the
choice you make, also written straight back to `[theme] name` in
`config.toml` (along with `[ui] border`, `[ui] line_indicator`,
`[ui] timer` and `[ui] json_highlight`) — see
`tmr_core::config::persist_settings`, which edits just those keys via
`toml_edit` so the rest of your file (comments included) is left alone.

[← Back to README](../README.md)
