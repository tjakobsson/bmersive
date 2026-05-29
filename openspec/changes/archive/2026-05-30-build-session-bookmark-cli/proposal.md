## Why

Developers often juggle a small set of directories while working on a focused session, but existing directory-jump tools emphasize history, frequency, and fuzzy ranking. `bmersive` should provide an explicit, curated working set that can be listed, jumped to, and materialized into terminal multiplexer layouts without hidden ranking or recency behavior.

## What Changes

- Add a Rust CLI named `bmersive` for managing a small session-local bookmark list.
- Add shell initialization commands for zsh and bash that define a `b` function.
- Support indexed bookmark listing, path lookup, add, and remove workflows.
- Store bookmarks in a session-oriented JSON state file under `$BMERSIVE_STATE_DIR/bookmarks.json` when shell integration is active, otherwise `$XDG_RUNTIME_DIR/bmersive/bookmarks.json`, with a `/tmp/bmersive-$USER/bookmarks.json` fallback.
- Enforce no duplicates, manual insertion order, and a configurable maximum bookmark count using `BMERSIVE_MAX_BOOKMARKS`, defaulting to 10.
- Add tmux materialization commands, defaulting to tiled panes and supporting explicit one-window-per-bookmark mode.
- Keep dependencies minimal and avoid copyleft-licensed dependencies.

## Capabilities

### New Capabilities
- `session-bookmarks`: Manage a curated session-local directory bookmark list with shell-friendly indexed lookup.
- `shell-integration`: Emit zsh and bash wrapper functions that allow `b <index>` to change the parent shell directory.
- `tmux-materialization`: Create or reuse tmux layouts from the current bookmark list.

### Modified Capabilities

- None.

## Impact

- Adds a new Rust command-line application structure.
- Adds session-local JSON state management.
- Adds shell wrapper generation for zsh and bash.
- Adds tmux integration through the `tmux` executable.
- Introduces runtime behavior controlled by `BMERSIVE_MAX_BOOKMARKS`.
