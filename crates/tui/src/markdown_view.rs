//! Converts a `tmr_markdown::Block` tree into terminal lines, styled the
//! way an "Obsidian-style" live-preview editor would: raw syntax markers
//! (`#`, `**`, `` ` ``, `[]()`, `> `) are never printed — each construct
//! gets its own glyph/weight/color instead, distinct from every other
//! construct, so the shape of a document is visible at a glance. This only
//! applies to Markdown; [`render_plain_text`] is the untouched-text path
//! used for every other format (see `crates/tui/src/input.rs`, which picks
//! between the two based on `DocumentFormat`).
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
    render_blocks(
        blocks, palette, image_cap, base_dir, max_width, 0, 0, &mut out,
    );
    out
}

/// Renders a non-Markdown document as plain text: one line per source
/// line, no parsing, no styling beyond the theme's base foreground. This
/// is what a `.txt` (or any other/unknown-format) file gets — Markdown
/// syntax in such a file is shown verbatim, exactly as written.
pub fn render_plain_text(content: &str, palette: &Palette) -> Vec<RenderedLine> {
    let style = Style::default().fg(palette.fg);
    content
        .lines()
        .map(|line| RenderedLine {
            line: Line::from(Span::styled(line.to_string(), style)),
            task_index: None,
        })
        .collect()
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

#[allow(clippy::too_many_arguments)]
fn render_blocks(
    blocks: &[Block],
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
    indent: usize,
    list_depth: usize,
    out: &mut Vec<RenderedLine>,
) {
    for block in blocks {
        render_block(
            block, palette, image_cap, base_dir, max_width, indent, list_depth, out,
        );
    }
}

/// Heading style by level: prominence (color, weight, an underline rule
/// for level 1) decreases as the level increases, since a terminal can't
/// vary font size the way Obsidian's preview does.
fn heading_style(level: u8, palette: &Palette) -> (Style, bool) {
    match level {
        1 => (
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            true,
        ),
        2 => (
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
            false,
        ),
        3 => (
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            false,
        ),
        4 => (
            Style::default()
                .fg(palette.fg)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            false,
        ),
        5 => (
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::BOLD),
            false,
        ),
        _ => (
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::ITALIC),
            false,
        ),
    }
}

/// Bullet glyph for an unordered list item, varied by nesting depth —
/// distinct rings the way Obsidian's preview distinguishes list levels.
fn bullet_glyph(depth: usize) -> &'static str {
    match depth {
        0 => "•",
        1 => "◦",
        _ => "▪",
    }
}

#[allow(clippy::too_many_arguments)]
fn render_block(
    block: &Block,
    palette: &Palette,
    image_cap: ImageCapability,
    base_dir: &Path,
    max_width: u16,
    indent: usize,
    list_depth: usize,
    out: &mut Vec<RenderedLine>,
) {
    match block {
        Block::Heading { level, inlines } => {
            let (style, underline_rule) = heading_style(*level, palette);
            for spans in inline_lines(inlines, style, palette) {
                push_plain(out, indent, spans);
            }
            if underline_rule {
                let width = (max_width as usize).saturating_sub(indent).clamp(4, 60);
                push_plain(
                    out,
                    indent,
                    vec![Span::styled(
                        "─".repeat(width),
                        Style::default().fg(palette.accent),
                    )],
                );
            }
        }
        Block::Paragraph(inlines) => {
            let style = Style::default().fg(palette.fg);
            for spans in inline_lines(inlines, style, palette) {
                push_plain(out, indent, spans);
            }
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            render_list(
                items, *ordered, *start, palette, image_cap, base_dir, max_width, indent,
                list_depth, out,
            );
        }
        Block::CodeBlock { code, .. } => {
            let bar = Span::styled("▏", Style::default().fg(palette.muted));
            let style = Style::default().fg(palette.muted);
            for line in code.lines() {
                push_plain(
                    out,
                    indent,
                    vec![
                        bar.clone(),
                        Span::raw(" "),
                        Span::styled(line.to_string(), style),
                    ],
                );
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
                list_depth,
                out,
            );
            let marker = Span::styled("▎", Style::default().fg(palette.accent));
            for rendered in &mut out[start..] {
                replace_leading_indent(
                    &mut rendered.line,
                    vec![indent_span(indent), marker.clone(), Span::raw(" ")],
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
                    "─".repeat(width),
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
    list_depth: usize,
    out: &mut Vec<RenderedLine>,
) {
    let mut num = start.unwrap_or(1);
    for item in items {
        let (prefix, style) = match item.checked {
            Some(true) => ("☑ ".to_string(), Style::default().fg(palette.success)),
            Some(false) => ("☐ ".to_string(), Style::default().fg(palette.fg)),
            None if ordered => {
                let s = format!("{num}. ");
                num += 1;
                (s, Style::default().fg(palette.fg))
            }
            None => (
                format!("{} ", bullet_glyph(list_depth)),
                Style::default().fg(palette.fg),
            ),
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
            list_depth + 1,
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
                spans.push(Span::styled("│ ", Style::default().fg(palette.border)));
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
    let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
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
fn inline_lines(
    inlines: &[Inline],
    base_style: Style,
    palette: &Palette,
) -> Vec<Vec<Span<'static>>> {
    let mut out = vec![Vec::new()];
    push_inlines(inlines, base_style, palette, &mut out);
    out
}

fn push_inlines(
    inlines: &[Inline],
    style: Style,
    palette: &Palette,
    out: &mut Vec<Vec<Span<'static>>>,
) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => out.last_mut().unwrap().push(Span::styled(t.clone(), style)),
            Inline::Bold(inner) => {
                push_inlines(inner, style.add_modifier(Modifier::BOLD), palette, out)
            }
            Inline::Italic(inner) => {
                push_inlines(inner, style.add_modifier(Modifier::ITALIC), palette, out)
            }
            Inline::Strikethrough(inner) => push_inlines(
                inner,
                style.add_modifier(Modifier::CROSSED_OUT),
                palette,
                out,
            ),
            // Padded like a small pill, echoing how Obsidian sets inline
            // code apart from surrounding prose with a filled background.
            Inline::Code(t) => out.last_mut().unwrap().push(Span::styled(
                format!(" {t} "),
                style.fg(palette.fg).add_modifier(Modifier::REVERSED),
            )),
            Inline::Link { text, .. } => push_inlines(
                text,
                style.fg(palette.accent).add_modifier(Modifier::UNDERLINED),
                palette,
                out,
            ),
            Inline::Image { alt, .. } => out
                .last_mut()
                .unwrap()
                .push(Span::styled(format!("[{alt}]"), style.fg(palette.muted))),
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

    fn line_text(rl: &RenderedLine) -> String {
        rl.line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn heading_hides_hash_marker_and_gets_underline_rule() {
        let pal = palette();
        let blocks = tmr_markdown::parse("# Title\n");
        let lines = render(&blocks, &pal, ImageCapability::None, Path::new("."), 80);
        // line 0: styled text, no literal "#"; line 1: the underline rule.
        assert_eq!(lines.len(), 2);
        assert!(!line_text(&lines[0]).contains('#'));
        assert!(line_text(&lines[0]).contains("Title"));
        assert!(lines[0]
            .line
            .spans
            .iter()
            .any(|s| s.style.fg == Some(pal.accent)
                && s.style.add_modifier.contains(Modifier::UNDERLINED)));
        assert!(line_text(&lines[1]).chars().all(|c| c == '─'));
    }

    #[test]
    fn heading_levels_get_decreasing_prominence() {
        let (h1, _) = heading_style(1, &palette());
        let (h3, _) = heading_style(3, &palette());
        let (h6, _) = heading_style(6, &palette());
        assert!(h1.add_modifier.contains(Modifier::UNDERLINED));
        assert!(!h3.add_modifier.contains(Modifier::UNDERLINED));
        assert_ne!(h1.fg, h6.fg);
    }

    #[test]
    fn task_list_uses_checkbox_glyphs_not_literal_brackets() {
        let blocks = tmr_markdown::parse("- [ ] a\n- [x] b\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        assert!(line_text(&lines[0]).contains('☐'));
        assert!(line_text(&lines[1]).contains('☑'));
        assert!(!line_text(&lines[0]).contains('['));
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
        let text = line_text(nested);
        assert!(text.starts_with("  ")); // indented under parent item
        assert!(text.contains("☐ nested"));
    }

    #[test]
    fn unordered_bullets_vary_by_nesting_depth() {
        let blocks = tmr_markdown::parse("- top\n  - mid\n    - deep\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let texts: Vec<String> = lines.iter().map(line_text).collect();
        assert!(texts[0].trim_start().starts_with('•'));
        assert!(texts.iter().any(|t| t.trim_start().starts_with('◦')));
        assert!(texts.iter().any(|t| t.trim_start().starts_with('▪')));
    }

    #[test]
    fn blockquote_uses_bar_marker_not_angle_bracket() {
        let blocks = tmr_markdown::parse("> quoted\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let text = line_text(&lines[0]);
        assert!(text.contains('▎'));
        assert!(!text.contains('>'));
    }

    #[test]
    fn code_block_gets_left_bar_and_no_backticks() {
        let blocks = tmr_markdown::parse("```\nlet x = 1;\n```\n");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let text = line_text(&lines[0]);
        assert!(text.contains('▏'));
        assert!(text.contains("let x = 1;"));
        assert!(!text.contains('`'));
    }

    #[test]
    fn inline_code_is_padded_like_a_pill() {
        let blocks = tmr_markdown::parse("a `code` b");
        let lines = render(
            &blocks,
            &palette(),
            ImageCapability::None,
            Path::new("."),
            80,
        );
        let text = line_text(&lines[0]);
        assert!(text.contains(" code "));
        assert!(!text.contains('`'));
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
        assert!(line_text(&lines[0]).contains("[image:"));
    }

    #[test]
    fn plain_text_render_does_not_parse_markdown_syntax() {
        let lines = render_plain_text("# not a heading\n- [ ] not a task\n", &palette());
        assert_eq!(lines.len(), 2);
        assert_eq!(line_text(&lines[0]), "# not a heading");
        assert_eq!(line_text(&lines[1]), "- [ ] not a task");
        assert!(lines.iter().all(|l| l.task_index.is_none()));
    }
}
