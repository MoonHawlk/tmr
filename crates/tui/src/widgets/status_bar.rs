use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use tmr_core::config::BorderStyle;

use crate::layout::styled_block;
use crate::state::{Mode, PromptKind, StatusLevel, UiState};
use crate::theme::Palette;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    ui: &UiState,
    doc_name: Option<&str>,
    dirty: bool,
    border: BorderStyle,
    palette: &Palette,
) {
    let block = styled_block("STATUS", border, palette, false);

    let line = match &ui.mode {
        Mode::Search { buffer, .. } => Line::from(vec![
            Span::styled("Search: ", Style::default().fg(palette.accent)),
            Span::raw(buffer.clone()),
            Span::styled("_", Style::default().fg(palette.muted)),
        ]),
        Mode::Prompt { kind, buffer } => {
            let label = match kind {
                PromptKind::NewFile => "New file name: ",
                PromptKind::Rename { .. } => "Rename to: ",
            };
            Line::from(vec![
                Span::styled(label, Style::default().fg(palette.accent)),
                Span::raw(buffer.clone()),
                Span::styled("_", Style::default().fg(palette.muted)),
            ])
        }
        Mode::Confirm { message, .. } => Line::from(Span::styled(
            message.clone(),
            Style::default().fg(palette.warning),
        )),
        Mode::Edit => {
            let name = doc_name.unwrap_or("untitled");
            let mark = if dirty { "*" } else { "" };
            Line::from(vec![
                Span::styled("EDIT ", Style::default().fg(palette.accent)),
                Span::raw(format!("{name}{mark}  ")),
                Span::styled("ctrl+s save · esc back", Style::default().fg(palette.muted)),
            ])
        }
        Mode::Normal => {
            if let Some((msg, level)) = &ui.status {
                let color = match level {
                    StatusLevel::Info => palette.fg,
                    StatusLevel::Success => palette.success,
                    StatusLevel::Warning => palette.warning,
                    StatusLevel::Error => palette.error,
                };
                Line::from(Span::styled(msg.clone(), Style::default().fg(color)))
            } else {
                let name = doc_name.unwrap_or("(no file open)");
                let mark = if dirty { "*" } else { "" };
                Line::from(vec![
                    Span::raw(format!("{name}{mark}  ")),
                    Span::styled(
                        "tab focus · enter open/edit · space toggle · / search · ctrl+n new · d delete · q quit",
                        Style::default().fg(palette.muted),
                    ),
                ])
            }
        }
    };

    frame.render_widget(Paragraph::new(line).block(block), area);
}
