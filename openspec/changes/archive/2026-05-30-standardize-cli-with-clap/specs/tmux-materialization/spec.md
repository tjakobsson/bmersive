## ADDED Requirements

### Requirement: Tmux command uses standardized CLI parsing
The system SHALL expose `bmersive tmux [mode]` through the standardized CLI parser while preserving existing tmux materialization behavior.

#### Scenario: Default tmux mode through parser
- **WHEN** the user runs `bmersive tmux`
- **THEN** the parser accepts the omitted mode and the system uses panes mode

#### Scenario: Explicit tmux mode through parser
- **WHEN** the user runs `bmersive tmux windows` or `bmersive tmux panes`
- **THEN** the parser accepts the mode and the system applies the existing tmux behavior for that mode

#### Scenario: Unsupported tmux mode is rejected
- **WHEN** the user runs `bmersive tmux <unsupported-mode>`
- **THEN** the system exits with an error and does not perform tmux side effects
