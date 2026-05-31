## ADDED Requirements

### Requirement: Shell wrapper chooses session when unappointed
The generated `b` function SHALL prompt the user to choose a saved session when called without arguments and no session is appointed for the current shell, then SHALL change directory to bookmark index 0 when available.

#### Scenario: No appointment and sessions exist
- **WHEN** the initialized user runs `b` without an appointed session and saved sessions exist
- **THEN** the wrapper invokes session selection before listing bookmarks

#### Scenario: Session selected from chooser
- **WHEN** the user selects a session from the chooser
- **THEN** subsequent `b` commands in the same shell use the selected session and the shell changes directory to bookmark index 0 when it exists

#### Scenario: Session selected without jump
- **WHEN** the user selects a session from the chooser with jump disabled
- **THEN** subsequent `b` commands in the same shell use the selected session and the shell does not change directory

### Requirement: Shell wrapper lists appointed session bookmarks
The generated `b` function SHALL list bookmarks for the appointed session when called without arguments after a session has been appointed.

#### Scenario: Appointment exists
- **WHEN** the initialized user runs `b` and session `api` is appointed
- **THEN** the wrapper invokes bookmark listing for `api`

### Requirement: Shell wrapper delegates session commands
The generated `b` function SHALL delegate `b session` and its arguments to `bmersive session`, and SHALL change directory to bookmark index 0 after successful `session use` or `session choose` unless `--no-jump` is present.

#### Scenario: Session command through wrapper
- **WHEN** the initialized user runs `b session ls`
- **THEN** the wrapper invokes `bmersive session ls`

#### Scenario: Session unset through wrapper
- **WHEN** the initialized user runs `b session unset`
- **THEN** the wrapper invokes `bmersive session unset` and does not change directory

#### Scenario: Session use through wrapper jumps
- **WHEN** the initialized user runs `b session use api` and bookmark index 0 exists
- **THEN** the wrapper appoints `api` and changes directory to bookmark index 0

#### Scenario: Session use through wrapper opts out
- **WHEN** the initialized user runs `b session use api --no-jump`
- **THEN** the wrapper appoints `api` without changing directory

## MODIFIED Requirements

### Requirement: Shell wrapper lists bookmarks
The generated `b` function SHALL list appointed-session bookmarks when called without arguments, choosing a session first when no session is appointed.

#### Scenario: No shell arguments with appointment
- **WHEN** the initialized user runs `b` and a session is appointed
- **THEN** the wrapper invokes `bmersive ls`

#### Scenario: No shell arguments without appointment
- **WHEN** the initialized user runs `b` and no session is appointed
- **THEN** the wrapper invokes session selection before invoking `bmersive ls`

### Requirement: Shell integration sets session state directory
The generated shell code SHALL set `BMERSIVE_STATE_DIR` when it is not already set so the appointed session is scoped to the initialized shell session.

#### Scenario: State directory is unset
- **WHEN** the user evaluates `bmersive init zsh` or `bmersive init bash` without `BMERSIVE_STATE_DIR` set
- **THEN** the generated shell code exports a shell-specific `BMERSIVE_STATE_DIR` for appointment state

### Requirement: Shell wrapper changes directory by index
The generated `b` function SHALL change the parent shell directory for numeric bookmark indexes by using `bmersive path <index>` against the appointed session and shell `cd`.

#### Scenario: Indexed jump
- **WHEN** session `api` is appointed and the initialized user runs `b 2`
- **THEN** the wrapper changes the current shell directory to the path printed by `bmersive path 2` from `api`

### Requirement: Shell wrapper delegates add
The generated `b` function SHALL delegate `b add` and its arguments to `bmersive add` for the appointed session.

#### Scenario: Add current directory through wrapper
- **WHEN** session `api` is appointed and the initialized user runs `b add`
- **THEN** the wrapper invokes `bmersive add` for `api`

### Requirement: Shell wrapper delegates remove
The generated `b` function SHALL delegate `b rm` and `b rm <index>` to `bmersive rm` for the appointed session.

#### Scenario: Remove through wrapper
- **WHEN** session `api` is appointed and the initialized user runs `b rm 2`
- **THEN** the wrapper invokes `bmersive rm 2` for `api`

### Requirement: Shell wrapper delegates tmux
The generated `b` function SHALL delegate `b tmux` and its arguments to `bmersive tmux` for the appointed session.

#### Scenario: Tmux through wrapper
- **WHEN** session `api` is appointed and the initialized user runs `b tmux panes`
- **THEN** the wrapper invokes `bmersive tmux panes` for `api`

### Requirement: Shell wrapper delegation remains compatible
The generated `b` function SHALL continue delegating to command forms accepted by the standardized CLI parser, including session command forms.

#### Scenario: Delegated command remains accepted
- **WHEN** the generated shell wrapper invokes `bmersive add`, `bmersive rm`, `bmersive session`, or `bmersive tmux` with user-provided arguments
- **THEN** the standardized CLI parser accepts the delegated command forms that are supported
