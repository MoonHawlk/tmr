---
name: extending-the-app
description: Add a widget, an addon, or a new document format - concrete steps and file pointers for each.
---

# Extending tmr

Read `architecture.md` first if you haven't — this assumes you know the
crate layout and the `Command -> App::dispatch -> AppEvent` flow.

## Adding a widget

A widget is a small, optional side-panel element (a clock, a quick TODO
list, ...). The trait is `tmr_core::widget::Widget`
(`crates/core/src/widget.rs`):

```rust
pub trait Widget {
    fn id(&self) -> &str;                              // config id, e.g. "clock"
    fn title(&self) -> &str;                            // shown in the panel's border
    fn configure(&mut self, _config: &toml::Value) {}    // optional [widgets.<id>] table
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn tick_interval(&self) -> Option<Duration> { None }  // Some(d) to get periodic ticks
    fn tick(&mut self) {}
    fn on_event(&mut self, _event: &AppEvent) {}          // react to engine events
    fn render_lines(&self) -> Vec<String>;                // plain text — no ratatui type here
}
```

Note `render_lines` returns plain `String`s, not styled ratatui text —
that's what keeps `tmr-core` free of a ratatui dependency. The TUI side
(`crates/tui/src/widgets/side_panel.rs`) draws whatever lines you return
inside a generically-styled bordered box.

Steps:
1. Implement `Widget` for a new struct (see `ClockWidget` at the bottom
   of `crates/core/src/widget.rs` as the reference example — it's
   deliberately trivial, just enough to prove the trait works).
2. In `src/main.rs`, register an instance behind a config check, following
   the existing `clock` pattern:
   ```rust
   if widgets_enabled.iter().any(|w| w == "my-widget") {
       let mut w = MyWidget::default();
       w.set_enabled(true);
       app.register_widget(Box::new(w));
   }
   ```
3. Tell users to add `"my-widget"` to `[widgets] enabled = [...]` in
   `config.toml`.
4. The side panel only appears in the layout at all when at least one
   registered widget is enabled (`crates/tui/src/ui.rs`, `has_widgets`) —
   you don't need to touch layout code.
5. If your widget needs periodic redraws (like the clock), return
   `Some(duration)` from `tick_interval`; the event loop
   (`crates/tui/src/lib.rs::run_loop`) automatically switches from
   blocking-read to short-poll mode when any enabled widget wants ticks,
   and calls `App::tick_widgets()` on timeout.

## Adding an addon

An addon observes engine events and optionally contributes a status-bar
string; there is deliberately **no dynamic loading** (`.so`/plugin ABI)
in this version (see the module doc comment at the top of
`crates/core/src/addon.rs` for why — Rust's ABI instability makes that a
poor fit for a v1). Addons are Rust structs compiled into the binary.

Trait (`tmr_core::addon::Addon`):

```rust
pub trait Addon {
    fn id(&self) -> &str;                              // config id, e.g. "stats"
    fn on_load(&mut self, _ctx: &AddonContext) {}         // called once at startup
    fn on_event(&mut self, _event: &AppEvent) {}          // called after every dispatched Command
    fn status_text(&self) -> Option<String> { None }      // optional status-bar contribution
}
```

Steps mirror widgets: implement the trait (see `StatsAddon` in
`crates/core/src/addon.rs` — counts opens/saves/creates/deletes as a
minimal working reference), register it in `main.rs` behind a
`config.addons.enabled` check via `app.register_addon(Box::new(...))`,
document the id under `[addons] enabled = [...]`.

If your addon needs to *do* something rather than just observe (e.g.
contribute a new `Command`), there's no registration point for that yet
— `crates/tui/src/input.rs::handle_action`'s match is closed over the
built-in action names. Extending that cleanly (e.g. letting an addon
claim a keymap action and a command) is exactly the kind of "seam that
doesn't exist yet" this trait was scoped to make *possible* to add later
without a rewrite, not something v1 wires end to end — don't assume it
works until you've actually threaded it through `input.rs`.

## Adding a new document format

Only Markdown gets real rendering today; everything else (`.txt`,
unrecognized extensions) falls through to a plain-text path. The seam:

1. `tmr_core::document::DocumentFormat` (`crates/core/src/document.rs`)
   is the format enum, detected by extension in `DocumentFormat::from_path`.
   Add your variant and its extension(s) there.
2. Everything in `tmr-core` past that point is format-agnostic — `App`,
   `Command::OpenFile`/`Save`, `fs_ops` all just move `String` content
   around regardless of format. You don't need to touch them.
3. The format-specific work is a parser crate analogous to
   `tmr-markdown`: source text -> some renderer-agnostic structure. It
   does not need to mirror `tmr-markdown`'s `Block`/`Inline` shape if the
   format doesn't fit that shape (e.g. a JSON viewer might want a tree of
   key/value nodes instead) — the only real contract is "produces
   something `tmr-tui` knows how to turn into `Vec<RenderedLine>`".
4. In `tmr-tui`, `crates/tui/src/input.rs::refresh_rendered` already
   dispatches on `doc.format`: `DocumentFormat::Markdown` goes through
   `tmr_markdown::parse` + `markdown_view::render` (the Obsidian-style
   path), everything else goes through `markdown_view::render_plain_text`.
   Add your variant as a new match arm calling your own parser + a new
   `render_your_format` function in `markdown_view.rs` (or a sibling
   module) — following the existing arms is the reference, there's no
   separate format-registry abstraction to learn.
5. **Keep the "only `.md` gets rich rendering" rule intact** unless
   explicitly asked to change it: a `.txt` file (or any format without
   its own render arm) must keep showing exactly what's on disk, never
   be silently parsed as Markdown — that was a real bug fixed in this
   codebase's history (see `CHANGELOG.md`'s Unreleased section), don't
   reintroduce it by adding a fallback that guesses.

## Adding a Kitty/iTerm2/Sixel image backend

`crates/tui/src/image_backend.rs` documents the seam explicitly:
`ImageCapability` is the detection result and `render()` is the single
entry point `markdown_view.rs` calls. To add a real terminal-graphics
backend:
1. Add a variant to `ImageCapability` (e.g. `Kitty`).
2. Extend `detect_capability()` — keep it environment-variable-only
   (checking `KITTY_WINDOW_ID`, `TERM_PROGRAM`, etc.); do **not** add a
   blocking terminal query (a write-and-read-response probe) without a
   hard timeout — the existing code deliberately avoids anything that
   could hang in an unusual terminal/CI environment.
3. Extend `render()` to try your new backend when detected, still
   falling back to `placeholder()` on any decode/protocol failure — never
   let a broken image propagate an error that takes down the document view.
