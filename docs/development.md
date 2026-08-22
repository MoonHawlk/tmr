# Development

### Quick setup

`./setup.sh` checks for a Rust toolchain (offering to install one via
`rustup` if missing), fetches dependencies, builds the release binary, and
optionally installs it onto your `PATH` and creates
`~/.config/tmr/config.toml` from the example — see `./setup.sh --help`.
`./debug.sh` is the day-to-day dev-loop companion: toolchain/config info,
`cargo check`/`clippy`/`fmt --check`/`test`, a debug build, and (with
`./debug.sh run [DIR]`) launching tmr itself with `RUST_BACKTRACE=full`
against `sandbox/` by default — see `./debug.sh --help`.

### Commands

```sh
cargo build --workspace       # build everything
cargo test --workspace        # run all unit tests
cargo clippy --workspace --all-targets
cargo fmt --all
```

If you're an AI agent (or onboarding one) working on this repo, see
[`.llm/`](../.llm/) — task-oriented docs on driving the TUI, the config/theme
schema, the crate architecture and data flow, how to extend it (widgets,
addons, formats), and known-limitation troubleshooting, written to be
read on demand rather than all at once.

## Tests

Unit tests live next to the code they cover (`#[cfg(test)]` modules), no
TUI initialization required — the engine, parser and rendering-to-lines
logic are all plain functions/structs testable in isolation:

- `tmr-core`: filesystem ops (create/save/delete/rename, size guard,
  workspace containment), config loading (missing/partial/invalid file)
  and settings persistence (`persist_settings`'s format-preserving
  partial-file writes), theme resolution, keymap parsing/overrides,
  search, the `App` engine's command dispatch (open/save/toggle-task/
  create/delete + addon/widget event fan-out).
- `tmr-markdown`: the Markdown parser (headings, lists, nested task lists,
  code blocks, blockquotes, tables, links, images, thematic breaks) and
  checkbox toggling (index-based, nested lists, no-trailing-newline files).
- `tmr-tui`: the built-in editor buffer (insert/delete/UTF-8/scrolling,
  Shift-selection extend/normalize/delete, select-all, cursor-row seeking,
  selected-text extraction), color parsing, Markdown-AST-to-terminal-lines
  rendering (task index tracking, image fallback, gutter width, the
  selection-highlight span-splitting logic, and the horizontal-scroll
  character-trimming logic), `UiState`'s horizontal-scroll clamping, the
  OSC 52 clipboard module's base64 encoder (RFC 4648 test vectors), and
  the `h` popup's query-filtering logic.

Run everything with `cargo test --workspace`.

[← Back to README](../README.md)
