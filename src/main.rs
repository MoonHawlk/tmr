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
    println!(
        "tmr {} — Terminal Markdown Reader",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("A fast, low-footprint terminal UI for browsing, reading and editing");
    println!("Markdown notes.");
    println!();
    println!("Usage:");
    println!("  tmr [DIRECTORY]");
    println!("  tmr -h | --help");
    println!("  tmr -V | --version");
    println!();
    println!("Arguments:");
    println!("  DIRECTORY   Directory to open. If omitted, tmr uses (in order):");
    println!("              workspace.default_dir from config.toml, otherwise the");
    println!("              current working directory.");
    println!();
    println!("Options:");
    println!("  -h, --help     Print this help and exit");
    println!("  -V, --version  Print the version and exit");
    println!();
    println!("Key bindings (defaults; press 'h' inside tmr for the full, current");
    println!("reference, or see docs/keybindings.md — all of these are remappable):");
    println!("  tab              switch focus between Files and Document panes");
    println!("  up/down          move selection (files) or cursor (document)");
    println!("  right/enter      open selected entry; in Document, enter starts editing");
    println!("  left             go to parent directory (Files pane)");
    println!("  space            toggle the task checkbox under the cursor");
    println!("  /                search (filenames or in-document text)");
    println!("  ctrl+n           new file        r   rename        d   delete");
    println!("  ctrl+s           save (Edit mode)   esc   cancel / leave Edit mode");
    println!("  ctrl+t           Quick-TODO window   ctrl+e   export tasks to .tsv");
    println!("  s                Settings window     alt+c    Calendar window");
    println!("  ctrl+r           reload directory + config.toml");
    println!("  h                command-reference popup (searchable)");
    println!("  q                quit");
    println!();
    println!("Try it risk-free:  tmr sandbox");
}

fn print_version() {
    println!("tmr {}", env!("CARGO_PKG_VERSION"));
}

fn parse_args() -> Result<Option<PathBuf>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(None),
        Some("-h") | Some("--help") => {
            print_usage();
            std::process::exit(0);
        }
        Some("-V") | Some("--version") => {
            print_version();
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

    let tasks_path = tmr_core::config::default_tasks_path();
    let mut app = App::new(workspace, config, keymap, theme, tasks_path);

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
