## 1. Dependency Setup

- [x] 1.1 Add `clap` with derive support using a current stable major version and permissive license.
- [x] 1.2 Confirm no Viper-like configuration dependency is added as part of this change.

## 2. CLI Command Model

- [x] 2.1 Define a typed top-level CLI parser for `bmersive` with package name, version, and about metadata.
- [x] 2.2 Define typed subcommands for `init`, `add`, `ls`, `path`, `rm`, and `tmux` matching the existing command surface.
- [x] 2.3 Define typed value handling for supported shell arguments (`zsh`, `bash`) and tmux modes (`windows`, `panes`).
- [x] 2.4 Route parsed commands to the existing command handlers or minimal equivalents without changing bookmark, shell, or tmux behavior.

## 3. Behavior Preservation

- [x] 3.1 Preserve `bmersive add [path]`, duplicate handling, max bookmark handling, and state persistence behavior.
- [x] 3.2 Preserve `bmersive ls`, `bmersive path <index>`, and `bmersive rm [index]` behavior.
- [x] 3.3 Preserve `bmersive init zsh` and `bmersive init bash` output behavior used by the generated `b` wrapper.
- [x] 3.4 Preserve `bmersive tmux`, `bmersive tmux panes`, and `bmersive tmux windows` behavior.

## 4. Help, Version, and Validation

- [x] 4.1 Replace custom help dispatch with parser-generated top-level help that lists supported commands.
- [x] 4.2 Add command-specific help support for supported subcommands.
- [x] 4.3 Add parser-generated version output using the Cargo package version.
- [x] 4.4 Ensure unknown commands and parser-detectable missing required arguments fail before command side effects.

## 5. Tests and Documentation

- [x] 5.1 Update existing CLI tests to pass with the standardized parser while preserving behavior assertions.
- [x] 5.2 Add tests for top-level help, command-specific help, version output, unknown commands, and missing required arguments.
- [x] 5.3 Update README command/help documentation if generated help or version behavior changes user-facing guidance.

## 6. Verification

- [x] 6.1 Run `cargo fmt --check`.
- [x] 6.2 Run `cargo test`.
- [x] 6.3 Run `cargo build`.
