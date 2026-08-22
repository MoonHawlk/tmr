# Roadmap / known limitations

- **No word-wrap**: long lines are clipped at the pane edge rather than
  wrapped, so the document cursor can address a stable line index for
  checkbox toggling. Word-wrap is a reasonable follow-up but needs a
  cursor model that survives reflow. Edit mode's raw-source view is the
  one exception — it horizontally scrolls to follow the cursor past the
  right edge (see `UiState::doc_hscroll`) — but the normal Obsidian-style
  and plain-text Normal-mode views still just clip.
- **Editor starts at the top of the file for Markdown documents**: the
  opening cursor row now matches the line you were viewing for plain-text/
  unknown files, but Markdown's Obsidian-style rendering doesn't keep a
  1:1 line mapping to the raw source (headings, blank-line handling, etc.
  can shift rows), so there's no correct row to seed the editor with there
  yet — see `crates/tui/src/input.rs::viewed_source_row`. The built-in
  editor is intentionally minimal otherwise (see `crates/tui/src/editor.rs`);
  an external-editor integration is a plausible alternative for users who
  want more.
- **Images**: only local (non-`http`) images are rendered, as
  colored-halfblock approximations, gated on detecting truecolor support
  via environment variables (no blocking terminal queries, to avoid any
  risk of hanging on an unusual terminal). Kitty/iTerm2/Sixel protocol
  support is a natural next backend behind the existing `ImageBackend`
  seam (`crates/tui/src/image_backend.rs`).
- **Search** is filename substring / in-document line substring only, in
  the current directory / current document — no recursive global search
  or indexing (deliberately: the brief calls for not indexing the whole
  workspace up front).
- **No file-system watching** — the listing refreshes on navigation and
  on the explicit `reload` action, not via polling or `inotify`.
- Widgets/addons are compiled-in only (see [Architecture](architecture.md)).

See also [TODO](todo.md) for the tracked task list.

[← Back to README](../README.md)
