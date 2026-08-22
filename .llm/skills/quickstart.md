---
name: quickstart
description: Build tmr and run it for the first time, including the disposable sandbox/ playground.
---

# Quickstart

## Build

Requires a Rust toolchain (stable, 2021 edition; developed/tested against
1.98). No network access is needed at runtime, only at build time (to
fetch crates the first time).

```sh
cargo build --release
# binary at target/release/tmr
```

Debug builds (`cargo build`, `cargo run`) work too and compile faster;
use them while iterating.

## Run against the built-in sandbox

The repo ships a `sandbox/` directory specifically for trying tmr out
without touching real notes — see [`../../sandbox/README.md`](../../sandbox/README.md)
for what's in it. This is the fastest way to see the app working:

```sh
cargo run --release -- sandbox
# or, once built:
./target/release/tmr sandbox
```

Everything you create, edit, or delete inside `sandbox/` beyond the
shipped example files is gitignored (see the `sandbox/*` block near the
end of `.gitignore`) — experiment freely, it won't show up in `git
status`.

## Run against any other directory

```sh
tmr                 # current working directory
tmr ~/notes          # a specific directory
tmr --help
```

If no directory is given, tmr uses `[workspace] default_dir` from
`~/.config/tmr/config.toml` if set, otherwise the current working
directory.

## Driving it non-interactively (for automated smoke tests)

tmr is a full-screen terminal app (raw mode + alternate screen), so
piping keys into it via plain stdin redirection does not work the way it
would for a line-oriented CLI. To drive it from a script or an agent
without a human at the keyboard, use a real pty:

```sh
# tmux: launch, send keys, capture the screen as text
tmux new-session -d -s tmrtest -x 100 -y 30 "tmr sandbox"
sleep 0.3
tmux send-keys -t tmrtest Down Enter   # select+open the first file
sleep 0.3
tmux capture-pane -t tmrtest -p        # dump the rendered screen as text
tmux send-keys -t tmrtest q            # quit
tmux kill-session -t tmrtest 2>/dev/null
```

`script -qec "tmr sandbox" /dev/null` also works for a quick one-shot
pty session. Piping into plain `tmr < input.txt` will hang or misbehave —
don't do that.

Exit code is `0` on a normal `q` quit, non-zero (with a message on
stderr, via `anyhow` context) for startup failures such as a missing
workspace directory.
