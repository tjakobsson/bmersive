## ADDED Requirements

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
