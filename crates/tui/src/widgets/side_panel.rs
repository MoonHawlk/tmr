use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use tmr_core::config::BorderStyle;
use tmr_core::widget::Widget;

use crate::layout::styled_block;
use crate::theme::Palette;

/// Renders every enabled [`Widget`] stacked vertically in a side column.
/// This panel only appears when at least one widget is enabled via
/// `[widgets] enabled = [...]` in `config.toml` — it's the proof that the
/// widget abstraction is wired end to end, not a permanent UI fixture.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    widgets: &[Box<dyn Widget>],
    palette: &Palette,
    border: BorderStyle,
) {
    let enabled: Vec<&Box<dyn Widget>> = widgets.iter().filter(|w| w.is_enabled()).collect();
    if enabled.is_empty() {
        return;
    }
    let constraints: Vec<Constraint> = enabled.iter().map(|_| Constraint::Length(3)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (widget, chunk) in enabled.iter().zip(chunks.iter()) {
        let block = styled_block(widget.title(), border, palette, false);
        let lines: Vec<Line> = widget.render_lines().into_iter().map(Line::from).collect();
        frame.render_widget(Paragraph::new(lines).block(block), *chunk);
    }
}
