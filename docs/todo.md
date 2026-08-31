# TODO

Tracked here as a plain checklist. Items are grouped by priority and
purpose rather than implementation order.

Update this list, and `CHANGELOG.md`, as part of any change that adds or
fixes user-facing behavior.

### Bugs / Fixes

- [x] (BUG) Make the newly available border styles selectable from the Settings window.
      — new "Border" row (ASCII/Rounded/Double/None), cycled with `left`/`right`, applied live
      and persisted to `[ui] border` (`crates/tui/src/settings.rs`,
      `crates/tui/src/input.rs::handle_settings_key`).

- [x] (BUG) `tmr` crashed instantly on macOS with "terminal UI exited with
      an error / Invalid argument (os error 22)", every launch.
      — root cause: `crates/tui/src/lib.rs::run_loop` polled with a
      `Duration::from_secs(u64::MAX / 2)` sentinel to mean "block
      indefinitely". Crossterm forwards that duration into a kqueue
      `timespec`; macOS's kernel rejects the resulting near-`i64::MAX`
      timeout with EINVAL when computing the deadline (Linux's epoll takes
      a 32-bit millisecond timeout, so the same sentinel just saturates
      there instead of erroring — that's why it only broke on macOS).
      Fixed by using a real `Option<Duration>` and calling `event::read()`
      directly (skipping `event::poll` entirely) when there's nothing to
      poll for, instead of passing a giant duration through `poll`.

### Core UX

- [ ] Double-click/word-level selection
- [ ] Word-wrap for long lines (currently clipped — see [Roadmap](roadmap.md))
- [ ] Undo/redo in the built-in editor
- [ ] Mouse support (click to select/open, scroll, drag-to-select text)

### CLI

- [ ] `tmr --version` / `-V` — print the crate version and exit, same as
      `--help` does today. `src/main.rs::parse_args` currently only
      recognizes `-h`/`--help`; any other flag is treated as a directory
      argument.

### Search

- [ ] Recursive/global search across the workspace

### Markdown

- [ ] Syntax highlighting inside fenced code blocks

### Images

- [ ] Kitty/iTerm2/Sixel image backends (currently half-block only)

### Formats

- [x] Rendering support for a second document format (TXT/JSON/YAML), to
      exercise the `DocumentFormat` dispatch point beyond Markdown-vs-plain
      ONLY ALLOW THIS IF THE USED SET AS POSSIBLE AT CONFIG WINDOW
      — JSON: `[ui] json_highlight`, off by default, toggle in the Settings window or config.toml
      (`crates/tui/src/json_view.rs`, `DocumentFormat::Json`). TXT/YAML remain open.

### Productivity

- [x] Quick-TODO window backed by a persistent task file.

      The Quick-TODO should provide a minimal interface focused on creating,
      checking and organizing simple tasks without requiring a Markdown
      document to be opened.

      Tasks should be stored persistently so they can later be searched,
      filtered and reused by other TMR features.

      Add an application-level export action (`Ctrl+E`) that asks for
      confirmation and exports the current and historical tasks to `.tsv`.

      — `ctrl+t` opens a minimal task list (create/check/reorder/delete), independent of any
      open document. Tasks persist to `~/.config/tmr/tasks.tsv` (TSV, one task per line: id,
      status, created_at, done_at, text) via `tmr_core::tasks::TaskStore`, reachable through the
      existing `Command → App::dispatch → AppEvent` flow like everything else. Deletion is soft
      (marked `deleted`, kept on disk) so the full history stays available for later search/
      filtering and for `ctrl+e`'s export, which writes every task ever recorded to
      `~/.config/tmr/tasks-export.tsv` after a confirm dialog. See `crates/tui/src/todo_view.rs`,
      `crates/tui/src/input.rs::handle_todo_key`, and `tmr_core::tasks`.

### Customization

- [ ] Expand the configuration system with additional TUI customization,
      including ASCII icons, font-related terminal options, highlights and
      other visual properties.

      All supported options should remain configurable without modifying
      the source code and, where appropriate, should be exposed through
      the Settings window.

### Roadmap / Product Direction

- [x] Add a dedicated section to the README describing future features
      and the long-term purpose of TMR.

      This section should distinguish implemented functionality, planned
      functionality and experimental ideas, keeping the project's scope
      clear as it grows.

[← Back to README](../README.md)
