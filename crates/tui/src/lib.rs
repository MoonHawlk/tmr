pub mod clipboard;
pub mod editor;
pub mod help;
pub mod image_backend;
pub mod input;
pub mod keymap;
pub mod layout;
pub mod markdown_view;
pub mod settings;
pub mod state;
pub mod theme;
pub mod ui;
pub mod widgets;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use tmr_core::app::App;
use tmr_core::command::Command;

use crate::input::ControlFlow;
use crate::layout::{compute_panes, inner_size};
use crate::state::{Mode, UiState};
use crate::theme::Palette;
use crate::widgets::document_view;

/// Restores the terminal on drop, so a panic or early return can't leave
/// the user's shell in raw/alternate-screen mode.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

/// Runs tmr's terminal UI until the user quits. Owns the `App` engine for
/// the duration of the session; `app` is expected to already have its
/// workspace, config, keymap and theme set up (see `tmr`'s `main.rs`).
pub fn run(mut app: App) -> io::Result<()> {
    let _guard = TerminalGuard;
    let mut terminal = setup_terminal()?;
    run_loop(&mut terminal, &mut app)
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    let mut ui = UiState {
        default_theme: app.theme.clone(),
        default_theme_name: app.config.theme.name.clone(),
        line_indicator: crate::state::LineIndicatorStyle::from_config_str(
            &app.config.ui.line_indicator,
        ),
        ..UiState::default()
    };
    let image_cap = image_backend::detect_capability();

    let root = app.workspace().root().to_path_buf();
    match app.dispatch(Command::ListDir(root)) {
        Ok(_) => {}
        Err(e) => ui.set_status(e.to_string(), crate::state::StatusLevel::Error),
    }

    loop {
        // A transient status message (e.g. "Saved") reverts to the default
        // helper bar on its own after `state::STATUS_TTL`, even with no
        // further keypress — see `UiState::expire_status`.
        ui.expire_status();

        // Recomputed every iteration (all cheap: a handful of hex-string
        // parses, and small-map/plain-field reads) rather than once
        // outside the loop, since the Settings window can change
        // `app.theme` live and `ctrl+r`'s config reload
        // (`input.rs::reload_config`) can change `app.keymap`/
        // `app.config` live — this keeps every frame and every keypress in
        // sync without needing a separate "config changed" flag.
        let palette = Palette::from_theme(&app.theme);
        let resolved_keymap = app.keymap.resolve();
        let tab_width = app.config.editor.tab_width;
        terminal.draw(|f| ui::draw(f, app, &ui, &palette, image_cap))?;

        let has_ticking_widget = app
            .widgets()
            .iter()
            .any(|w| w.is_enabled() && w.tick_interval().is_some());
        // The Timer bar reads the clock fresh on every draw rather than
        // keeping its own tick state, so it just needs the loop to keep
        // redrawing periodically while it's enabled — same as a ticking
        // widget.
        let needs_periodic_redraw = has_ticking_widget || app.config.ui.timer;
        let timeout = if needs_periodic_redraw {
            Duration::from_millis(500)
        } else if ui.status.is_some() {
            // A status message is pending expiry: poll periodically instead
            // of blocking indefinitely, so it gets cleared (and the bar
            // redrawn) close to `STATUS_TTL` even if the user is idle.
            Duration::from_millis(250)
        } else {
            // No widget needs periodic redraws: block indefinitely on the
            // next terminal event rather than polling.
            Duration::from_secs(u64::MAX / 2)
        };

        if !event::poll(timeout)? {
            app.tick_widgets();
            continue;
        }

        let Event::Key(key_event) = event::read()? else {
            continue;
        };
        if key_event.kind != KeyEventKind::Press {
            continue;
        }
        let Some(key) = keymap::to_core_key(key_event) else {
            continue;
        };

        let size = terminal.size()?;
        let has_widgets = app.widgets().iter().any(|w| w.is_enabled());
        let panes = compute_panes(
            ratatui::layout::Rect::new(0, 0, size.width, size.height),
            has_widgets,
            app.config.ui.timer,
        );
        let border = app.config.ui.border;
        let (doc_width, doc_height) = inner_size(panes.document, border);

        match input::handle_key(
            &mut ui,
            app,
            &resolved_keymap,
            key,
            tab_width,
            &palette,
            image_cap,
            doc_width,
        ) {
            ControlFlow::Quit => break,
            ControlFlow::Continue => {}
        }
        ui.ensure_doc_visible(doc_height as usize);
        if let Mode::Edit = ui.mode {
            if let Some(editor) = &ui.editor {
                let gutter = document_view::gutter_cols(ui.rendered.len());
                let available = doc_width.saturating_sub(gutter) as usize;
                ui.ensure_doc_hscroll(editor.cursor().1, available);
            }
        } else {
            ui.doc_hscroll = 0;
        }
    }
    Ok(())
}
