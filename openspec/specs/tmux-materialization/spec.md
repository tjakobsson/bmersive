## Purpose

Create or reuse tmux layouts from the current curated bookmark list.

## Requirements

### Requirement: Default tmux mode creates panes
The system SHALL treat `bmersive tmux` as `bmersive tmux panes`.

#### Scenario: Default tmux mode
- **WHEN** the user runs `bmersive tmux`
- **THEN** the system uses panes mode

### Requirement: Create tmux windows from bookmarks
The system SHALL create one tmux window per bookmark in windows mode, using each bookmark path as the window working directory.

#### Scenario: Windows mode with bookmarks
- **WHEN** the bookmark list contains three directories and the user runs `bmersive tmux windows`
- **THEN** the system creates three tmux windows with those directories as working directories

### Requirement: Name tmux windows from basenames
The system SHALL name tmux windows from the bookmark directory basename where possible.

#### Scenario: Window basename
- **WHEN** a bookmark path is `/work/api`
- **THEN** the created tmux window name is based on `api`

### Requirement: Operate on current tmux session when inside tmux
The system SHALL operate on the current tmux session when `TMUX` indicates the command is running inside tmux.

#### Scenario: Inside tmux windows mode
- **WHEN** the user runs `bmersive tmux windows` from inside tmux
- **THEN** the system creates bookmark windows in the current tmux session

### Requirement: Attach or create bmersive tmux session outside tmux
The system SHALL attach to an existing `bmersive` tmux session when outside tmux, and SHALL create a new `bmersive` session from bookmarks when no such session exists.

#### Scenario: Existing session outside tmux
- **WHEN** the user runs `bmersive tmux windows` outside tmux and a `bmersive` tmux session already exists
- **THEN** the system attaches to the existing session without duplicating bookmark windows

#### Scenario: New session outside tmux
- **WHEN** the user runs `bmersive tmux windows` outside tmux and no `bmersive` tmux session exists
- **THEN** the system creates a `bmersive` session from bookmarks and attaches to it

### Requirement: Create tiled tmux panes from bookmarks
The system SHALL create tiled panes from bookmarks when the user runs `bmersive tmux` or explicitly runs `bmersive tmux panes`.

#### Scenario: Panes mode with bookmarks
- **WHEN** the bookmark list contains three directories and the user runs `bmersive tmux panes`
- **THEN** the system creates tmux panes using those directories as pane working directories and applies a tiled layout

### Requirement: Handle empty bookmark list gracefully
The system SHALL handle tmux commands with an empty bookmark list without creating empty layouts.

#### Scenario: Empty bookmarks for tmux
- **WHEN** the bookmark list is empty and the user runs `bmersive tmux`
- **THEN** the system reports that there are no bookmarks to materialize and exits without creating bookmark panes or windows
