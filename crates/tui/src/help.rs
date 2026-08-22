//! The `h` command-reference popup: a static list of every built-in action
//! paired with a human description, resolved against the *actual* (possibly
//! remapped) keymap so the popup never lies about what a key does.
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use tmr_core::input::{Key, KeyCode};
use tmr_core::keymap::Keymap;

use crate::theme::Palette;

/// action id -> one-line description. Order here is the order shown in the
/// popup before filtering. Kept in sync with `crates/core/src/keymap.rs`'s
/// `DEFAULTS` by hand — there's no derive that would keep them honest
/// automatically, so add a row here whenever a new action id ships.
const ENTRIES: &[(&str, &str)] = &[
    ("focus_files", "Switch focus between Files and Document"),
    ("nav_up", "Move selection / cursor up"),
    ("nav_down", "Move selection / cursor down"),
    ("nav_enter", "Open the selected entry"),
    ("edit", "Start editing the open document"),
    ("nav_back", "Go to the parent directory"),
    ("toggle_task", "Toggle the checkbox on the cursor's line"),
    ("save", "Save the document (Edit mode)"),
    ("cancel", "Leave Edit mode / cancel a dialog"),
    ("select_all", "Select the entire document (Edit mode)"),
    ("copy", "Copy the current selection (Edit mode)"),
    ("cut", "Cut the current selection (Edit mode)"),
    ("search", "Search filenames or in-document text"),
    ("new_file", "Create a new file"),
    ("rename", "Rename the selected file"),
    ("delete", "Delete the selected file"),
    ("confirm", "Confirm a pending delete"),
    ("reload", "Re-list the current directory"),
    ("help", "Show this command reference"),
    (
        "settings",
        "Open the Settings window (theme, line indicator)",
    ),
    ("quit", "Quit tmr"),
];

/// Reconstructs a display string like `"ctrl+s"` from a resolved [`Key`] —
/// the inverse of `tmr_core::input::parse_key`, kept here (rather than in
/// `tmr-core`) because it exists purely to render UI text.
fn format_key(key: Key) -> String {
    let mut parts = Vec::new();
    if key.ctrl {
        parts.push("ctrl".to_string());
    }
    if key.alt {
        parts.push("alt".to_string());
    }
    if key.shift {
        parts.push("shift".to_string());
    }
    let name = match key.code {
        KeyCode::Char(' ') => "space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "pageup".to_string(),
        KeyCode::PageDown => "pagedown".to_string(),
    };
    parts.push(name);
    parts.join("+")
}

/// A rect centered in `area`, `percent_x`/`percent_y` of its size (clamped
/// so it never exceeds `area`).
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x.min(100) / 100;
    let height = area.height * percent_y.min(100) / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// `ENTRIES` filtered case-insensitively by `query` against the action id,
/// its resolved key and its description — resolved against `keymap` so the
/// popup never lies about what a (possibly remapped) key does. Shared by
/// `draw` (to render the rows) and `visible_count` (to clamp keyboard
/// navigation against however many rows are actually showing).
fn filtered<'a>(keymap: &Keymap, query: &str) -> Vec<(String, &'a str)> {
    let needle = query.to_ascii_lowercase();
    ENTRIES
        .iter()
        .filter_map(|(action, desc)| {
            let key = keymap
                .key_for(action)
                .map(format_key)
                .unwrap_or_else(|| "unbound".to_string());
            if !needle.is_empty()
                && !key.to_ascii_lowercase().contains(&needle)
                && !desc.to_ascii_lowercase().contains(&needle)
                && !action.to_ascii_lowercase().contains(&needle)
            {
                return None;
            }
            Some((key, *desc))
        })
        .collect()
}

/// How many rows the popup currently shows for `query` — used to clamp the
/// `selected` row when the user navigates with Up/Down, since typing can
/// shrink the filtered list out from under a previously-valid selection.
pub fn visible_count(keymap: &Keymap, query: &str) -> usize {
    filtered(keymap, query).len()
}

/// Draws the centered, `Clear`-backed popup listing every action bound in
/// `keymap`, filtered case-insensitively by `query` against both the key
/// and the description. `selected` is highlighted and, on terminals too
/// short to fit every row, kept in view via `ListState`'s built-in
/// auto-scroll. Purely visual — nothing here dispatches a `Command`.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    palette: &Palette,
    keymap: &Keymap,
    query: &str,
    selected: usize,
) {
    let popup = centered_rect(70, 80, area);
    frame.render_widget(Clear, popup);

    let entries = filtered(keymap, query);
    let rows: Vec<ListItem> = entries
        .iter()
        .map(|(key, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{key:>10}  "),
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(*desc),
            ]))
        })
        .collect();

    let has_matches = !rows.is_empty();
    let block = Block::default()
        .title(" Commands — type to filter, esc to close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.accent));

    let layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(0),
        ])
        .margin(1)
        .split(block.inner(popup));
    frame.render_widget(block, popup);

    let search_line = Line::from(vec![
        Span::styled("Search: ", Style::default().fg(palette.accent)),
        Span::raw(query),
        Span::styled("_", Style::default().fg(palette.muted)),
    ]);
    frame.render_widget(Paragraph::new(search_line), layout[0]);

    if has_matches {
        let list = List::new(rows).highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(palette.accent),
        );
        let mut state = ListState::default();
        state.select(Some(selected.min(entries.len() - 1)));
        frame.render_stateful_widget(list, layout[1], &mut state);
    } else {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No matching command",
                Style::default().fg(palette.muted),
            )),
            layout[1],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_count_matches_all_entries_when_query_is_empty() {
        let keymap = Keymap::default();
        assert_eq!(visible_count(&keymap, ""), ENTRIES.len());
    }

    #[test]
    fn visible_count_narrows_with_a_query() {
        let keymap = Keymap::default();
        let narrowed = visible_count(&keymap, "quit");
        assert!(narrowed >= 1);
        assert!(narrowed < ENTRIES.len());
    }

    #[test]
    fn visible_count_is_zero_for_a_query_matching_nothing() {
        let keymap = Keymap::default();
        assert_eq!(visible_count(&keymap, "zzzzz-no-such-command"), 0);
    }
}
