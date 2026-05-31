## ADDED Requirements

### Requirement: Saved session bookmark storage
The system SHALL store multiple named saved sessions in durable state, with each saved session containing its own ordered bookmark list.

#### Scenario: Multiple saved sessions exist
- **WHEN** the user has saved sessions named `api` and `web`
- **THEN** the system stores separate bookmark lists for `api` and `web`

#### Scenario: Saved sessions outlive shell runtime state
- **WHEN** a shell session exits after saving bookmarks in a named session
- **THEN** a later shell can select the same named session and access its bookmarks

### Requirement: Appointed session controls bookmark commands
The system SHALL apply bookmark add, list, path lookup, and remove commands to the currently appointed saved session.

#### Scenario: Add bookmark to appointed session
- **WHEN** the appointed session is `api` and the user runs `bmersive add /work/api`
- **THEN** `/work/api` is added to the `api` session bookmark list

#### Scenario: List appointed session bookmarks
- **WHEN** the appointed session is `api` and `api` has bookmarks
- **THEN** `bmersive ls` lists bookmarks from `api` only

#### Scenario: Lookup appointed session bookmark
- **WHEN** the appointed session is `api` and bookmark index 1 exists in `api`
- **THEN** `bmersive path 1` prints only that bookmark path from `api`

#### Scenario: Remove appointed session bookmark
- **WHEN** the appointed session is `api` and the user runs `bmersive rm 1`
- **THEN** bookmark index 1 is removed from `api` only

### Requirement: Require appointed session for bookmark access
The system SHALL require a saved session to be appointed before bookmark commands can access session-scoped bookmarks.

#### Scenario: No appointed session for list
- **WHEN** no session is appointed and the user runs `bmersive ls` directly
- **THEN** the system reports that no session is selected and does not list bookmarks

#### Scenario: No appointed session for add
- **WHEN** no session is appointed and the user runs `bmersive add /work/api` directly
- **THEN** the system reports that no session is selected and does not create a bookmark

### Requirement: Create saved session
The system SHALL allow users to create a named saved session.

#### Scenario: Create new session
- **WHEN** the user creates a session named `api`
- **THEN** the system stores a saved session named `api` with an empty bookmark list

#### Scenario: Reject duplicate session
- **WHEN** a saved session named `api` already exists and the user creates `api` again
- **THEN** the system rejects the duplicate session without changing bookmarks

### Requirement: List saved sessions
The system SHALL list saved sessions with stable numeric indexes suitable for prompt selection.

#### Scenario: List sessions
- **WHEN** saved sessions `api` and `web` exist
- **THEN** the system lists both sessions with numeric indexes

#### Scenario: No saved sessions
- **WHEN** no saved sessions exist
- **THEN** the system reports the empty session state without error

### Requirement: Appoint saved session
The system SHALL allow users to appoint a saved session for the current shell runtime state and SHALL report bookmark index 0 as the default jump target when that bookmark exists.

#### Scenario: Appoint existing session by name
- **WHEN** saved session `api` exists and the user appoints `api`
- **THEN** bookmark commands in the current shell use `api`

#### Scenario: Appointed session has first bookmark
- **WHEN** saved session `api` exists with bookmark index 0 and the user appoints `api`
- **THEN** the system reports that bookmark index 0 is the jump target

#### Scenario: Appointed session has no first bookmark
- **WHEN** saved session `api` exists without bookmarks and the user appoints `api`
- **THEN** the system reports that no bookmark is available to jump to

#### Scenario: Appointment opts out of jump
- **WHEN** saved session `api` exists and the user appoints `api` with jump disabled
- **THEN** the system appoints `api` without reporting a jump target

#### Scenario: Reject missing session appointment
- **WHEN** saved session `missing` does not exist and the user appoints `missing`
- **THEN** the system exits with an error and does not change the current appointment

### Requirement: Unset appointed session
The system SHALL allow users to clear the appointed saved session for the current shell runtime state.

#### Scenario: Unset existing appointment
- **WHEN** session `api` is appointed and the user unsets the session
- **THEN** no session is appointed for the current shell

#### Scenario: Unset without appointment
- **WHEN** no session is appointed and the user unsets the session
- **THEN** the system leaves the shell with no appointed session and exits without error

### Requirement: Migrate single-session bookmarks
The system SHALL preserve existing single-session bookmark state by importing it into a default saved session when the new saved-session state does not exist.

#### Scenario: Legacy bookmarks are imported
- **WHEN** the old `bookmarks.json` exists and the new sessions state does not exist
- **THEN** the system exposes the old bookmarks in a saved session named `default`

#### Scenario: New state takes precedence
- **WHEN** both the new sessions state and the old `bookmarks.json` exist
- **THEN** the system uses the new sessions state as the source of truth

## MODIFIED Requirements

### Requirement: Session-local bookmark storage
The system SHALL store the current shell's appointed session in shell-local runtime state and SHALL store saved session bookmark lists in durable bmersive state.

#### Scenario: Shell session state directory is available
- **WHEN** `BMERSIVE_STATE_DIR` is set and the user appoints a session
- **THEN** the system stores the appointment under `BMERSIVE_STATE_DIR`

#### Scenario: Runtime directory is available
- **WHEN** `XDG_RUNTIME_DIR` is set and durable state is needed without a more specific user state directory
- **THEN** the system uses a bmersive state location under the available user runtime or state directory

#### Scenario: Runtime directory is unavailable
- **WHEN** no user state or runtime directory is available and durable state is needed
- **THEN** the system uses a `/tmp/bmersive-$USER` fallback for saved session state

### Requirement: Add bookmark
The system SHALL allow users to add the current directory or an explicit path to the appointed session's bookmark list using `bmersive add [path]`.

#### Scenario: Add current directory
- **WHEN** the appointed session is `api` and the user runs `bmersive add` without a path
- **THEN** the system adds the current working directory to the `api` bookmark list

#### Scenario: Add explicit path
- **WHEN** the appointed session is `api` and the user runs `bmersive add /some/project`
- **THEN** the system adds `/some/project` to the `api` bookmark list

### Requirement: Preserve insertion order
The system SHALL preserve manual insertion order when storing and listing bookmarks within each saved session.

#### Scenario: List reflects manual order
- **WHEN** the user adds directories A, B, and C in that order to session `api`
- **THEN** `bmersive ls` lists A at index 0, B at index 1, and C at index 2 while `api` is appointed

### Requirement: Reject duplicate bookmarks
The system SHALL reject duplicate bookmarks within the appointed session using normalized absolute path strings that resolve relative path segments but preserve symlink spelling.

#### Scenario: Duplicate normalized path
- **WHEN** the appointed session's bookmark list already contains `/work/project` and the user adds `/work/./project`
- **THEN** the system does not add a second bookmark for that path in the appointed session

### Requirement: Enforce maximum bookmark count
The system SHALL limit each saved session's bookmark list to `BMERSIVE_MAX_BOOKMARKS` when set to a positive integer, otherwise to 10 bookmarks by default.

#### Scenario: List is full
- **WHEN** the appointed session's bookmark list has reached the configured maximum and the user adds another bookmark
- **THEN** the system rejects the add operation and reports that the list is full

#### Scenario: Max is configured
- **WHEN** `BMERSIVE_MAX_BOOKMARKS` is set to `5`
- **THEN** the system allows at most 5 bookmarks per saved session

### Requirement: List indexed bookmarks
The system SHALL list the appointed session's bookmarks with stable numeric indexes suitable for shell selection.

#### Scenario: List bookmarks
- **WHEN** the appointed session has bookmarks and the user runs `bmersive ls`
- **THEN** the system prints each appointed-session bookmark with its current numeric index

#### Scenario: Empty list
- **WHEN** the appointed session has no bookmarks and the user runs `bmersive ls`
- **THEN** the system reports the empty state without error

### Requirement: Resolve bookmark path by index
The system SHALL print only the appointed session's bookmark path for a valid index using `bmersive path <index>`.

#### Scenario: Valid path lookup
- **WHEN** bookmark index 2 exists in the appointed session and the user runs `bmersive path 2`
- **THEN** the system prints only the path for bookmark 2 to stdout

#### Scenario: Invalid path lookup
- **WHEN** the user runs `bmersive path 9` and index 9 does not exist in the appointed session
- **THEN** the system exits with an error and does not print a path to stdout

### Requirement: Remove bookmark
The system SHALL remove bookmarks from the appointed session by index using `bmersive rm [index]`, prompting for an index when none is provided.

#### Scenario: Direct remove
- **WHEN** the appointed session is `api` and the user runs `bmersive rm 2` and index 2 exists
- **THEN** the system removes bookmark 2 from `api` and preserves the relative order of remaining bookmarks

#### Scenario: Prompted remove
- **WHEN** the appointed session is `api` and the user runs `bmersive rm` and enters a valid listed index at the prompt
- **THEN** the system removes the selected bookmark from `api`
