## ADDED Requirements

### Requirement: Expose session command group
The system SHALL expose a `bmersive session` command group for saved session management.

#### Scenario: Session commands are parsed
- **WHEN** the user runs a supported `bmersive session` subcommand
- **THEN** the parser accepts the command and dispatches to session behavior

#### Scenario: Session help is available
- **WHEN** the user requests help for `bmersive session`
- **THEN** the system prints help output for supported session subcommands

### Requirement: Parse session listing command
The system SHALL expose a command to list saved sessions.

#### Scenario: List sessions command
- **WHEN** the user runs `bmersive session ls`
- **THEN** the parser accepts the command and the system lists saved sessions

### Requirement: Parse session appointment command
The system SHALL expose a command to appoint an existing saved session by name, with an opt-out flag for the default jump behavior.

#### Scenario: Use session command
- **WHEN** the user runs `bmersive session use api`
- **THEN** the parser accepts `api` as the session name and appoints that session when it exists

#### Scenario: Use session command without jump
- **WHEN** the user runs `bmersive session use api --no-jump`
- **THEN** the parser accepts the opt-out flag and appoints that session without default jump reporting

### Requirement: Parse session creation command
The system SHALL expose a command to create a named saved session.

#### Scenario: New session command
- **WHEN** the user runs `bmersive session new api`
- **THEN** the parser accepts `api` as the session name and creates the saved session when valid

### Requirement: Parse session chooser command
The system SHALL expose a command for interactive session selection by numeric index, with an opt-out flag for the default jump behavior.

#### Scenario: Choose session command
- **WHEN** the user runs `bmersive session choose`
- **THEN** the parser accepts the command and the system prompts for a saved session when sessions exist

#### Scenario: Choose session command without jump
- **WHEN** the user runs `bmersive session choose --no-jump`
- **THEN** the parser accepts the opt-out flag and the system prompts for a saved session without default jump reporting

### Requirement: Parse session unset command
The system SHALL expose a command to clear the appointed session for the current shell runtime state.

#### Scenario: Unset session command
- **WHEN** the user runs `bmersive session unset`
- **THEN** the parser accepts the command and clears the appointed session

## MODIFIED Requirements

### Requirement: Preserve command surface
The system SHALL preserve the existing public command names and accepted positional arguments for `init`, `add`, `ls`, `path`, `rm`, and `tmux`, and SHALL add the `session` command group for saved session management.

#### Scenario: Existing command remains available
- **WHEN** the user runs an existing documented command form with a session appointed
- **THEN** the system accepts the command form and executes the same user-facing bookmark, shell, or tmux behavior against the appointed session

#### Scenario: Session command is available
- **WHEN** the user runs a documented `session` command form
- **THEN** the system accepts the command form and executes the requested saved-session behavior

### Requirement: Generate standard help output
The system SHALL provide parser-generated help output for the top-level CLI, supported commands, and supported session subcommands.

#### Scenario: Top-level help
- **WHEN** the user runs `bmersive --help` or `bmersive help`
- **THEN** the system prints help output that lists the supported commands including `session`

#### Scenario: Command-specific help
- **WHEN** the user requests help for a supported command or session subcommand
- **THEN** the system prints help output for that command's accepted arguments

### Requirement: Reject invalid CLI input consistently
The system SHALL reject unknown commands and invalid argument forms before command side effects occur.

#### Scenario: Unknown command
- **WHEN** the user runs `bmersive unknown-command`
- **THEN** the system exits with an error and does not perform session, bookmark, shell, or tmux side effects

#### Scenario: Missing required argument
- **WHEN** the user runs a command without a required positional argument
- **THEN** the system exits with parser-generated usage guidance and does not perform command side effects

#### Scenario: Invalid session name form
- **WHEN** the user runs a session command with an invalid session name
- **THEN** the system exits with an error and does not create or appoint a session
