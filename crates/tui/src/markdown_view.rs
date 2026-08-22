//! Converts a `tmr_markdown::Block` tree into terminal lines.
//!
//! Deliberately does **not** word-wrap: every `Block` produces a fixed,
//! predictable number of [`RenderedLine`]s so that scrolling and
//! task-checkbox interaction can address lines by a stable index. Long
//! lines are clipped at the pane edge instead of wrapping (see README
//! roadmap) — a reasonable MVP trade-off, since word-wrap would make the
//! "cursor is on line N" bookkeeping needed for checkbox toggling depend on
//! terminal width.

use std::path::Path;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use tmr_markdown::{Alignment, Block, Inline, ListItem};

use crate::image_backend::{self, ImageCapability};
use crate::theme::Palette;

/// One line of rendered document body, plus which task-list item (if any)
/// it represents — set only on the first line of a task list item, which
/// is enough for cursor-based toggle-what's-under-me interaction.
pub struct RenderedLine {
    pub line: Line<'static>,
    pub task_index: Option<usize>,
}

pub fn render(
    blocks: &[Block],
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
) -> Vec<RenderedLine> {
    let mut out = Vec::new();
    render_blocks(blocks, palette, image_cap, base_dir, max_width, 0, &mut out);
    out
}

fn indent_span(indent: usize) -> Span<'static> {
    Span::raw(" ".repeat(indent))
}

fn push_plain(out: &mut Vec<RenderedLine>, indent: usize, spans: Vec<Span<'static>>) {
    let mut line_spans = vec![indent_span(indent)];
    line_spans.extend(spans);
    out.push(RenderedLine {
        line: Line::from(line_spans),
        task_index: None,
    });
}

fn render_blocks(
    blocks: &[Block],
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
    indent: usize,
    out: &mut Vec<RenderedLine>,
) {
    for block in blocks {
        render_block(block, palette, image_cap, base_dir, max_width, indent, out);
    }
}

fn render_block(
    block: &Block,
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
    indent: usize,
    out: &mut Vec<RenderedLine>,
) {
    match block {
        Block::Heading { level, inlines } => {
            let marker = "#".repeat(*level as usize) + " ";
            let mut style = Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD);
            if *level == 1 {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            for (i, spans) in inline_lines(inlines, style).into_iter().enumerate() {
                let lead = if i == 0 {
                    Span::styled(marker.clone(), style)
                } else {
                    Span::raw(" ".repeat(marker.chars().count()))
                };
                push_plain(out, indent, [vec![lead], spans].concat());
            }
        }
        Block::Paragraph(inlines) => {
            let style = Style::default().fg(palette.fg);
            for spans in inline_lines(inlines, style) {
                push_plain(out, indent, spans);
            }
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            render_list(
                items, *ordered, *start, palette, image_cap, base_dir, max_width, indent, out,
            );
        }
        Block::CodeBlock { code, .. } => {
            let style = Style::default().fg(palette.muted);
            for line in code.lines() {
                push_plain(out, indent, vec![Span::styled(line.to_string(), style)]);
            }
        }
        Block::Blockquote(inner) => {
            let start = out.len();
            render_blocks(
                inner,
                palette,
                image_cap,
                base_dir,
                max_width,
                indent + 2,
                out,
            );
            let marker = Span::styled(
                "> ",
                Style::default()
                    .fg(palette.muted)
                    .add_modifier(Modifier::ITALIC),
            );
            for rendered in &mut out[start..] {
                replace_leading_indent(
                    &mut rendered.line,
                    vec![indent_span(indent), marker.clone()],
                );
            }
        }
        Block::Table {
            alignments,
            headers,
            rows,
        } => {
            render_table(alignments, headers, rows, palette, indent, out);
        }
        Block::Image { alt, url, .. } => {
            for line in render_image(
                url,
                alt,
                image_cap,
                base_dir,
                max_width.saturating_sub(indent as u16),
            ) {
                out.push(RenderedLine {
                    line,
                    task_index: None,
                });
            }
        }
        Block::ThematicBreak => {
            let width = (max_width as usize).saturating_sub(indent).clamp(4, 60);
            push_plain(
                out,
                indent,
                vec![Span::styled(
                    "-".repeat(width),
                    Style::default().fg(palette.border),
                )],
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_list(
    items: &[ListItem],
    ordered: bool,
    start: Option<u64>,
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
    indent: usize,
    out: &mut Vec<RenderedLine>,
) {
    let mut num = start.unwrap_or(1);
    for item in items {
        let (prefix, style) = match item.checked {
            Some(true) => ("[x] ".to_string(), Style::default().fg(palette.success)),
            Some(false) => ("[ ] ".to_string(), Style::default().fg(palette.fg)),
            None if ordered => {
                let s = format!("{num}. ");
                num += 1;
                (s, Style::default().fg(palette.fg))
            }
            None => ("• ".to_string(), Style::default().fg(palette.fg)),
        };
        let prefix_len = prefix.chars().count();
        let content_indent = indent + prefix_len;
        let start_idx = out.len();
        render_blocks(
            &item.content,
            palette,
            image_cap,
            base_dir,
            max_width,
            content_indent,
            out,
        );

        let marker = vec![indent_span(indent), Span::styled(prefix, style)];
        if start_idx == out.len() {
            out.push(RenderedLine {
                line: Line::from(marker),
                task_index: item.task_index,
            });
        } else {
            replace_leading_indent(&mut out[start_idx].line, marker);
            out[start_idx].task_index = item.task_index;
        }
    }
}

/// Replaces the leading indent span (assumed to be `spans[0]`, produced by
/// [`indent_span`]) with `prefix`, preserving the rest of the line.
fn replace_leading_indent(line: &mut Line<'static>, prefix: Vec<Span<'static>>) {
    let mut rest = std::mem::take(&mut line.spans);
    if !rest.is_empty() {
        rest.remove(0);
    }
    let mut new_spans = prefix;
    new_spans.extend(rest);
    line.spans = new_spans;
}

fn render_table(
    alignments: &[Alignment],
    headers: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    palette: &Palette,
    indent: usize,
    out: &mut Vec<RenderedLine>,
) {
    let col_count = headers
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    let mut widths = vec![3usize; col_count];
    let header_text: Vec<String> = headers
        .iter()
        .map(|c| tmr_markdown::ast::flatten_text(c))
        .collect();
    let row_text: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|c| tmr_markdown::ast::flatten_text(c))
                .collect()
        })
        .collect();
    for (i, text) in header_text.iter().enumerate() {
        widths[i] = widths[i].max(text.chars().count());
    }
    for row in &row_text {
        for (i, text) in row.iter().enumerate() {
            widths[i] = widths[i].max(text.chars().count());
        }
    }

    let fmt_row = |cells: &[String], style: Style| -> Line<'static> {
        let mut spans = vec![indent_span(indent)];
        for (i, w) in widths.iter().enumerate() {
            let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            let aligned = pad_cell(
                cell,
                *w,
                alignments.get(i).copied().unwrap_or(Alignment::None),
            );
            spans.push(Span::styled(format!("{aligned} "), style));
            if i + 1 < widths.len() {
                spans.push(Span::styled("| ", Style::default().fg(palette.border)));
            }
        }
        Line::from(spans)
    };

    let header_style = Style::default()
        .fg(palette.accent)
        .add_modifier(Modifier::BOLD);
    out.push(RenderedLine {
        line: fmt_row(&header_text, header_style),
        task_index: None,
    });
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    out.push(RenderedLine {
        line: fmt_row(&sep, Style::default().fg(palette.border)),
        task_index: None,
    });
    for row in &row_text {
        out.push(RenderedLine {
            line: fmt_row(row, Style::default().fg(palette.fg)),
            task_index: None,
        });
    }
}

fn pad_cell(text: &str, width: usize, align: Alignment) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.to_string();
    }
    let pad = width - len;
    match align {
        Alignment::Right => format!("{}{}", " ".repeat(pad), text),
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
        Alignment::Left | Alignment::None => format!("{}{}", text, " ".repeat(pad)),
    }
}

fn render_image(
    url: &str,
    alt: &str,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
) -> Vec<Line<'static>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return vec![Line::from(Span::styled(
            format!("[image: {alt} ({url})]"),
            Style::default().add_modifier(Modifier::ITALIC),
        ))];
    }
    let path = base_dir.join(url);
    image_backend::render(&path, image_cap, max_width, 12)
}

/// Splits an inline run into groups of styled spans, breaking a new group
/// at each hard line break.
fn inline_lines(inlines: &[Inline], base_style: Style) -> Vec<Vec<Span<'static>>> {
    let mut out = vec![Vec::new()];
    push_inlines(inlines, base_style, &mut out);
    out
}

fn push_inlines(inlines: &[Inline], style: Style, out: &mut Vec<Vec<Span<'static>>>) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => out.last_mut().unwrap().push(Span::styled(t.clone(), style)),
            Inline::Bold(inner) => push_inlines(inner, style.add_modifier(Modifier::BOLD), out),
            Inline::Italic(inner) => push_inlines(inner, style.add_modifier(Modifier::ITALIC), out),
            Inline::Strikethrough(inner) => {
                push_inlines(inner, style.add_modifier(Modifier::CROSSED_OUT), out)
            }
            Inline::Code(t) => out.last_mut().unwrap().push(Span::styled(
                t.clone(),
                style.add_modifier(Modifier::REVERSED),
            )),
            Inline::Link { text, .. } => {
                push_inlines(text, style.add_modifier(Modifier::UNDERLINED), out)
            }
            Inline::Image { alt, .. } => out
                .last_mut()
                .unwrap()
                .push(Span::styled(format!("[{alt}]"), style)),
            Inline::SoftBreak => out.last_mut().unwrap().push(Span::styled(" ", style)),
            Inline::HardBreak => out.push(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tmr_core::theme::Theme;

    fn palette() -> Palette {
        Palette::from_theme(&Theme::dark())
    }

    #[test]
    fn renders_heading_with_marker() {
        let blocks = tmr_markdown::parse("# Title\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        assert_eq!(lines.len(), 1);
        let text: String = lines[0]
            .line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("# Title"));
    }

    #[test]
    fn task_list_lines_carry_task_index() {
        let blocks = tmr_markdown::parse("- [ ] a\n- [x] b\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        assert_eq!(lines[0].task_index, Some(0));
        assert_eq!(lines[1].task_index, Some(1));
    }

    #[test]
    fn nested_task_list_keeps_indent_and_index() {
        let blocks = tmr_markdown::parse("- item\n  - [ ] nested\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let nested = lines.iter().find(|l| l.task_index == Some(0)).unwrap();
        let text: String = nested
            .line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.starts_with("  ")); // indented under parent item
        assert!(text.contains("[ ] nested"));
    }

    #[test]
    fn missing_image_falls_back_to_placeholder_text() {
        let blocks = tmr_markdown::parse("![alt](missing.png)\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let text: String = lines[0]
            .line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("[image:"));
    }
}
