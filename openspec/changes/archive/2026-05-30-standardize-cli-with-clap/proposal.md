## Why

`bmersive` currently parses commands manually, which makes command validation, help text, and future CLI growth ad hoc. Standardizing on a healthy, permissively licensed Rust CLI crate gives the project a consistent command model without expanding the configuration surface prematurely.

## What Changes

- Replace manual top-level command parsing with a typed `clap` command model.
- Preserve the existing public command surface and behavior for `init`, `add`, `ls`, `path`, `rm`, and `tmux`.
- Generate standard help and version output from the CLI parser instead of maintaining custom help text by hand.
- Use only current, permissively licensed, healthy-community third-party dependencies for CLI standardization.
- Defer any Viper-like configuration framework until `bmersive` has a real config-file requirement.

## Capabilities

### New Capabilities
- `command-line-interface`: Defines the standard CLI command surface, generated help/version behavior, typed argument validation, and dependency policy for CLI parsing.

### Modified Capabilities
- `session-bookmarks`: Bookmark command requirements are refined to preserve existing behavior under the standardized CLI parser.
- `shell-integration`: Shell initialization and wrapper delegation requirements are refined to preserve existing behavior under the standardized CLI parser.
- `tmux-materialization`: Tmux command requirements are refined to preserve existing behavior under the standardized CLI parser.

## Impact

- Affected code: `src/main.rs`, CLI integration tests in `tests/cli.rs`, and user-facing command/help output in `README.md` if needed.
- Dependencies: add `clap` with derive support, using a current stable release and permissive license (`MIT OR Apache-2.0`).
- Behavior: existing commands should remain compatible; generated help/version output may change format but should become more standard and complete.
- Non-goals: no application-code implementation in this proposal step, no config-file system, and no Viper-equivalent dependency.
