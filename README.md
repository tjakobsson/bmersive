# bmersive

`bmersive` is a curated, session-oriented directory workspace tool. It keeps a small explicit working set of folders, lets the shell jump by index, and can materialize the set into tmux layouts.

## Install

From this repository:

```sh
cargo install --path .
```

This installs the `bmersive` binary into Cargo's bin directory, usually `~/.cargo/bin`. Make sure that directory is on your `PATH`.

For local development without installing:

```sh
cargo run -- <command>
```

`bmersive --help` prints the generated command help, and `bmersive --version` prints the installed version.

## Shell Setup

Add one of these lines to your shell startup file.

For zsh, add this to `~/.zshrc`:

```sh
eval "$(bmersive init zsh)"
```

For bash, add this to `~/.bashrc`:

```sh
eval "$(bmersive init bash)"
```

Use `eval` here because `bmersive init <shell>` prints shell code. The current shell needs to evaluate that generated code so the `b` function is defined in the current shell process.

`source` is for reading shell code from a file. You could source a generated file if you created one, but the intended setup is direct evaluation:

```sh
bmersive init zsh > ~/.config/bmersive/init.zsh
source ~/.config/bmersive/init.zsh
```

The direct `eval "$(...)"` form avoids managing that extra generated file.

## State

The shell integration creates `BMERSIVE_STATE_DIR` when your shell starts. That keeps bookmarks scoped to that shell session instead of using one global file per user.

With shell integration enabled, the state file is:

```sh
$BMERSIVE_STATE_DIR/bookmarks.json
```

Without shell integration, `bmersive` falls back to:

```sh
$XDG_RUNTIME_DIR/bmersive/bookmarks.json
/tmp/bmersive-$USER/bookmarks.json
```

The `/tmp/bmersive-$USER/bookmarks.json` path is a last-resort fallback for direct CLI use when no session-specific state directory is available.

## Usage

```sh
b          # list bookmarks
b 2        # cd to bookmark 2
b add      # add the current directory
b rm       # choose a bookmark to remove
b rm 2     # remove bookmark 2
b tmux     # create or attach a tmux workspace using tiled panes
b tmux windows
```

By default, `bmersive` keeps up to 10 bookmarks. Override this with:

```sh
export BMERSIVE_MAX_BOOKMARKS=7
```

## Checks

```sh
cargo fmt --check
cargo test
cargo build
```
