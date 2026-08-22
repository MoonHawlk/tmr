#!/usr/bin/env bash
# setup.sh — quick install and environment prep for working on tmr.
#
# What it does, in order:
#   1. Checks for a Rust toolchain (cargo/rustc); offers to install one via
#      rustup if missing.
#   2. Fetches crate dependencies (`cargo fetch`).
#   3. Builds the release binary (`cargo build --release`).
#   4. Offers to install the binary onto your PATH (~/.local/bin by default).
#   5. Offers to create ~/.config/tmr/config.toml from the example, if you
#      don't already have one.
#
# Safe to re-run any time — every step either checks first or asks before
# touching anything outside the repo.
#
# Usage:
#   ./setup.sh              interactive (asks before installing anything)
#   ./setup.sh --yes         non-interactive, accepts every prompt
#   ./setup.sh --no-install  build only, skip PATH/config install steps

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

ASSUME_YES=0
DO_INSTALL=1
for arg in "$@"; do
    case "$arg" in
        --yes|-y) ASSUME_YES=1 ;;
        --no-install) DO_INSTALL=0 ;;
        --help|-h)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "setup.sh: unknown argument: $arg" >&2
            exit 1
            ;;
    esac
done

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
step() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
ok()   { printf '  \033[1;32m✓\033[0m %s\n' "$*"; }
warn() { printf '  \033[1;33m!\033[0m %s\n' "$*"; }

confirm() {
    local prompt="$1"
    if [ "$ASSUME_YES" -eq 1 ]; then
        return 0
    fi
    read -r -p "$prompt [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

bold "tmr — environment setup"

step "Checking for a Rust toolchain"
if ! command -v cargo >/dev/null 2>&1; then
    warn "cargo not found on PATH"
    if confirm "Install Rust via rustup now?"; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # rustup installs to ~/.cargo/bin; make it visible to the rest of
        # this script without requiring a new shell.
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    else
        echo "cargo is required to build tmr. Install Rust (https://rustup.rs) and re-run this script." >&2
        exit 1
    fi
else
    ok "cargo found: $(command -v cargo)"
fi
ok "$(rustc --version)"
ok "$(cargo --version)"

step "Fetching dependencies"
cargo fetch
ok "dependencies fetched"

step "Building tmr (release)"
cargo build --release --workspace
binary="$repo_root/target/release/tmr"
if [ -x "$binary" ]; then
    ok "built: $binary"
else
    echo "build finished but $binary is missing — something went wrong." >&2
    exit 1
fi

if [ "$DO_INSTALL" -eq 1 ]; then
    step "Installing onto your PATH"
    install_dir="$HOME/.local/bin"
    if confirm "Install tmr to $install_dir?"; then
        mkdir -p "$install_dir"
        install -m 755 "$binary" "$install_dir/tmr"
        ok "installed: $install_dir/tmr"
        case ":$PATH:" in
            *":$install_dir:"*) ;;
            *) warn "$install_dir is not on your PATH — add it in your shell profile." ;;
        esac
    else
        warn "skipped; run manually with: install -m 755 $binary ~/.local/bin/tmr"
    fi

    step "Local configuration"
    config_dir="$HOME/.config/tmr"
    config_file="$config_dir/config.toml"
    if [ -f "$config_file" ]; then
        ok "config already exists: $config_file (left untouched)"
    elif confirm "Create $config_file from config/config.example.toml?"; then
        mkdir -p "$config_dir"
        cp "$repo_root/config/config.example.toml" "$config_file"
        ok "created: $config_file"
    else
        warn "skipped; tmr runs fine with built-in defaults until you want to customize it"
    fi
else
    ok "skipped install/config steps (--no-install)"
fi

step "Done"
echo "Try it risk-free:  $binary sandbox"
echo "Run the debug helper any time with: ./debug.sh"
