## Why

`bmersive` currently scopes bookmarks to the active shell state directory, which makes each shell feel isolated rather than letting users intentionally return to named project contexts. Users working across multiple projects need durable, selectable sessions so `b` can help them get into the right zone before choosing a bookmark.

## What Changes

- Add support for multiple saved bookmark sessions, where each session owns its own ordered bookmark list.
- Allow the shell `b` command with no appointed session to present a session chooser before showing or using bookmarks.
- Keep bookmark selection scoped to the appointed session once a session is selected for the running shell.
- Jump to bookmark index 0 by default after choosing or using a session, with an opt-out flag for users who only want to appoint the session.
- Allow users to unset the appointed session so the shell returns to the no-session-selected state.
- Update tmux materialization to use the appointed session's bookmarks and document a four-folder split-pane demo.
- Update `README.md` with a user-facing demo covering session selection, bookmark selection, and tmux integration with an ASCII split-window diagram.

## Capabilities

### New Capabilities

### Modified Capabilities

- `session-bookmarks`: Bookmarks become scoped to named saved sessions instead of only the current shell state directory.
- `shell-integration`: The `b` wrapper chooses a session when none is appointed and then lists or selects bookmarks for that session.
- `tmux-materialization`: Tmux layouts use the appointed session's bookmark list and are documented as project immersion workspaces.
- `command-line-interface`: CLI commands expose the minimal session management surface needed by shell integration and direct users.

## Impact

- Affects bookmark state schema and state loading/saving logic.
- Affects shell init output and the generated `b` wrapper behavior.
- Affects CLI command definitions, help output, and parser validation.
- Affects tmux commands by requiring them to resolve bookmarks through the appointed session.
- Requires migration or compatibility handling for any existing single-session bookmark state.
- Updates `README.md` usage and demo documentation.
