## ADDED Requirements

### Requirement: Use appointed session for tmux layouts
The system SHALL create tmux layouts from the appointed session's bookmark list.

#### Scenario: Panes use appointed session
- **WHEN** session `api` is appointed and the user runs `bmersive tmux panes`
- **THEN** the system creates panes from `api` bookmarks only

#### Scenario: Windows use appointed session
- **WHEN** session `web` is appointed and the user runs `bmersive tmux windows`
- **THEN** the system creates windows from `web` bookmarks only

### Requirement: Name outside tmux session from appointed session
The system SHALL use a tmux session name derived from the appointed saved session when creating or attaching outside tmux.

#### Scenario: Outside tmux named session
- **WHEN** session `api` is appointed and the user runs `bmersive tmux panes` outside tmux
- **THEN** the system creates or attaches a tmux session named for `api` rather than a single global session name

## MODIFIED Requirements

### Requirement: Create tmux windows from bookmarks
The system SHALL create one tmux window per appointed-session bookmark in windows mode, using each bookmark path as the window working directory.

#### Scenario: Windows mode with bookmarks
- **WHEN** the appointed session bookmark list contains three directories and the user runs `bmersive tmux windows`
- **THEN** the system creates three tmux windows with those directories as working directories

### Requirement: Operate on current tmux session when inside tmux
The system SHALL operate on the current tmux session when `TMUX` indicates the command is running inside tmux, using bookmarks from the appointed saved session.

#### Scenario: Inside tmux windows mode
- **WHEN** session `api` is appointed and the user runs `bmersive tmux windows` from inside tmux
- **THEN** the system creates `api` bookmark windows in the current tmux session

### Requirement: Attach or create bmersive tmux session outside tmux
The system SHALL attach to an existing tmux session for the appointed saved session when outside tmux, and SHALL create a new session-specific tmux session from bookmarks when no such session exists.

#### Scenario: Existing session outside tmux
- **WHEN** session `api` is appointed, the user runs `bmersive tmux windows` outside tmux, and the tmux session for `api` already exists
- **THEN** the system attaches to the existing session without duplicating bookmark windows

#### Scenario: New session outside tmux
- **WHEN** session `api` is appointed, the user runs `bmersive tmux windows` outside tmux, and no tmux session for `api` exists
- **THEN** the system creates a session-specific tmux session from `api` bookmarks and attaches to it

### Requirement: Create tiled tmux panes from bookmarks
The system SHALL create tiled panes from appointed-session bookmarks when the user runs `bmersive tmux` or explicitly runs `bmersive tmux panes`.

#### Scenario: Panes mode with bookmarks
- **WHEN** the appointed session bookmark list contains four directories and the user runs `bmersive tmux panes`
- **THEN** the system creates tmux panes using those directories as pane working directories and applies a tiled layout

### Requirement: Handle empty bookmark list gracefully
The system SHALL handle tmux commands with an empty appointed-session bookmark list without creating empty layouts.

#### Scenario: Empty bookmarks for tmux
- **WHEN** the appointed session bookmark list is empty and the user runs `bmersive tmux`
- **THEN** the system reports that there are no bookmarks to materialize and exits without creating bookmark panes or windows
