//! Addon (plugin) abstraction.
//!
//! **Scope of this version:** addons are Rust structs implementing the
//! [`Addon`] trait, compiled directly into the `tmr` binary and registered
//! at startup (gated by `[addons] enabled = [...]` in `config.toml`). There
//! is **no dynamic loading** (`.so`/`.dylib`) in this version — Rust's ABI
//! instability and the complexity of a safe plugin ABI make that a poor fit
//! for a v1. This trait is the seam a future version could grow dynamic or
//! WASM-based loading behind, without changing how addons are written or
//! how the engine dispatches events to them.

use crate::events::AppEvent;
use crate::workspace::Workspace;

/// Read-only context handed to an addon when it's loaded.
pub struct AddonContext<'a> {
    pub workspace: &'a Workspace,
}

pub trait Addon {
    /// Stable identifier used in config (`[addons] enabled = ["stats"]`).
    fn id(&self) -> &str;

    /// Called once at startup, after the workspace is known.
    fn on_load(&mut self, _ctx: &AddonContext) {}

    /// Called after every successfully dispatched command.
    fn on_event(&mut self, _event: &AppEvent) {}

    /// Optional single-line status text contributed to the status bar.
    fn status_text(&self) -> Option<String> {
        None
    }
}

/// Holds the set of addons enabled for this session and fans events out to
/// them. Registration is static (see module docs) — built by `main.rs`
/// from the list of addons compiled into the binary, filtered by config.
#[derive(Default)]
pub struct AddonRegistry {
    addons: Vec<Box<dyn Addon>>,
}

impl AddonRegistry {
    pub fn new() -> Self {
        AddonRegistry::default()
    }

    pub fn register(&mut self, addon: Box<dyn Addon>) {
        self.addons.push(addon);
    }

    pub fn on_load(&mut self, ctx: &AddonContext) {
        for addon in &mut self.addons {
            addon.on_load(ctx);
        }
    }

    pub fn notify(&mut self, event: &AppEvent) {
        for addon in &mut self.addons {
            addon.on_event(event);
        }
    }

    pub fn status_texts(&self) -> Vec<String> {
        self.addons.iter().filter_map(|a| a.status_text()).collect()
    }
}

/// Minimal example addon that counts how many files were opened/saved/
/// created/deleted during the session. It exists to prove the [`Addon`]
/// trait out end-to-end, not as a real feature.
#[derive(Default)]
pub struct StatsAddon {
    opened: u32,
    saved: u32,
    created: u32,
    deleted: u32,
}

impl Addon for StatsAddon {
    fn id(&self) -> &str {
        "stats"
    }

    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::DocumentOpened { .. } => self.opened += 1,
            AppEvent::DocumentSaved { .. } => self.saved += 1,
            AppEvent::FileCreated { .. } => self.created += 1,
            AppEvent::FileDeleted { .. } => self.deleted += 1,
            _ => {}
        }
    }

    fn status_text(&self) -> Option<String> {
        if self.opened == 0 && self.saved == 0 && self.created == 0 && self.deleted == 0 {
            return None;
        }
        Some(format!(
            "opened:{} saved:{} created:{} deleted:{}",
            self.opened, self.saved, self.created, self.deleted
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stats_addon_counts_events() {
        let mut addon = StatsAddon::default();
        assert_eq!(addon.status_text(), None);
        addon.on_event(&AppEvent::DocumentOpened {
            path: PathBuf::from("a.md"),
        });
        addon.on_event(&AppEvent::DocumentSaved {
            path: PathBuf::from("a.md"),
        });
        assert_eq!(
            addon.status_text(),
            Some("opened:1 saved:1 created:0 deleted:0".to_string())
        );
    }

    #[test]
    fn registry_fans_out_events_to_all_addons() {
        let mut registry = AddonRegistry::new();
        registry.register(Box::new(StatsAddon::default()));
        registry.notify(&AppEvent::FileCreated {
            path: PathBuf::from("new.md"),
        });
        assert_eq!(
            registry.status_texts(),
            vec!["opened:0 saved:0 created:1 deleted:0"]
        );
    }
}
