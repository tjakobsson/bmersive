## Context

`bmersive` is a new CLI and shell companion for maintaining a small, explicit set of directories that define the user's current working session. The repository currently has OpenSpec scaffolding but no Rust application structure, so this change establishes the initial architecture, command surface, state format, and shell/tmux integration behavior.

The product boundary is intentionally narrow: this is not a directory history, ranking, or fuzzy-jump tool. The user manually curates a small working set, jumps by index, and can materialize the set into tmux.

License and dependency constraints are important. The implementation must avoid copyleft-licensed dependencies and keep the dependency set minimal.

## Goals / Non-Goals

**Goals:**

- Provide a Rust CLI named `bmersive` with commands for shell initialization, bookmark management, path lookup, and tmux materialization.
- Provide zsh and bash wrapper output so `b <index>` can change the parent shell directory.
- Store bookmarks in session-local JSON state with a deterministic fallback path.
- Preserve manual insertion order and reject duplicates.
- Enforce a configurable maximum bookmark count using `BMERSIVE_MAX_BOOKMARKS`, defaulting to 10.
- Keep implementation dependency-light and permissive-license-only.

**Non-Goals:**

- No fuzzy search, frecency, auto-ranking, directory history, or recency behavior.
- No persistent workspace system in the initial version.
- No zellij support in the initial version.
- No config file in the initial version.
- No automatic tmux session rebuild or destructive pane/window cleanup.

## Decisions

### Use lexical absolute paths for bookmark identity

Bookmarks will be converted to absolute paths and lexically normalized for `.` and `..`, but symlink spelling will be preserved. Duplicate detection will compare this normalized absolute string.

This preserves the path the user thinks in, which matters for a workspace tool. Fully canonicalizing filesystem paths would collapse symlinked workspace names into their targets and can make display and jumping feel surprising.

Alternatives considered:

- Full filesystem canonicalization: stronger inode-level dedupe, but surprising for symlinked workspaces.
- Raw path storage: preserves exact input but allows accidental duplicates such as `.` and `../project`.

### Store session state in JSON under runtime storage

The shell integration will set `BMERSIVE_STATE_DIR` to a session-specific directory when it is not already set. The CLI will store bookmarks at `$BMERSIVE_STATE_DIR/bookmarks.json` when that variable is set, then `$XDG_RUNTIME_DIR/bmersive/bookmarks.json` when `XDG_RUNTIME_DIR` is set, otherwise `/tmp/bmersive-$USER/bookmarks.json`. The state file will contain an ordered list of bookmark paths.

This makes the shell UX session-oriented without introducing a long-lived persistent workspace model. JSON is easy to inspect and can be handled with `serde` and `serde_json`.

Alternatives considered:

- Persistent config/data directory: conflicts with the initial session-oriented product boundary.
- Plain text file: lower dependency footprint, but less extensible for future metadata.

### Configure max size with an environment variable

The default maximum bookmark count will be 10. Users can override it with `BMERSIVE_MAX_BOOKMARKS`. Invalid or non-positive values will fall back to the default.

This avoids adding config file discovery and parsing for a single setting. When the list is full, add operations fail clearly and tell the user to remove an entry.

Alternatives considered:

- Config file: premature for one setting.
- Auto-eviction: violates the curated working-set model.

### Generate thin shell wrappers

`bmersive init zsh` and `bmersive init bash` will print shell code defining function `b`. The wrapper delegates all behavior to the Rust CLI except changing directories. For `b <index>`, the wrapper runs `cd "$(bmersive path <index>)"`.

This is required because a child process cannot change the parent shell directory.

Alternatives considered:

- Shell aliases: insufficient for argument dispatch and command substitution.
- Asking users to call `cd $(bmersive path N)` manually: too clumsy for core UX.

### Make tmux panes the default materialization mode

`bmersive tmux` defaults to `panes`. Inside tmux, it creates a new bookmark-backed window and tiles panes for the remaining bookmarks. Outside tmux, it attaches to an existing `bmersive` session if one exists; otherwise it creates a `bmersive` session from the bookmark list, tiles panes, and attaches to it.

This matches the intended immersive workspace feel: `b tmux` should materialize the active working set into a single tiled workspace. One-window-per-bookmark mode remains available explicitly with `bmersive tmux windows`.

Alternatives considered:

- Defaulting to windows: less destructive, but weaker fit for the desired workspace materialization flow.
- Rebuilding an existing `bmersive` session: surprising and potentially destructive; defer to a future explicit command if needed.

### Keep dependencies minimal and permissive

The initial implementation should prefer the Rust standard library where practical. `serde` and `serde_json` are acceptable for state serialization. CLI parsing can be implemented manually to avoid an additional dependency unless command complexity warrants adding a permissive parser dependency.

Dependencies must be reviewed for license compatibility and must not introduce copyleft licensing.

## Risks / Trade-offs

- Symlink-preserving duplicate detection can allow two different symlink spellings for the same target → This is acceptable because the tool tracks user-facing workspace paths, not filesystem identity.
- Runtime state paths can survive longer or shorter than the user expects depending on OS behavior → Document the session-local intent and fallback path behavior.
- Manual argument parsing can become brittle as commands grow → Keep the command surface small initially; revisit a parser dependency only if complexity increases.
- Tmux integration depends on the external `tmux` binary and environment variables → Detect command failures and print clear errors.
- Outside-tmux attach behavior will not refresh an existing `bmersive` session from updated bookmarks → Prefer non-destructive behavior now; add explicit rebuild behavior later if needed.

## Migration Plan

This is a new application with no existing user data to migrate. Implementation can be added incrementally: establish the Rust project, add bookmark state and commands, add shell initialization, then add tmux materialization.

Rollback is removal of the new application files and OpenSpec change artifacts before release. Once released, the state file is session-local and can be safely removed by users if needed.

## Open Questions

- Should future persistent workspaces reuse the same state model or use a separate data location?
- Should future tmux support include explicit `--session` and `--rebuild` options?
