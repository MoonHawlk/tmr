use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use tmr_core::config::BorderStyle;

use crate::layout::styled_block;
use crate::markdown_view::RenderedLine;
use crate::state::{Focus, LineIndicatorStyle};
use crate::theme::Palette;

/// Width (in characters) of the line-number digits alone, given how many
/// lines are in the document — at least 2, so short files don't get a
/// cramped 1-wide gutter.
fn number_width(total_lines: usize) -> usize {
    total_lines.max(1).to_string().len().max(2)
}

/// Total gutter width in columns: a 1-char marker column (blank, or the
/// `Bar` indicator on the cursor's line) + the line-number digits + one
/// trailing space before the content. Exposed so `ui.rs` can compute the
/// same offset when placing the real terminal cursor during Edit mode —
/// it must agree with `draw` exactly, or the cursor lands on the wrong
/// column.
pub fn gutter_cols(total_lines: usize) -> u16 {
    (number_width(total_lines) + 2) as u16
}

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
    indicator: LineIndicatorStyle,
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
    let digits = number_width(rendered.len());
    let lines: Vec<Line> = rendered
        .iter()
        .enumerate()
        .skip(scroll)
        .take(height)
        .map(|(idx, rl)| {
            let is_cursor = is_focused && idx == cursor;
            let bar = is_cursor && indicator == LineIndicatorStyle::Bar;
            let marker = if bar { "\u{258F}" } else { " " };
            let gutter_text = format!("{marker}{:>digits$} ", idx + 1);
            let gutter_style = if bar {
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.muted)
            };

            if is_cursor && indicator == LineIndicatorStyle::Highlight {
                let mut spans = vec![Span::styled(
                    gutter_text,
                    gutter_style.add_modifier(Modifier::REVERSED),
                )];
                spans.extend(rl.line.spans.iter().map(|s| {
                    Span::styled(s.content.clone(), s.style.add_modifier(Modifier::REVERSED))
                }));
                Line::from(spans)
            } else {
                let mut spans = vec![Span::styled(gutter_text, gutter_style)];
                spans.extend(rl.line.spans.iter().cloned());
                Line::from(spans)
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
    inner
}
