## Context

`bmersive` is a small Rust CLI with a stable command surface for session-local bookmarks, shell integration, and tmux materialization. The current implementation manually converts `env::args_os()` into strings, matches the first argument, and prints custom help text.

The project needs a more standardized CLI approach, but dependency policy is strict: no copyleft licenses, no stale versions, and only third-party crates with healthy community signals. The researched Rust equivalent to Cobra is `clap`; the closest Viper-like options are not needed yet because `bmersive` currently has only a small env-var configuration surface and no real config file.

## Goals / Non-Goals

**Goals:**
- Use `clap` as the standard command parser for top-level commands, subcommand arguments, generated help, and generated version output.
- Preserve existing command names, positional arguments, defaults, and shell-wrapper expectations.
- Keep bookmark state and environment-based configuration behavior unchanged.
- Keep dependency additions minimal and aligned with the permissive-license and healthy-community policy.

**Non-Goals:**
- Do not introduce a Viper-equivalent configuration framework in this change.
- Do not change bookmark state format or location precedence.
- Do not add shell completions or man-page generation yet; those can be follow-up changes if desired.
- Do not rename existing commands or introduce breaking CLI behavior.

## Decisions

### Use `clap` derive for the command model

Use `clap` with its `derive` feature to define a typed command enum and command-specific argument structs. This keeps the command surface visible in Rust types and lets `clap` generate standard help/version behavior.

Alternatives considered:
- Keep manual parsing: avoids a dependency but leaves help, validation, and future CLI growth ad hoc.
- Use a smaller parser: may reduce dependency weight but usually has weaker community/adoption signals than `clap`.
- Use `clap` builder API: also valid, but derive is simpler for the current command shape.

### Add only `clap` for this change

Add `clap` as the only new runtime dependency, pinned through Cargo's normal semver-compatible dependency management to a current stable major version. Do not add `config`, `figment`, or `confy` until there is a concrete config-file requirement.

Alternatives considered:
- Add `config` now: healthy and permissive, but unnecessary for the current env-only configuration surface.
- Add `figment`: powerful, but smaller-community and less current than `config` for strict dependency policy.
- Add `confy`: useful for simple config files, but `bmersive` does not yet need persisted app configuration distinct from bookmark state.

### Preserve behavior by adapting parsed commands to existing functions

The implementation should introduce typed CLI parsing at the boundary and then call the existing command functions or lightly adjusted equivalents. This minimizes behavioral risk and keeps most bookmark, shell, and tmux logic unchanged.

Alternatives considered:
- Rewrite command handlers around new abstractions: cleaner long term, but higher risk for a parser standardization change.
- Split into a library crate first: useful eventually, but unnecessary for this focused change.

### Treat generated help format as allowed to improve

`clap` generated help will differ from the current custom `print_help()` output. The functional requirement is that users get standard help and version output covering the supported commands; exact formatting should not be treated as a compatibility contract.

Alternatives considered:
- Recreate the current help text exactly: possible, but it undercuts the benefit of standard help generation.
- Keep custom help alongside `clap`: duplicates command metadata and risks drift.

## Risks / Trade-offs

- Help output snapshots or documentation may change → Update tests and README expectations to assert behavior rather than exact old formatting where possible.
- `clap` rejects some malformed argument combinations earlier than manual parsing → Add tests for missing arguments, unknown commands, and supported optional arguments.
- Dependency policy could become implicit over time → Document the dependency rationale in the change and keep future CLI helper crates (`clap_complete`, `clap_mangen`) as separate decisions.
- Adding a derive dependency increases compile work → Acceptable because `clap` is the Rust CLI standard with strong health signals and replaces custom parsing complexity.

## Migration Plan

1. Add `clap` with derive support.
2. Define the typed command model matching the existing command surface.
3. Route parsed commands to existing command handlers while preserving current behavior.
4. Replace custom help dispatch with `clap` generated help and version output.
5. Update or add CLI tests for command compatibility, generated help, version output, and validation.
6. Run formatting, tests, and build checks.

Rollback is straightforward: remove the `clap` dependency and restore manual parsing if the change causes unacceptable compatibility issues before release.

## Open Questions

- Should shell completions be generated in a future change via `clap_complete`?
- Should man pages be generated in a future change via `clap_mangen`?
- Should the README document the dependency policy explicitly, or should that remain internal project guidance?
