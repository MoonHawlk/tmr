---
name: development-workflow
description: Build, test, lint, format, and add tests - commands and conventions this codebase actually follows.
---

# Development workflow

## Commands

```sh
cargo build --workspace                    # build tmr-core, tmr-markdown, tmr-tui, tmr
cargo test --workspace                     # run every unit test in every crate
cargo clippy --workspace --all-targets     # lint, including test code
cargo fmt --all                             # format
cargo fmt --all -- --check                  # verify formatting without changing files
cargo build --release                       # optimized binary at target/release/tmr
```

All four (`build`, `test`, `clippy`, `fmt --check`) should be clean before
considering a change done — that was the bar used throughout this
project's initial implementation, and clippy runs with default lints (no
`#![allow]`-everything escape hatches at the crate level; a couple of
`#[allow(clippy::too_many_arguments)]` are on specific rendering/input
functions where splitting the signature into a context struct wasn't
judged worth the indirection — see `crates/tui/src/input.rs::handle_key`
and `crates/tui/src/markdown_view.rs::render_list`).

## If your environment has no Rust toolchain / no C linker

This came up during initial development in a minimal container: no
`cargo`, no `gcc`/`cc`, no passwordless `sudo`. If you hit "linker `cc`
not found":
1. Install Rust without root: `curl https://sh.rustup.rs | sh -s -- -y
   --profile minimal`, then `source $HOME/.cargo/env`.
2. If there's still no C compiler/linker available and you can't
   `apt install build-essential`, download a standalone
   [Zig](https://ziglang.org/download/) release (a plain tarball, no root
   needed) and point Cargo at `zig cc` as the linker via a wrapper script
   plus `~/.cargo/config.toml`:
   ```toml
   [target.x86_64-unknown-linux-gnu]
   linker = "/path/to/a/wrapper/script/that/execs/zig-cc-with-args"
   ```
   This is purely a local build-environment workaround, not something the
   repo depends on — a normal machine with `gcc`/`clang` (or `rustup`'s
   own defaults) needs none of this.

## Test layout and conventions

Tests are `#[cfg(test)] mod tests` blocks colocated with the code they
cover — no separate `tests/` integration-test directory, because nothing
here needs a live terminal or a running binary to test (see
`architecture.md`: the engine and the Markdown-to-lines conversion are
both plain functions/structs). `tempfile` is a dev-dependency of
`tmr-core` for filesystem tests; nothing else needs one.

When adding a test:
- Filesystem-touching core tests build an isolated `tempfile::tempdir()`
  workspace per test (see any `make_workspace()`/`make_app()` helper in
  `crates/core/src/*.rs`) — never assume a fixed path or touch the real
  filesystem outside a tempdir.
- Parser tests in `tmr-markdown` assert on the `Block`/`Inline` structure
  directly (pattern-match, don't stringify-and-compare) — see
  `crates/markdown/src/parser.rs`'s test module for the pattern.
- `tmr-tui` tests avoid touching the terminal entirely: `editor.rs`'s
  tests exercise the buffer directly, `markdown_view.rs`'s tests inspect
  the returned `Vec<RenderedLine>`'s `task_index` fields and flattened
  span text, not any rendered pixels/cells.

## Manual / interactive testing

There is no automated end-to-end TUI test suite (rendering correctness
was verified manually via a `tmux` pty during initial development — see
`quickstart.md`'s "driving it non-interactively" section for the
technique). For anything touching `crates/tui`, after the unit tests
pass, actually run it against `sandbox/` (see `../../sandbox/README.md`)
and drive the specific workflow you changed by hand or via `tmux
send-keys` — don't rely on unit tests alone to catch a rendering or
input-handling regression.

## Style conventions actually followed in this codebase

- No doc comments explaining *what* obviously-named code does; comments
  explain non-obvious *why* (a subtle invariant, a workaround, a design
  tradeoff) — see e.g. the comment on `parse_inlines` in
  `crates/markdown/src/parser.rs`, or the module doc on
  `crates/tui/src/markdown_view.rs` explaining why there's no word-wrap.
- Errors: `tmr-core` uses `thiserror`-derived `AppError`
  (`crates/core/src/error.rs`) as its one error type everywhere; the
  binary crate (`src/main.rs`) uses `anyhow::Context` to attach
  human-readable context at the top level. `tmr-tui` propagates
  `std::io::Result` for terminal setup/teardown failures only —
  everything else (a failed `Command`) becomes a status-bar message via
  `UiState::set_status`, never a panic or process exit.
- No `unwrap()`/`expect()` outside test code and a handful of internal
  invariants that are provably safe at that point (e.g. indexing a
  `Vec` right after checking `.is_empty()`); fallible I/O always returns
  `Result` up to a layer that can show the user something useful.
- Config/theme/keymap loading never hard-fails: a missing or invalid file
  falls back to built-in defaults and produces a warning string instead
  of an `Err` — see `Config::load`, `Theme::resolve`.
