# sandbox

A disposable playground. Point tmr at this directory to try every feature
without touching your real notes:

```sh
cargo run --release -- sandbox
# or, once built:
./target/release/tmr sandbox
```

## What's safe to do here

Anything. Create files, edit them, delete them, toggle checkboxes,
rename things, make a mess. This directory exists specifically to be
experimented on.

Two example files (`welcome.md`, `showcase.md`) and this `logo.png`
ship with the repo so there's something to open immediately. Everything
*else* you create in here — new notes, subfolders, whatever — is
git-ignored (see the `sandbox/*` block near the end of the repo's
`.gitignore`), so your test scratch never shows up in `git status`. If
you edit or delete the shipped example files themselves, that *is* a
normal tracked change — `git checkout -- sandbox/welcome.md` (etc.)
puts them back if you want a clean slate.

## Where to start

Open `welcome.md` first (it has a short task list to try toggling with
`Space`), then `showcase.md` for a tour of every Markdown element tmr
renders, including a real embedded image (`logo.png`) to see the
half-block image renderer in action if your terminal supports truecolor.
