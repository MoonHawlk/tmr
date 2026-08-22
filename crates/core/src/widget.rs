//! Abstraction for small, self-contained UI widgets (a clock, a quick TODO
//! panel, a calendar...). Only one trivial example widget ships in this
//! version ([`ClockWidget`]) — its purpose is to validate the trait shape,
//! not to be a feature. The trait deliberately renders to plain text lines
//! rather than a toolkit-specific type, so `tmr-core` never has to depend
//! on ratatui: the TUI layer draws whatever lines a widget returns inside
//! its own box.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::events::AppEvent;

/// A pluggable panel that can be enabled/disabled and configured entirely
/// from `config.toml`, without recompiling.
pub trait Widget {
    /// Stable identifier used in config (`[widgets] enabled = ["clock"]`).
    fn id(&self) -> &str;

    /// Human-readable title shown in the widget's border.
    fn title(&self) -> &str;

    /// Called once at startup with the widget's own config sub-table, if
    /// present under `[widgets.<id>]`. Implementations ignore keys they
    /// don't understand rather than erroring.
    fn configure(&mut self, _config: &toml::Value) {}

    /// Whether the widget should currently be shown.
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, enabled: bool);

    /// Lets ticking widgets (like a clock) ask for periodic redraws. `None`
    /// means the widget only needs to redraw in response to input/events.
    fn tick_interval(&self) -> Option<std::time::Duration> {
        None
    }

    /// Advances internal state on each scheduled tick (see `tick_interval`).
    fn tick(&mut self) {}

    /// Notifies the widget of something that happened in the engine (a
    /// file was saved, a task toggled, ...). Default is a no-op; widgets
    /// that care about app state override this.
    fn on_event(&mut self, _event: &AppEvent) {}

    /// Renders the widget's body as plain text lines.
    fn render_lines(&self) -> Vec<String>;
}

/// Minimal example widget: shows the current time. Exists to prove out the
/// [`Widget`] trait end-to-end (enable/disable, tick, render) with no
/// external dependencies.
#[derive(Default)]
pub struct ClockWidget {
    enabled: bool,
    now_secs: u64,
}

impl Widget for ClockWidget {
    fn id(&self) -> &str {
        "clock"
    }

    fn title(&self) -> &str {
        "Clock"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn tick_interval(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(1))
    }

    fn tick(&mut self) {
        self.now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    fn render_lines(&self) -> Vec<String> {
        let secs_of_day = self.now_secs % 86400;
        let h = secs_of_day / 3600;
        let m = (secs_of_day % 3600) / 60;
        let s = secs_of_day % 60;
        vec![format!("{h:02}:{m:02}:{s:02} UTC")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_widget_starts_disabled() {
        let w = ClockWidget::default();
        assert!(!w.is_enabled());
        assert_eq!(w.id(), "clock");
    }

    #[test]
    fn clock_widget_ticks_and_renders() {
        let mut w = ClockWidget::default();
        w.set_enabled(true);
        w.tick();
        let lines = w.render_lines();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains(':'));
    }
}
