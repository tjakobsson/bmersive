## ADDED Requirements

### Requirement: Bookmark commands use standardized CLI parsing
The system SHALL expose bookmark commands through the standardized CLI parser while preserving their existing bookmark behavior.

#### Scenario: Add command through parser
- **WHEN** the user runs `bmersive add` or `bmersive add <path>`
- **THEN** the parser accepts the command and the system applies the existing add bookmark behavior

#### Scenario: List command through parser
- **WHEN** the user runs `bmersive ls`
- **THEN** the parser accepts the command and the system applies the existing list bookmark behavior

#### Scenario: Path command through parser
- **WHEN** the user runs `bmersive path <index>`
- **THEN** the parser accepts the command and the system applies the existing path lookup behavior

#### Scenario: Remove command through parser
- **WHEN** the user runs `bmersive rm` or `bmersive rm <index>`
- **THEN** the parser accepts the command and the system applies the existing remove bookmark behavior

### Requirement: Bookmark parser validation avoids state changes
The system SHALL reject invalid bookmark command forms before reading from or writing to bookmark state when validation can be performed by the CLI parser.

#### Scenario: Missing path index
- **WHEN** the user runs `bmersive path` without an index
- **THEN** the system exits with CLI usage guidance and does not modify bookmark state
