use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};

use tmr_core::config::BorderStyle;

use crate::theme::Palette;

/// The panes making up the main screen. Computed as a pure function of
/// terminal size so both drawing and input handling (which needs to know
/// the document pane's height/width for scrolling and rendering) agree.
pub struct Panes {
    pub timer: Option<Rect>,
    pub files: Rect,
    pub document: Rect,
    pub side: Option<Rect>,
    pub status: Rect,
}

pub fn compute_panes(size: Rect, has_widgets: bool, show_timer: bool) -> Panes {
    let outer_constraints = if show_timer {
        vec![
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ]
    } else {
        vec![Constraint::Min(0), Constraint::Length(3)]
    };
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints(outer_constraints)
        .split(size);
    let (timer, main_area, status) = if show_timer {
        (Some(outer[0]), outer[1], outer[2])
    } else {
        (None, outer[0], outer[1])
    };

    let main_constraints = if has_widgets {
        vec![
            Constraint::Percentage(22),
            Constraint::Min(20),
            Constraint::Length(28),
        ]
    } else {
        vec![Constraint::Percentage(25), Constraint::Min(20)]
    };
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(main_constraints)
        .split(main_area);

    Panes {
        timer,
        files: main[0],
        document: main[1],
        side: if has_widgets { Some(main[2]) } else { None },
        status,
    }
}

/// Interior width/height of `area` once a border (if any) is subtracted.
pub fn inner_size(area: Rect, border: BorderStyle) -> (u16, u16) {
    if border == BorderStyle::None {
        (area.width, area.height)
    } else {
        (area.width.saturating_sub(2), area.height.saturating_sub(2))
    }
}

const ASCII_BORDER: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

/// A denser, more "application UI" panel look than `ASCII_BORDER` — double
/// box-drawing lines instead of a plain `+---+` sketch.
const DOUBLE_BORDER: border::Set = border::Set {
    top_left: "\u{2554}",          // ╔
    top_right: "\u{2557}",         // ╗
    bottom_left: "\u{255a}",       // ╚
    bottom_right: "\u{255d}",      // ╝
    vertical_left: "\u{2551}",     // ║
    vertical_right: "\u{2551}",    // ║
    horizontal_top: "\u{2550}",    // ═
    horizontal_bottom: "\u{2550}", // ═
};

/// Builds a bordered [`Block`] with a title, styled from the configured
/// border style and the active theme. Focused panes get the accent color
/// so the user always knows where input is going.
pub fn styled_block(
    title: &str,
    border_style: BorderStyle,
    palette: &Palette,
    focused: bool,
) -> Block<'static> {
    let color = if focused {
        palette.accent
    } else {
        palette.border
    };
    let block = Block::default()
        .title(format!(" {title} "))
        .border_style(Style::default().fg(color));

    match border_style {
        BorderStyle::None => block.borders(Borders::NONE),
        BorderStyle::Rounded => block
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded),
        BorderStyle::Ascii => block.borders(Borders::ALL).border_set(ASCII_BORDER),
        BorderStyle::Double => block.borders(Borders::ALL).border_set(DOUBLE_BORDER),
    }
}
