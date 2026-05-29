## Purpose

Manage a curated session-local directory bookmark list with shell-friendly indexed lookup.

## Requirements

### Requirement: Session-local bookmark storage
The system SHALL store bookmarks in a session-oriented JSON state file at `$BMERSIVE_STATE_DIR/bookmarks.json` when `BMERSIVE_STATE_DIR` is set, otherwise at `$XDG_RUNTIME_DIR/bmersive/bookmarks.json` when `XDG_RUNTIME_DIR` is set, and SHALL fall back to `/tmp/bmersive-$USER/bookmarks.json` otherwise.

#### Scenario: Shell session state directory is available
- **WHEN** `BMERSIVE_STATE_DIR` is set and the user runs a bookmark command
- **THEN** the system uses `$BMERSIVE_STATE_DIR/bookmarks.json` as the state file

#### Scenario: Runtime directory is available
- **WHEN** `XDG_RUNTIME_DIR` is set and the user runs a bookmark command
- **THEN** the system uses `$XDG_RUNTIME_DIR/bmersive/bookmarks.json` as the state file

#### Scenario: Runtime directory is unavailable
- **WHEN** `XDG_RUNTIME_DIR` is not set and the user runs a bookmark command
- **THEN** the system uses `/tmp/bmersive-$USER/bookmarks.json` as the state file

### Requirement: Add bookmark
The system SHALL allow users to add the current directory or an explicit path to the bookmark list using `bmersive add [path]`.

#### Scenario: Add current directory
- **WHEN** the user runs `bmersive add` without a path
- **THEN** the system adds the current working directory to the bookmark list

#### Scenario: Add explicit path
- **WHEN** the user runs `bmersive add /some/project`
- **THEN** the system adds `/some/project` to the bookmark list

### Requirement: Preserve insertion order
The system SHALL preserve manual insertion order when storing and listing bookmarks.

#### Scenario: List reflects manual order
- **WHEN** the user adds directories A, B, and C in that order
- **THEN** `bmersive ls` lists A at index 0, B at index 1, and C at index 2

### Requirement: Reject duplicate bookmarks
The system SHALL reject duplicate bookmarks using normalized absolute path strings that resolve relative path segments but preserve symlink spelling.

#### Scenario: Duplicate normalized path
- **WHEN** the bookmark list already contains `/work/project` and the user adds `/work/./project`
- **THEN** the system does not add a second bookmark for that path

### Requirement: Enforce maximum bookmark count
The system SHALL limit the bookmark list to `BMERSIVE_MAX_BOOKMARKS` when set to a positive integer, otherwise to 10 bookmarks by default.

#### Scenario: List is full
- **WHEN** the bookmark list has reached the configured maximum and the user adds another bookmark
- **THEN** the system rejects the add operation and reports that the list is full

#### Scenario: Max is configured
- **WHEN** `BMERSIVE_MAX_BOOKMARKS` is set to `5`
- **THEN** the system allows at most 5 bookmarks

### Requirement: List indexed bookmarks
The system SHALL list bookmarks with stable numeric indexes suitable for shell selection.

#### Scenario: List bookmarks
- **WHEN** the user runs `bmersive ls`
- **THEN** the system prints each bookmark with its current numeric index

#### Scenario: Empty list
- **WHEN** the user runs `bmersive ls` and no bookmarks exist
- **THEN** the system reports the empty state without error

### Requirement: Resolve bookmark path by index
The system SHALL print only the bookmark path for a valid index using `bmersive path <index>`.

#### Scenario: Valid path lookup
- **WHEN** bookmark index 2 exists and the user runs `bmersive path 2`
- **THEN** the system prints only the path for bookmark 2 to stdout

#### Scenario: Invalid path lookup
- **WHEN** the user runs `bmersive path 9` and index 9 does not exist
- **THEN** the system exits with an error and does not print a path to stdout

### Requirement: Remove bookmark
The system SHALL remove bookmarks by index using `bmersive rm [index]`, prompting for an index when none is provided.

#### Scenario: Direct remove
- **WHEN** the user runs `bmersive rm 2` and index 2 exists
- **THEN** the system removes bookmark 2 and preserves the relative order of remaining bookmarks

#### Scenario: Prompted remove
- **WHEN** the user runs `bmersive rm` and enters a valid listed index at the prompt
- **THEN** the system removes the selected bookmark
