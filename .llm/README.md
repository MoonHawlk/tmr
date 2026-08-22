# .llm — orientation for AI agents

This folder is documentation written specifically for an LLM/agent that
needs to work with the **tmr** repository — as a user driving the CLI, or
as a developer reading/modifying the code. It is not shipped in the
release binary and is not read by tmr itself; it exists purely as context.

Human-facing docs live in the top-level [`README.md`](../README.md). The
files here go deeper and are organized by task rather than by feature, so
an agent can jump straight to what it needs instead of reading the whole
codebase.

## How to use this folder

Read `skills/` files on demand, matched to what you're about to do — you
don't need all of them for every task:

| File | Read this when you need to... |
|------|-------------------------------|
| [`skills/quickstart.md`](skills/quickstart.md) | Build tmr and run it for the first time, including the disposable `sandbox/` playground |
| [`skills/keybindings-and-workflows.md`](skills/keybindings-and-workflows.md) | Drive the running TUI: open/edit/save a file, toggle a checkbox, search, create/delete/rename |
| [`skills/configuration.md`](skills/configuration.md) | Read or write `config.toml`, a theme file, or remap a keybinding |
| [`skills/architecture.md`](skills/architecture.md) | Understand the crate layout and the `Key → Command → App::dispatch → AppEvent → render` data flow before changing code |
| [`skills/extending-the-app.md`](skills/extending-the-app.md) | Add a widget, an addon, or a new document format |
| [`skills/development-workflow.md`](skills/development-workflow.md) | Build, test, lint, format, or add tests |
| [`skills/troubleshooting.md`](skills/troubleshooting.md) | Diagnose something that isn't working as expected |

## The one invariant to never break

`crates/core` (`tmr-core`) must never depend on `ratatui`, `crossterm`, or
any other presentation-layer crate. If a change you're making would add
such a dependency to `tmr-core` or `tmr-markdown`, the change belongs in
`crates/tui` instead. See `skills/architecture.md` for why this matters
and where the boundary actually sits.
