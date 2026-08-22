use std::path::PathBuf;

use anyhow::{Context, Result};

use tmr_core::addon::StatsAddon;
use tmr_core::app::App;
use tmr_core::config::Config;
use tmr_core::keymap::Keymap;
use tmr_core::theme::Theme;
use tmr_core::widget::{ClockWidget, Widget};
use tmr_core::workspace::Workspace;

fn print_usage() {
    println!("tmr — Terminal Markdown Reader");
    println!();
    println!("Usage:");
    println!("  tmr [DIRECTORY]");
    println!();
    println!("If DIRECTORY is omitted, tmr uses (in order): the workspace.default_dir");
    println!("set in config.toml, otherwise the current working directory.");
}

fn parse_args() -> Result<Option<PathBuf>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(None),
        Some("-h") | Some("--help") => {
            print_usage();
            std::process::exit(0);
        }
        Some(other) => Ok(Some(PathBuf::from(other))),
    }
}

fn main() -> Result<()> {
    let cli_dir = parse_args()?;

    let load_result = Config::load(None);
    let config = load_result.config;
    for warning in &load_result.warnings {
        eprintln!("tmr: warning: {warning}");
    }

    let workspace_dir = cli_dir
        .or_else(|| config.workspace.default_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));

    let workspace = Workspace::new(workspace_dir.clone()).with_context(|| {
        format!(
            "could not open workspace directory: {}",
            workspace_dir.display()
        )
    })?;

    let config_dir = tmr_core::config::default_config_dir();
    let (theme, theme_warning) = Theme::resolve(config_dir.as_deref(), &config.theme.name);
    if let Some(w) = theme_warning {
        eprintln!("tmr: warning: {w}");
    }

    let keymap = Keymap::with_overrides(config.keys.clone());

    let addons_enabled = config.addons.enabled.clone();
    let widgets_enabled = config.widgets.enabled.clone();

    let mut app = App::new(workspace, config, keymap, theme);

    if addons_enabled.iter().any(|a| a == "stats") {
        app.register_addon(Box::new(StatsAddon::default()));
    }
    app.load_addons();

    if widgets_enabled.iter().any(|w| w == "clock") {
        let mut clock = ClockWidget::default();
        clock.set_enabled(true);
        app.register_widget(Box::new(clock));
    }

    tmr_tui::run(app).context("terminal UI exited with an error")?;
    Ok(())
}
