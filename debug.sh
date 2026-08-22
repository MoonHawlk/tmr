#!/usr/bin/env bash
# debug.sh — quick inspection of the tmr workspace: toolchain info, a debug
# build, lint/format status, test results, and (optionally) launching tmr
# itself against the disposable sandbox/ with a live backtrace, for fast
# iteration while developing.
#
# Usage:
#   ./debug.sh              build + check + clippy + test (no run)
#   ./debug.sh run [DIR]     also launch tmr (default DIR: sandbox/)
#   ./debug.sh --quick       build only, skip clippy/fmt/test
#   ./debug.sh --help

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

if command -v cargo >/dev/null 2>&1; then
    :
elif [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "debug.sh: cargo not found — run ./setup.sh first." >&2
    exit 1
fi

RUN=0
RUN_DIR="sandbox"
QUICK=0
for arg in "$@"; do
    case "$arg" in
        run) RUN=1 ;;
        --quick) QUICK=1 ;;
        --help|-h)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            if [ "$RUN" -eq 1 ]; then
                RUN_DIR="$arg"
            else
                echo "debug.sh: unknown argument: $arg" >&2
                exit 1
            fi
            ;;
    esac
done

step() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
ok()   { printf '  \033[1;32m✓\033[0m %s\n' "$*"; }
info() { printf '  %s\n' "$*"; }

step "Toolchain"
info "$(rustc --version)"
info "$(cargo --version)"
info "target dir: $repo_root/target"

step "Config resolution"
config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/tmr"
config_file="$config_dir/config.toml"
if [ -f "$config_file" ]; then
    info "config file: $config_file (exists)"
else
    info "config file: $config_file (absent — built-in defaults apply)"
fi

step "cargo check (workspace)"
cargo check --workspace --all-targets
ok "check passed"

if [ "$QUICK" -eq 0 ]; then
    step "cargo clippy (workspace)"
    cargo clippy --workspace --all-targets
    ok "clippy passed"

    step "cargo fmt --check"
    if cargo fmt --all -- --check; then
        ok "formatting is clean"
    else
        echo "  run 'cargo fmt --all' to fix formatting." >&2
    fi

    step "cargo test (workspace)"
    cargo test --workspace
    ok "tests passed"
fi

step "Building debug binary"
cargo build --workspace
binary="$repo_root/target/debug/tmr"
ok "built: $binary"

if [ "$RUN" -eq 1 ]; then
    step "Launching tmr against '$RUN_DIR' (RUST_BACKTRACE=full)"
    RUST_BACKTRACE=full "$binary" "$RUN_DIR"
else
    step "Done"
    echo "Launch it with a live backtrace: ./debug.sh run [DIR]"
fi
