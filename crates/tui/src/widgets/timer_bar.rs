//! The optional Timer bar: a single-line strip showing the current time
//! (UTC), drawn at the very top of the TUI — above the Files/Document
//! panes, below the terminal's top edge — when `[ui] timer = true`. See
//! `layout::compute_panes`'s `timer` slot and `tmr_core::datetime`.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use tmr_core::datetime;

use crate::theme::Palette;

pub fn draw(frame: &mut Frame, area: Rect, palette: &Palette) {
    let (h, m, s) = datetime::time_of_day(datetime::now_unix_secs());
    let line = Line::from(vec![Span::styled(
        format!("{h:02}:{m:02}:{s:02} UTC"),
        Style::default().fg(palette.accent),
    )]);
    frame.render_widget(
        Paragraph::new(line).alignment(ratatui::layout::Alignment::Center),
        area,
    );
}
