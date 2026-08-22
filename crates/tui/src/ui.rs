use ratatui::Frame;

use tmr_core::app::App;

use crate::calendar;
use crate::help;
use crate::image_backend::ImageCapability;
use crate::layout::compute_panes;
use crate::settings;
use crate::state::{Mode, UiState};
use crate::theme::Palette;
use crate::widgets::{document_view, file_list, side_panel, status_bar, timer_bar};

pub fn draw(
    frame: &mut Frame,
    app: &App,
    ui: &UiState,
    palette: &Palette,
    _image_cap: ImageCapability,
) {
    let border = app.config.ui.border;
    let has_widgets = app.widgets().iter().any(|w| w.is_enabled());
    let panes = compute_panes(frame.area(), has_widgets, app.config.ui.timer);

    if let Some(timer_area) = panes.timer {
        timer_bar::draw(frame, timer_area, palette);
    }

    file_list::draw(
        frame,
        panes.files,
        app.entries(),
        ui.selected,
        ui.focus,
        palette,
        border,
    );

    let empty_hint = if app.document().is_none() {
        Some("Select a .md file and press Enter to open it.")
    } else {
        None
    };
    let selection = if let Mode::Edit = ui.mode {
        ui.editor.as_ref().and_then(|e| e.selection_range())
    } else {
        None
    };
    let doc_inner = document_view::draw(
        frame,
        panes.document,
        &ui.rendered,
        ui.doc_cursor,
        ui.doc_scroll,
        ui.focus,
        palette,
        border,
        empty_hint,
        ui.line_indicator,
        selection,
        ui.doc_hscroll,
    );

    if let Some(side) = panes.side {
        side_panel::draw(frame, side, app.widgets(), palette, border);
    }

    let doc_name = app.document().map(|d| d.name());
    let dirty = ui.editor.as_ref().map(|e| e.is_dirty()).unwrap_or(false);
    status_bar::draw(frame, panes.status, ui, doc_name, dirty, border, palette);

    // A real, blinking terminal cursor is the clearest possible "where am I
    // typing" indicator — placed here (after the document pane has been
    // drawn against the raw-source view refresh_rendered switches to while
    // editing) so its row/col line up with what's on screen.
    if let Mode::Edit = ui.mode {
        if let Some(editor) = &ui.editor {
            let (row, col) = editor.cursor();
            // Must match `document_view::draw`'s gutter exactly, or the
            // cursor lands on the wrong column.
            let gutter = document_view::gutter_cols(ui.rendered.len());
            if row >= ui.doc_scroll
                && col >= ui.doc_hscroll
                && doc_inner.height > 0
                && doc_inner.width > gutter
            {
                let y = doc_inner.y + (row - ui.doc_scroll) as u16;
                if y < doc_inner.y + doc_inner.height {
                    let available = doc_inner.width - gutter;
                    let x =
                        doc_inner.x + gutter + ((col - ui.doc_hscroll) as u16).min(available - 1);
                    frame.set_cursor_position((x, y));
                }
            }
        }
    }

    if let Mode::Help { query, selected } = &ui.mode {
        help::draw(frame, frame.area(), palette, &app.keymap, query, *selected);
    }

    if let Mode::Settings { row } = &ui.mode {
        settings::draw(
            frame,
            frame.area(),
            palette,
            ui.theme_choice,
            ui.line_indicator,
            app.config.ui.timer,
            *row,
        );
    }

    if let Mode::Calendar { month_offset } = &ui.mode {
        calendar::draw(frame, frame.area(), palette, *month_offset);
    }
}
