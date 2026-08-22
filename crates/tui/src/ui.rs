use ratatui::Frame;

use tmr_core::app::App;

use crate::image_backend::ImageCapability;
use crate::layout::compute_panes;
use crate::state::UiState;
use crate::theme::Palette;
use crate::widgets::{document_view, file_list, side_panel, status_bar};

pub fn draw(
    frame: &mut Frame,
    app: &App,
    ui: &UiState,
    palette: &Palette,
    _image_cap: ImageCapability,
) {
    let border = app.config.ui.border;
    let has_widgets = app.widgets().iter().any(|w| w.is_enabled());
    let panes = compute_panes(frame.area(), has_widgets);

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
    document_view::draw(
        frame,
        panes.document,
        &ui.rendered,
        ui.doc_cursor,
        ui.doc_scroll,
        ui.focus,
        palette,
        border,
        empty_hint,
    );

    if let Some(side) = panes.side {
        side_panel::draw(frame, side, app.widgets(), palette, border);
    }

    let doc_name = app.document().map(|d| d.name());
    let dirty = ui.editor.as_ref().map(|e| e.is_dirty()).unwrap_or(false);
    status_bar::draw(frame, panes.status, ui, doc_name, dirty, border, palette);
}
