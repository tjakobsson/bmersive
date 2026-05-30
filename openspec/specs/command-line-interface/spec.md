## Purpose

Define the standard command-line interface contract, parser behavior, and dependency policy for `bmersive` commands.

## Requirements

### Requirement: Use standardized CLI parser
The system SHALL parse top-level commands and command arguments using a standardized Rust CLI parser dependency that is permissively licensed, current, and community healthy.

#### Scenario: Supported command is parsed
- **WHEN** the user runs a supported `bmersive` command
- **THEN** the system dispatches the command through the standardized parser before executing command behavior

#### Scenario: Dependency policy is satisfied
- **WHEN** the CLI parser dependency is added
- **THEN** it uses a non-copyleft license and a current stable version from a healthy Rust CLI project

### Requirement: Preserve command surface
The system SHALL preserve the existing public command names and accepted positional arguments for `init`, `add`, `ls`, `path`, `rm`, and `tmux`.

#### Scenario: Existing command remains available
- **WHEN** the user runs an existing documented command form
- **THEN** the system accepts the command form and executes the same user-facing behavior as before parser standardization

### Requirement: Generate standard help output
The system SHALL provide parser-generated help output for the top-level CLI and supported commands.

#### Scenario: Top-level help
- **WHEN** the user runs `bmersive --help` or `bmersive help`
- **THEN** the system prints help output that lists the supported commands

#### Scenario: Command-specific help
- **WHEN** the user requests help for a supported command
- **THEN** the system prints help output for that command's accepted arguments

### Requirement: Generate version output
The system SHALL provide parser-generated version output using the package version.

#### Scenario: Version flag
- **WHEN** the user runs `bmersive --version` or `bmersive -V`
- **THEN** the system prints the `bmersive` package version

### Requirement: Reject invalid CLI input consistently
The system SHALL reject unknown commands and invalid argument forms before command side effects occur.

#### Scenario: Unknown command
- **WHEN** the user runs `bmersive unknown-command`
- **THEN** the system exits with an error and does not perform bookmark, shell, or tmux side effects

#### Scenario: Missing required argument
- **WHEN** the user runs a command without a required positional argument
- **THEN** the system exits with parser-generated usage guidance and does not perform command side effects

### Requirement: Defer application configuration framework
The system SHALL NOT introduce a Viper-like application configuration dependency as part of CLI parser standardization.

#### Scenario: Configuration remains environment based
- **WHEN** the CLI parser is standardized
- **THEN** existing environment-based configuration and bookmark state handling remain implemented without a new configuration framework
