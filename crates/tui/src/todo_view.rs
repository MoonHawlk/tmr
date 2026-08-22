//! The Quick-TODO window: a minimal, always-available task list backed by
//! `tmr_core::tasks::TaskStore` — independent of any open document, so a
//! task can be captured without navigating to or opening a Markdown file.
//! Opened with `ctrl+t` (default binding); see
//! `crates/tui/src/input.rs::handle_todo_key` for the interaction logic
//! this only renders.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use tmr_core::tasks::{Task, TaskStatus};

use crate::theme::Palette;

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x.min(100) / 100;
    let height = area.height * percent_y.min(100) / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// `tasks` is expected to already be the visible (non-deleted) list, in
/// display order — callers pass `app.tasks().visible().collect()`, this
/// module doesn't know about `Deleted` tasks at all.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    palette: &Palette,
    tasks: &[&Task],
    selected: usize,
    composing: Option<&str>,
) {
    let popup = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(
            " Quick-TODO \u{2014} ctrl+n new \u{b7} space done \u{b7} d delete \u{b7} shift+\u{2191}\u{2193} move \u{b7} esc close ",
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.accent));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .margin(1)
        .split(inner);

    let header_line = match composing {
        Some(buf) => Line::from(vec![
            Span::styled("New: ", Style::default().fg(palette.accent)),
            Span::raw(buf.to_string()),
            Span::styled("_", Style::default().fg(palette.muted)),
        ]),
        None => Line::from(Span::styled(
            format!("{} task(s) \u{2014} ctrl+n to add", tasks.len()),
            Style::default().fg(palette.muted),
        )),
    };
    frame.render_widget(Paragraph::new(header_line), layout[0]);

    if tasks.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No tasks yet \u{2014} ctrl+n to add one",
                Style::default().fg(palette.muted),
            )),
            layout[1],
        );
        return;
    }

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|t| {
            let (glyph, style) = match t.status {
                TaskStatus::Done => ("\u{2611}", Style::default().fg(palette.success)),
                _ => ("\u{2610}", Style::default().fg(palette.fg)),
            };
            let text_style = if t.status == TaskStatus::Done {
                style.add_modifier(Modifier::CROSSED_OUT)
            } else {
                style
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{glyph} "), style),
                Span::styled(t.text.clone(), text_style),
            ]))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(palette.accent),
    );
    let mut state = ListState::default();
    state.select(Some(selected.min(tasks.len() - 1)));
    frame.render_stateful_widget(list, layout[1], &mut state);
}
