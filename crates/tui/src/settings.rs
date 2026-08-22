//! The Settings window: a small, always-available interface-customization
//! popup (default binding `s`) for choosing a color theme and how the
//! Document pane marks its current line. `Up`/`Down` moves between the two
//! rows, `Left`/`Right` (or `Enter`) cycles the highlighted row's value —
//! applied live, no restart needed — and `Esc` closes it.
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use tmr_core::config::BorderStyle;

use crate::state::{LineIndicatorStyle, ThemeChoice};
use crate::theme::Palette;

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let margin_y = (100 - percent_y) / 2;
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(margin_y),
            Constraint::Percentage(percent_y),
            Constraint::Percentage(margin_y),
        ])
        .split(area);

    let margin_x = (100 - percent_x) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin_x),
            Constraint::Percentage(percent_x),
            Constraint::Percentage(margin_x),
        ])
        .split(vertical[1])[1]
}

fn option_row(
    label: &'static str,
    options: &[(&'static str, bool)],
    focused: bool,
    palette: &Palette,
) -> Line<'static> {
    let label_style = if focused {
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.fg)
    };
    let mut spans = vec![
        Span::styled(if focused { "> " } else { "  " }, label_style),
        Span::styled(format!("{label:<15}"), label_style),
    ];
    for (i, (text, active)) in options.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let style = if *active {
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default().fg(palette.muted)
        };
        spans.push(Span::styled(format!(" {text} "), style));
    }
    Line::from(spans)
}

#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    palette: &Palette,
    theme_choice: ThemeChoice,
    border: BorderStyle,
    line_indicator: LineIndicatorStyle,
    timer: bool,
    row: usize,
) {
    let popup = centered_rect(64, 40, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(
            " Settings — \u{2191}\u{2193} select \u{b7} \u{2190}\u{2192} change \u{b7} esc close ",
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.accent));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(inner);

    let theme_options: Vec<(&'static str, bool)> = ThemeChoice::ALL
        .iter()
        .map(|c| (c.label(), *c == theme_choice))
        .collect();
    frame.render_widget(
        Paragraph::new(option_row("Theme", &theme_options, row == 0, palette)),
        rows[0],
    );

    let border_options: Vec<(&'static str, bool)> = BorderStyle::ALL
        .iter()
        .map(|b| (b.label(), *b == border))
        .collect();
    frame.render_widget(
        Paragraph::new(option_row("Border", &border_options, row == 1, palette)),
        rows[1],
    );

    let indicator_options = [
        (
            LineIndicatorStyle::Highlight.label(),
            line_indicator == LineIndicatorStyle::Highlight,
        ),
        (
            LineIndicatorStyle::Bar.label(),
            line_indicator == LineIndicatorStyle::Bar,
        ),
    ];
    frame.render_widget(
        Paragraph::new(option_row(
            "Line indicator",
            &indicator_options,
            row == 2,
            palette,
        )),
        rows[2],
    );

    let timer_options = [("On", timer), ("Off", !timer)];
    frame.render_widget(
        Paragraph::new(option_row("Timer bar", &timer_options, row == 3, palette)),
        rows[3],
    );
}
