use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use tmr_core::config::BorderStyle;

use crate::layout::styled_block;
use crate::markdown_view::RenderedLine;
use crate::state::Focus;
use crate::theme::Palette;

/// Draws the Document pane and returns its interior rect (inside the
/// border), so callers can compute where within it to place the real
/// terminal cursor (see `ui.rs::draw`, Edit-mode cursor placement).
#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    rendered: &[RenderedLine],
    cursor: usize,
    scroll: usize,
    focus: Focus,
    palette: &Palette,
    border: BorderStyle,
    empty_hint: Option<&str>,
) -> Rect {
    let is_focused = focus == Focus::Document;
    let block = styled_block("DOCUMENT", border, palette, is_focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if rendered.is_empty() {
        if let Some(hint) = empty_hint {
            frame.render_widget(Paragraph::new(hint), inner);
        }
        return inner;
    }

    let height = inner.height as usize;
    let lines: Vec<Line> = rendered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(height)
        .map(|(idx, rl)| {
            if is_focused && idx == cursor {
                let spans: Vec<Span> = rl
                    .line
                    .spans
                    .iter()
                    .map(|s| {
                        Span::styled(s.content.clone(), s.style.add_modifier(Modifier::REVERSED))
                    })
                    .collect();
                Line::from(spans)
            } else {
                rl.line.clone()
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
    inner
}
