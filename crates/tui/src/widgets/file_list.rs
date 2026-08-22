use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

use tmr_core::workspace::Entry;

use crate::layout::styled_block;
use crate::state::Focus;
use crate::theme::Palette;
use tmr_core::config::BorderStyle;

#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    entries: &[Entry],
    selected: usize,
    focus: Focus,
    palette: &Palette,
    border: BorderStyle,
) {
    let is_focused = focus == Focus::Files;
    let block = styled_block("FILES", border, palette, is_focused);

    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| {
            let label = if e.is_dir {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            };
            let style = if e.is_dir {
                Style::default().fg(palette.accent)
            } else {
                Style::default().fg(palette.fg)
            };
            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let mut state = ListState::default();
    if !entries.is_empty() {
        state.select(Some(selected));
    }

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list, area, &mut state);
}
