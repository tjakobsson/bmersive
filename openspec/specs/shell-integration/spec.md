## Purpose

Emit shell integration for zsh and bash so `b <index>` can change the parent shell directory while other operations delegate to the `bmersive` CLI.

## Requirements

### Requirement: Initialize zsh integration
The system SHALL emit zsh-compatible shell code from `bmersive init zsh` that defines a shell function named `b`.

#### Scenario: zsh init output
- **WHEN** the user runs `bmersive init zsh`
- **THEN** the system prints shell code that can be evaluated by zsh to define function `b`

### Requirement: Initialize bash integration
The system SHALL emit bash-compatible shell code from `bmersive init bash` that defines a shell function named `b`.

#### Scenario: bash init output
- **WHEN** the user runs `bmersive init bash`
- **THEN** the system prints shell code that can be evaluated by bash to define function `b`

### Requirement: Shell wrapper lists bookmarks
The generated `b` function SHALL list bookmarks when called without arguments.

#### Scenario: No shell arguments
- **WHEN** the initialized user runs `b`
- **THEN** the wrapper invokes `bmersive ls`

### Requirement: Shell integration sets session state directory
The generated shell code SHALL set `BMERSIVE_STATE_DIR` when it is not already set so bookmark state is scoped to the initialized shell session.

#### Scenario: State directory is unset
- **WHEN** the user evaluates `bmersive init zsh` or `bmersive init bash` without `BMERSIVE_STATE_DIR` set
- **THEN** the generated shell code exports a session-specific `BMERSIVE_STATE_DIR`

### Requirement: Shell wrapper changes directory by index
The generated `b` function SHALL change the parent shell directory for numeric bookmark indexes by using `bmersive path <index>` and shell `cd`.

#### Scenario: Indexed jump
- **WHEN** the initialized user runs `b 2`
- **THEN** the wrapper changes the current shell directory to the path printed by `bmersive path 2`

### Requirement: Shell wrapper delegates add
The generated `b` function SHALL delegate `b add` and its arguments to `bmersive add`.

#### Scenario: Add current directory through wrapper
- **WHEN** the initialized user runs `b add`
- **THEN** the wrapper invokes `bmersive add`

### Requirement: Shell wrapper delegates remove
The generated `b` function SHALL delegate `b rm` and `b rm <index>` to `bmersive rm`.

#### Scenario: Remove through wrapper
- **WHEN** the initialized user runs `b rm 2`
- **THEN** the wrapper invokes `bmersive rm 2`

### Requirement: Shell wrapper delegates tmux
The generated `b` function SHALL delegate `b tmux` and its arguments to `bmersive tmux`.

#### Scenario: Tmux through wrapper
- **WHEN** the initialized user runs `b tmux panes`
- **THEN** the wrapper invokes `bmersive tmux panes`

### Requirement: Init command uses standardized CLI parsing
The system SHALL expose `bmersive init <shell>` through the standardized CLI parser while preserving existing shell initialization behavior.

#### Scenario: Supported shell is parsed
- **WHEN** the user runs `bmersive init zsh` or `bmersive init bash`
- **THEN** the parser accepts the shell argument and the system prints the corresponding shell integration code

#### Scenario: Unsupported shell is rejected
- **WHEN** the user runs `bmersive init <unsupported-shell>`
- **THEN** the system exits with an error and does not print shell integration code for that shell

### Requirement: Shell wrapper delegation remains compatible
The generated `b` function SHALL continue delegating to command forms accepted by the standardized CLI parser.

#### Scenario: Delegated command remains accepted
- **WHEN** the generated shell wrapper invokes `bmersive add`, `bmersive rm`, or `bmersive tmux` with user-provided arguments
- **THEN** the standardized CLI parser accepts the delegated command forms that were previously supported
