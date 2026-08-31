//! The Calendar window: a small popup (default binding `c`) showing a
//! mini month-grid preview, aligned like a standard calendar — weekday
//! columns, today's day highlighted. `left`/`right` moves to the adjacent
//! month; `esc` closes it. Purely visual; nothing here dispatches a
//! `Command`.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use tmr_core::datetime::CivilDate;

use crate::theme::Palette;

const WEEKDAY_HEADERS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
/// Width of the fixed 7-column grid (`"XX "` per day) plus the popup's own
/// border, matching what `draw` actually renders — wide enough for the
/// grid, narrow enough to read as a small preview rather than a full pane.
const POPUP_WIDTH: u16 = 25;

/// A rect centered in `area`, clamped so it never exceeds `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Number of calendar-grid rows (weeks) needed to show every day of a month
/// that starts on `first_weekday` (`0` = Sunday) and has `days_in_month`
/// days.
fn week_rows(first_weekday: u32, days_in_month: u32) -> u16 {
    ((first_weekday + days_in_month).div_ceil(7)) as u16
}

pub fn draw(frame: &mut Frame, area: Rect, palette: &Palette, month_offset: i32) {
    let today = CivilDate::today();
    let viewed = today.month_shifted(month_offset);
    let days_in_month = viewed.days_in_month();
    let first_weekday = viewed.weekday(); // 0 = Sunday
    let rows_needed = week_rows(first_weekday, days_in_month);

    let popup = centered_rect(POPUP_WIDTH, rows_needed + 3, area);
    frame.render_widget(Clear, popup);

    let title = format!(" {} {} ", viewed.month_name(), viewed.year);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.accent));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut constraints = vec![Constraint::Length(1)]; // weekday header
    constraints.extend(std::iter::repeat_n(
        Constraint::Length(1),
        rows_needed as usize,
    ));
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let header = Line::from(
        WEEKDAY_HEADERS
            .iter()
            .map(|d| Span::styled(format!("{d} "), Style::default().fg(palette.muted)))
            .collect::<Vec<_>>(),
    );
    frame.render_widget(Paragraph::new(header), rows[0]);

    let is_current_month = month_offset == 0;
    for week in 0..rows_needed as i64 {
        let mut spans = Vec::with_capacity(7);
        for col in 0..7i64 {
            let day = week * 7 + col - first_weekday as i64 + 1;
            let in_month = day >= 1 && day <= days_in_month as i64;
            let text = if in_month {
                format!("{day:>2}")
            } else {
                "  ".to_string()
            };
            let is_today = is_current_month && in_month && day == today.day as i64;
            let style = if is_today {
                Style::default()
                    .fg(palette.bg)
                    .bg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.fg)
            };
            spans.push(Span::styled(text, style));
            spans.push(Span::raw(" "));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), rows[week as usize + 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_rows_counts_a_month_that_fits_five_weeks() {
        // Sunday start, 30-day month: 30 days over 5 full weeks.
        assert_eq!(week_rows(0, 30), 5);
    }

    #[test]
    fn week_rows_counts_a_month_that_spills_into_a_sixth_week() {
        // Saturday start, 31-day month needs 6 grid rows.
        assert_eq!(week_rows(6, 31), 6);
    }

    #[test]
    fn week_rows_counts_the_minimum_four_week_case() {
        // Sunday start, 28-day month (Feb, non-leap) fits exactly 4 weeks.
        assert_eq!(week_rows(0, 28), 4);
    }
}
