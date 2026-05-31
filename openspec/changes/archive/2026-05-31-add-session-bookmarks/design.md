## Context

The current implementation stores a single ordered bookmark list in `bookmarks.json`, with the shell integration creating a per-shell `BMERSIVE_STATE_DIR`. That makes bookmarks temporary and shell-local. The requested behavior needs named saved sessions so different project contexts can keep separate bookmark lists, while the running shell only needs to remember which saved session is currently appointed.

The codebase is intentionally small: CLI parsing, state management, shell integration, bookmark commands, tmux commands, and tests live in `src/main.rs`. The design should preserve that simplicity unless implementation pressure proves otherwise.

## Goals / Non-Goals

**Goals:**

- Store multiple named sessions in one durable state file, with each session owning an ordered bookmark list.
- Track the appointed session for the current shell separately from the saved session data.
- Make `b` with no appointed session guide the user through choosing a session before listing bookmarks.
- Jump to bookmark index 0 by default after choosing or using a session from the shell wrapper, with a `--no-jump` opt-out.
- Keep existing bookmark commands familiar once a session is appointed.
- Materialize tmux layouts from the appointed session's bookmarks.
- Document the workflow in `README.md`, including session chooser, bookmark chooser, and four-folder tmux pane diagram.

**Non-Goals:**

- Synchronizing sessions across machines or users.
- Adding fuzzy finder dependencies or terminal UI frameworks.
- Supporting nested sessions, tags, or bookmark metadata beyond paths.
- Replacing existing indexed bookmark selection semantics.

## Decisions

### Store saved sessions in a durable user state file

Use a user-level state path such as `$XDG_STATE_HOME/bmersive/sessions.json`, falling back to `$HOME/.local/state/bmersive/sessions.json`, then `/tmp/bmersive-$USER/sessions.json` when no home-like location is available. This separates durable saved sessions from the current shell's appointed-session marker.

Alternative considered: continue using `$BMERSIVE_STATE_DIR/bookmarks.json` and create one directory per session. That keeps implementation close to the current model, but still ties saved sessions to ephemeral shell state and makes cross-shell project contexts harder to reuse.

### Keep shell appointment ephemeral

The generated shell integration should continue exporting `BMERSIVE_STATE_DIR` for shell-local runtime data, but that directory should contain the appointed session marker rather than the saved bookmark database. Appointment is per running shell; saved sessions are durable.

Alternative considered: export the appointed session directly as `BMERSIVE_SESSION`. That is simple for shell code but makes appointment harder to update from the CLI without emitting shell assignments. A marker file in `BMERSIVE_STATE_DIR` lets `bmersive session use <name>` persist appointment for the shell wrapper and direct commands.

### Add minimal session CLI commands

Add a `session` command group with enough surface area for shell integration and direct use: list sessions, create/select a session, print the current appointed session, and prompt for a session when needed. The exact parser shape should remain small and help-driven, for example `bmersive session ls`, `bmersive session use <name>`, and `bmersive session choose`.

Alternative considered: overload `ls` and `add` with session flags. A command group is clearer and keeps bookmark commands focused on the appointed session.

### Preserve legacy bookmark state where practical

If the old single-session `bookmarks.json` exists and the new sessions file does not, load those bookmarks into a default saved session such as `default`. This avoids losing existing user data while allowing the new model to take over on first save.

Alternative considered: require users to recreate bookmarks. That is simpler but violates expectations for an existing tool and would make the change feel destructive.

### Keep chooser behavior simple and dependency-free

Implement choosers as numbered stdout prompts with stdin input, matching the existing remove prompt style. The shell `b` wrapper can call the chooser command when no session is appointed, then continue to list bookmarks for the selected session.

Alternative considered: integrate with `fzf` or another selector. That would be nicer interactively, but it adds external dependency and install assumptions that are not necessary for the first version.

### Jump from shell wrapper, not the CLI process

When `bmersive session use <name>` or `bmersive session choose` succeeds, the CLI should print whether bookmark index 0 is the default jump target. The generated `b` function should perform the actual `cd` after successful session appointment unless `--no-jump` is present. Direct `bmersive` invocations cannot change the parent shell directory, so they only report the intended jump.

Alternative considered: have the CLI print only the path for command substitution. That would make normal direct output less clear and require more shell parsing. Keeping human-readable output plus a separate shell `bmersive path 0` call is simpler.

### Name tmux sessions from appointed sessions

When outside tmux, use a tmux session name derived from the appointed bmersive session, such as `bmersive-<session>`, instead of one global `bmersive` session. This prevents project sessions from colliding when multiple saved contexts exist.

Alternative considered: keep the current `bmersive` tmux session name. That preserves current behavior but conflicts with the purpose of having multiple project sessions.

## Risks / Trade-offs

- Existing state migration could surprise users if default-session naming is unclear -> Print clear messages and document the default migration path.
- Session names used in file data and tmux names can contain unsafe characters -> Validate names to a conservative kebab-case-like character set and sanitize for tmux where needed.
- Prompting from shell functions can be awkward in scripts -> Keep direct `bmersive` commands non-interactive unless the command explicitly chooses or removes without an argument.
- Durable state may live in different places depending on environment -> Centralize path resolution and cover fallback behavior with unit tests.
- One-file state can lose concurrent writes from multiple shells -> Accept this for the small local tool, but keep writes atomic enough for normal usage by writing complete JSON in one save operation.

## Migration Plan

- Introduce the new sessions state structs and path resolution alongside the existing bookmark path loader.
- On load, prefer the new sessions file. If absent, import existing `bookmarks.json` into a default session in memory and save to the new sessions file on first mutation.
- Update bookmark commands to resolve the appointed session before reading or mutating bookmarks.
- Update shell integration to choose a session when appointment is missing.
- Update tmux session naming and bookmark lookup to use appointed session data.
- Update README examples and tests.

Rollback is straightforward before migration writes occur. After migration, the old `bookmarks.json` remains available as a backup unless explicitly removed by a future cleanup.

## Open Questions

- Should session names be strictly kebab-case, or allow broader project names with spaces? The recommended first implementation is kebab-case-like names for predictable CLI and tmux behavior.
