# Keybindings

All of these are remappable — see [Configuration](configuration.md). Defaults:

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
| `ctrl+r`    | Reload the current directory listing, and re-read `config.toml` (theme, keymap, editor, ui settings) |
| `shift+arrows`/`shift+home`/`shift+end` | Edit mode only: select text from the cursor; `backspace`/`delete` removes it, typing replaces it |
| `ctrl+a`    | Edit mode only: select the entire document              |
| `ctrl+c`/`ctrl+x` | Edit mode only: copy/cut the current selection to the system clipboard |
| `h`         | Open the command-reference popup; type to filter, `up`/`down` to move the highlighted row, `esc` to close |
| `s`         | Open the Settings window (theme, border style, line indicator, timer bar, JSON highlighting); `up`/`down` select, `left`/`right`/`enter` change, `esc` close |
| `c`         | Open the Calendar window (mini month preview, today highlighted); `left`/`right` change month, `esc` close |
| `ctrl+t`    | Open the Quick-TODO window; `ctrl+n` new task, `space`/`enter` toggle done, `shift+↑`/`shift+↓` reorder, `d` delete, `esc` close |
| `ctrl+e`    | Export all tasks (current + historical) to `.tsv`, asking for confirmation first |
| `q`         | Quit                                                 |

[← Back to README](../README.md)
