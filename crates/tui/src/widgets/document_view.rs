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

fn char_byte_offset(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

/// Re-splits `spans` so the half-open character range `[start, end)`
/// (character offsets from the start of the *line*, not any one span) gets
/// `extra` layered onto its existing style, leaving the rest untouched.
/// `end == usize::MAX` means "to the end of the line" — used for the
/// first/middle lines of a multi-line selection. Operates per-span rather
/// than assuming a single span per line, so it stays correct if a future
/// rendered format puts more than one style on a line.
fn overlay_style<'a>(
    spans: &[Span<'a>],
    start: usize,
    end: usize,
    extra: Modifier,
) -> Vec<Span<'a>> {
    let mut out = Vec::with_capacity(spans.len());
    let mut offset = 0usize;
    for span in spans {
        let len = span.content.chars().count();
        let span_start = offset;
        let span_end = offset + len;
        offset = span_end;

        if end <= span_start || start >= span_end {
            out.push(span.clone());
            continue;
        }

        let local_start = start.saturating_sub(span_start).min(len);
        let local_end = end.min(span_end).saturating_sub(span_start).min(len);
        let content = span.content.as_ref();
        let b0 = char_byte_offset(content, local_start);
        let b1 = char_byte_offset(content, local_end);

        if b0 > 0 {
            out.push(Span::styled(content[..b0].to_string(), span.style));
        }
        if b1 > b0 {
            out.push(Span::styled(
                content[b0..b1].to_string(),
                span.style.add_modifier(extra),
            ));
        }
        if b1 < content.len() {
            out.push(Span::styled(content[b1..].to_string(), span.style));
        }
    }
    out
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
    // Shift+navigation text selection during Edit mode: `((start_row,
    // start_col), (end_row, end_col))` in the same row/char-column space
    // as `rendered` — `None` outside Edit mode, where there's no selection
    // concept. See `crates/tui/src/editor.rs::Editor::selection_range`.
    selection: Option<((usize, usize), (usize, usize))>,
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
            let is_highlight_line = is_cursor && indicator == LineIndicatorStyle::Highlight;
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
            let gutter_style = if is_highlight_line {
                gutter_style.add_modifier(Modifier::REVERSED)
            } else {
                gutter_style
            };

            let mut content_spans: Vec<Span> = if is_highlight_line {
                rl.line
                    .spans
                    .iter()
                    .map(|s| {
                        Span::styled(s.content.clone(), s.style.add_modifier(Modifier::REVERSED))
                    })
                    .collect()
            } else {
                rl.line.spans.to_vec()
            };

            if let Some(((sel_start_row, sel_start_col), (sel_end_row, sel_end_col))) = selection {
                if idx >= sel_start_row && idx <= sel_end_row {
                    let start = if idx == sel_start_row {
                        sel_start_col
                    } else {
                        0
                    };
                    let end = if idx == sel_end_row {
                        sel_end_col
                    } else {
                        usize::MAX
                    };
                    content_spans = overlay_style(&content_spans, start, end, Modifier::REVERSED);
                }
            }

            let mut spans = vec![Span::styled(gutter_text, gutter_style)];
            spans.extend(content_spans);
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
    inner
}

#[cfg(test)]
mod tests {
    use super::*;

    // `draw` itself needs a real `Frame` (this project's TUI tests avoid
    // that — see `development-workflow.md`), so these exercise the plain
    // helper functions that decide *what* gets styled, the same way
    // `markdown_view`'s tests inspect spans without touching a terminal.

    #[test]
    fn number_width_grows_with_line_count() {
        assert_eq!(number_width(1), 2);
        assert_eq!(number_width(9), 2);
        assert_eq!(number_width(10), 2);
        assert_eq!(number_width(100), 3);
        assert_eq!(number_width(1000), 4);
    }

    #[test]
    fn gutter_cols_is_marker_plus_digits_plus_space() {
        assert_eq!(gutter_cols(9), 4); // 1 marker + 2 digits + 1 space
        assert_eq!(gutter_cols(100), 5); // 1 marker + 3 digits + 1 space
    }

    #[test]
    fn overlay_style_highlights_middle_of_a_single_span() {
        let spans = vec![Span::raw("hello world")];
        let out = overlay_style(&spans, 2, 5, Modifier::REVERSED);
        let rendered: Vec<(String, bool)> = out
            .iter()
            .map(|s| {
                (
                    s.content.to_string(),
                    s.style.add_modifier.contains(Modifier::REVERSED),
                )
            })
            .collect();
        assert_eq!(
            rendered,
            vec![
                ("he".to_string(), false),
                ("llo".to_string(), true),
                (" world".to_string(), false),
            ]
        );
    }

    #[test]
    fn overlay_style_to_end_of_line_uses_max_sentinel() {
        let spans = vec![Span::raw("abcdef")];
        let out = overlay_style(&spans, 3, usize::MAX, Modifier::REVERSED);
        let rendered: Vec<(String, bool)> = out
            .iter()
            .map(|s| {
                (
                    s.content.to_string(),
                    s.style.add_modifier.contains(Modifier::REVERSED),
                )
            })
            .collect();
        assert_eq!(
            rendered,
            vec![("abc".to_string(), false), ("def".to_string(), true)]
        );
    }

    #[test]
    fn overlay_style_spans_a_selection_crossing_two_spans() {
        // "foo" + "bar" = "foobar"; selecting chars [2, 5) covers "o" (end
        // of the first span) and "ba" (start of the second).
        let spans = vec![Span::raw("foo"), Span::raw("bar")];
        let out = overlay_style(&spans, 2, 5, Modifier::REVERSED);
        let selected: String = out
            .iter()
            .filter(|s| s.style.add_modifier.contains(Modifier::REVERSED))
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(selected, "oba");
    }

    #[test]
    fn overlay_style_handles_multibyte_utf8_without_panicking() {
        let spans = vec![Span::raw("h\u{e9}llo")]; // "héllo"
        let out = overlay_style(&spans, 1, 3, Modifier::REVERSED);
        let full: String = out.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(full, "h\u{e9}llo");
        let selected: String = out
            .iter()
            .filter(|s| s.style.add_modifier.contains(Modifier::REVERSED))
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(selected, "\u{e9}l");
    }

    #[test]
    fn overlay_style_no_op_when_range_misses_span() {
        let spans = vec![Span::raw("hello")];
        let out = overlay_style(&spans, 10, 20, Modifier::REVERSED);
        assert_eq!(out.len(), 1);
        assert!(!out[0].style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(out[0].content, "hello");
    }
}
