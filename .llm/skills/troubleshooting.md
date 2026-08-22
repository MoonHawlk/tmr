---
name: troubleshooting
description: Diagnose things that look like bugs but are documented v1 behavior, plus real failure modes and how tmr handles them.
---

# Troubleshooting

## "The editor opened at the wrong spot"

Known limitation, not a bug: entering Edit mode always seeds the cursor
at `(row 0, col 0)` of the file (`crates/tui/src/editor.rs::Editor::new`),
regardless of which line you were viewing in the rendered document. There
is no mapping from "rendered line index" back to "raw source line
number" yet — rendered lines and source lines aren't 1:1 (a heading's
raw `#` marker is hidden and replaced with styling, a list item's
continuation lines are indented, a checkbox's `[ ]`/`[x]` becomes a
`☐`/`☑` glyph, etc.), so
this would need real bookkeeping to fix, not a one-line change. Documented
in the README roadmap.

## "Tab does different things in different modes"

By design: in Edit mode, `Tab` inserts `tab_width` spaces into the buffer
(`crates/tui/src/input.rs::handle_editor_key`) — it does *not* switch
focus, even though `Tab` is also the default `focus_files` binding.
Edit mode's key routing only checks the keymap for the `save` action
before falling through to raw editor key handling
(`crates/tui/src/input.rs::handle_key`'s `Mode::Edit` arm) — it never
does a general action lookup while editing, precisely so ordinary typing
(including `Tab`, `q`, `y`, etc.) can't accidentally trigger a global
command. Search/Prompt modes have the same property (see
`keybindings-and-workflows.md`).

## "Opening a `.txt` file doesn't show any Markdown styling"

Intentional, not a bug: the rich "Obsidian-style" rendering (hidden
syntax markers, heading hierarchy, checkbox glyphs, ...) is scoped to
`DocumentFormat::Markdown` only. Any other format —  including `.txt`
and unrecognized extensions — renders via
`markdown_view::render_plain_text`, which shows the file's raw content
line-for-line with no parsing at all; a literal `# heading` or
`- [ ] task` inside a `.txt` file stays exactly as typed. This dispatch
happens in `crates/tui/src/input.rs::refresh_rendered`, matching on
`doc.format`. If you want Markdown-*like* files with a different
extension to render richly, add that extension to
`DocumentFormat::from_path` (`crates/core/src/document.rs`) rather than
loosening the dispatch — see `extending-the-app.md`.

## "Images aren't rendering, just `[image: name.png]`"

Expected unless the terminal advertises truecolor.
`crates/tui/src/image_backend.rs::detect_capability` checks `COLORTERM`
(must contain `truecolor` or `24bit`) or `TERM` (must contain
`256color`, `kitty`, or `alacritty`) — nothing else. If your terminal
supports truecolor but doesn't set `COLORTERM`, export it yourself:
`COLORTERM=truecolor tmr ...`. Remote/`http(s)://` image URLs are never
fetched (no network I/O in tmr) and always show the placeholder — that's
intentional, not a capability-detection failure. A local image that fails
to decode (corrupt file, unsupported codec — only `png`/`jpeg` decoders
are compiled in, see `crates/tui/Cargo.toml`'s `image` dependency
features) also falls back to the placeholder rather than erroring.

## "The terminal is left in a weird state after a crash"

Shouldn't happen: `crates/tui/src/lib.rs::TerminalGuard` is a `Drop` impl
that disables raw mode and leaves the alternate screen unconditionally,
even during a panic unwind. If it does happen anyway (e.g. the process
was `kill -9`'d, which skips `Drop`), `reset` or `tput rmcp` in the shell
fixes it — that's an unavoidable limitation of any raw-mode TUI, not
specific to tmr.

## "I can't navigate above the directory I launched tmr in"

Intentional: `Workspace::guard` (`crates/core/src/workspace.rs`) rejects
any path that canonicalizes outside the workspace root, and
`crates/tui/src/input.rs`'s `nav_back` handler explicitly no-ops at the
root rather than attempting (and failing) to list the parent. This is the
"don't allow operations outside the workspace" safety requirement, not a
missing feature.

## "Delete doesn't work on a directory"

Intentional in v1: `crates/tui/src/input.rs`'s `delete` action handler
only opens the confirm dialog for `!entry.is_dir` entries. `fs_ops::
delete_file` itself also refuses non-regular-file targets (directories,
device files, ...) — see its `!meta.is_file()` check. Recursive directory
deletion was judged too risky for a keystroke-driven MVP.

## "A large file won't open"

Intentional guard, not a bug: `fs_ops::MAX_FILE_SIZE_BYTES` is 10 MB;
`read_file` returns `AppError::FileTooLarge` above that rather than
loading it (and potentially freezing the UI) — shown as a status-bar
error, not a crash.

## "My config/theme change didn't take effect"

Config, theme, and keymap are all resolved once at startup
(`src/main.rs`) and handed into `App::new` — there is no runtime
hot-reload in v1. Restart tmr after editing `config.toml` or a theme file.
(`reload`/`Ctrl+R` only re-reads the *directory listing*, not config.)

## "My config file has a typo and nothing seems to load"

By design, this never crashes tmr: `Config::load` catches a TOML parse
error, falls back to `Config::default()`, and returns a warning that
`main.rs` prints to stderr as `tmr: warning: ...` *before* the TUI takes
over the screen (so it's visible only if you're watching stderr — the TUI
itself doesn't currently surface config-load warnings inside the UI).
Same pattern for an invalid/missing theme file
(`Theme::resolve`). If tmr looks like it's ignoring your config, check
stderr for that warning line, and check you edited the right path — see
`configuration.md` for exactly where `dirs::config_dir()` resolves to on
your platform.

## "The Files pane disappeared / layout looks broken"

At very small terminal sizes, `ratatui`'s constraint solver
(`crates/tui/src/layout.rs::compute_panes`, `Percentage(25)` for Files vs
`Min(20)` for Document) can squeeze the Files pane to zero width rather
than panicking. Not a crash, just unusable below roughly 25-30 columns —
resize the terminal.
