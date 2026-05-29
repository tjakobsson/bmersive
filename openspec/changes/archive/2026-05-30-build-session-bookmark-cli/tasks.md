## 1. Project Setup

- [x] 1.1 Create the initial Rust binary project structure for `bmersive`.
- [x] 1.2 Add only required permissive-license dependencies for JSON state serialization.
- [x] 1.3 Add basic build and test scripts or documentation for running checks.

## 2. Bookmark State

- [x] 2.1 Implement runtime state path resolution using `$BMERSIVE_STATE_DIR/bookmarks.json`, `$XDG_RUNTIME_DIR/bmersive/bookmarks.json`, and `/tmp/bmersive-$USER/bookmarks.json` fallback.
- [x] 2.2 Implement JSON state load/save with automatic parent directory creation.
- [x] 2.3 Implement lexical absolute path normalization that resolves `.` and `..` without resolving symlinks.
- [x] 2.4 Implement maximum bookmark count resolution from `BMERSIVE_MAX_BOOKMARKS`, defaulting to 10 for invalid or unset values.

## 3. Bookmark Commands

- [x] 3.1 Implement command dispatch for `init`, `add`, `ls`, `path`, `rm`, and `tmux` without unnecessary parser dependencies.
- [x] 3.2 Implement `bmersive add [path]` with insertion-order preservation, duplicate rejection, and full-list rejection.
- [x] 3.3 Implement `bmersive ls` with indexed output and graceful empty-state output.
- [x] 3.4 Implement `bmersive path <index>` with path-only stdout for valid indexes and stderr errors for invalid indexes.
- [x] 3.5 Implement `bmersive rm [index]` with direct removal and prompted removal mode.

## 4. Shell Integration

- [x] 4.1 Implement `bmersive init zsh` to emit a zsh-compatible `b` function.
- [x] 4.2 Implement `bmersive init bash` to emit a bash-compatible `b` function.
- [x] 4.3 Ensure generated wrappers call `bmersive ls` for `b`, `cd "$(bmersive path N)"` for numeric indexes, and delegate `add`, `rm`, and `tmux` subcommands.

## 5. Tmux Materialization

- [x] 5.1 Implement `bmersive tmux` as the default panes mode and parse explicit `windows` and `panes` modes.
- [x] 5.2 Implement inside-tmux windows mode by creating one window per bookmark in the current session.
- [x] 5.3 Implement outside-tmux behavior that attaches to an existing `bmersive` session or creates and attaches a new session from bookmarks.
- [x] 5.4 Implement window naming from bookmark basenames where possible.
- [x] 5.5 Implement explicit panes mode with bookmark working directories and tiled layout.
- [x] 5.6 Handle empty bookmark lists gracefully for tmux commands.

## 6. Verification

- [x] 6.1 Add unit tests for state path resolution, max count parsing, path normalization, duplicate rejection, and index validation.
- [x] 6.2 Add command-level tests or integration coverage for `add`, `ls`, `path`, and `rm` using isolated state paths or environment variables.
- [x] 6.3 Add tests or documented manual verification for generated zsh and bash wrappers.
- [x] 6.4 Add tests or documented manual verification for tmux command construction and inside/outside tmux branching.
- [x] 6.5 Run formatting, tests, and build verification.
