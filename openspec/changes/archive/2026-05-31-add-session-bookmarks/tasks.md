## 1. State Model And Migration

- [x] 1.1 Add saved-session state structs for named sessions and per-session bookmark lists.
- [x] 1.2 Add durable sessions state path resolution with user state, home-local state, and `/tmp/bmersive-$USER` fallbacks.
- [x] 1.3 Add shell-local appointment path resolution under `BMERSIVE_STATE_DIR`.
- [x] 1.4 Implement load/save helpers for sessions state and appointed-session marker.
- [x] 1.5 Import legacy `bookmarks.json` into a `default` saved session when new sessions state is absent.
- [x] 1.6 Add unit tests for state path resolution, appointment path resolution, and legacy import precedence.

## 2. Session CLI

- [x] 2.1 Add `bmersive session` parser types and subcommands for `ls`, `new <name>`, `use <name>`, and `choose`.
- [x] 2.2 Validate session names with a conservative CLI-safe character set.
- [x] 2.3 Implement saved session creation with duplicate detection.
- [x] 2.4 Implement saved session listing with stable numeric indexes.
- [x] 2.5 Implement session appointment by name for the current shell runtime state.
- [x] 2.6 Implement interactive session chooser using numbered prompts and stdin.
- [x] 2.7 Add parser and behavior tests for supported and invalid session command forms.

## 3. Bookmark Commands

- [x] 3.1 Update `add`, `ls`, `path`, and `rm` to resolve the appointed session before accessing bookmarks.
- [x] 3.2 Scope duplicate detection, maximum bookmark count, ordering, path lookup, and removal to the appointed session.
- [x] 3.3 Return a clear error when direct bookmark commands run without an appointed session.
- [x] 3.4 Update existing bookmark tests to cover appointed-session behavior and cross-session isolation.

## 4. Shell Integration

- [x] 4.1 Update generated zsh and bash shell code to keep `BMERSIVE_STATE_DIR` as shell-local appointment state.
- [x] 4.2 Update `b` with no arguments to run session selection when no session is appointed, then list bookmarks.
- [x] 4.3 Delegate `b session ...` to `bmersive session ...`.
- [x] 4.4 Preserve numeric `b <index>`, `b add`, `b rm`, and `b tmux` behavior against the appointed session.
- [x] 4.5 Add tests for generated shell code containing the new session-selection and delegation behavior.

## 5. Tmux Integration

- [x] 5.1 Update tmux commands to load bookmarks from the appointed session.
- [x] 5.2 Derive outside-tmux session names from the appointed saved session to avoid project collisions.
- [x] 5.3 Preserve inside-tmux behavior while using appointed-session bookmarks.
- [x] 5.4 Update tmux action and argument tests for session-specific names and four-bookmark pane layouts.

## 6. README And Verification

- [x] 6.1 Update `README.md` state and usage sections for saved sessions and appointed shell sessions.
- [x] 6.2 Add a demo showing session chooser, bookmark chooser, and tmux usage.
- [x] 6.3 Add an ASCII diagram showing four project folders in a tmux split-window grid.
- [x] 6.4 Run `cargo fmt --check`.
- [x] 6.5 Run `cargo test`.
- [x] 6.6 Run `cargo build`.

## 7. Default Session Jump

- [x] 7.1 Add `--no-jump` to `session use` and `session choose`.
- [x] 7.2 Report bookmark index 0 as the default jump target after choosing or using a session when available.
- [x] 7.3 Update the shell wrapper to change directory to bookmark index 0 after successful `session use` or `session choose` unless `--no-jump` is present.
- [x] 7.4 Update README and tests for default jump and opt-out behavior.
- [x] 7.5 Run `cargo fmt --check`, `cargo test`, and `cargo build`.

## 8. Session Unset

- [x] 8.1 Add `session unset` parser support.
- [x] 8.2 Clear the appointed-session marker without deleting saved sessions.
- [x] 8.3 Update README and tests for unset behavior.
- [x] 8.4 Run `cargo fmt --check`, `cargo test`, and `cargo build`.
