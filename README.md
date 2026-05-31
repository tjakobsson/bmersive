# bmersive

`bmersive` is a curated, session-oriented directory workspace tool. It keeps small explicit working sets of folders, lets the shell jump by index, and can materialize a project session into tmux layouts.

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

`bmersive` stores saved sessions separately from the running shell. Each saved session has its own bookmark list, so different projects can keep different working sets.

Saved sessions live in:

```sh
$XDG_STATE_HOME/bmersive/sessions.json
$HOME/.local/state/bmersive/sessions.json
/tmp/bmersive-$USER/sessions.json
```

The shell integration creates `BMERSIVE_STATE_DIR` when your shell starts. That directory stores only the appointed session for that running shell:

```sh
$BMERSIVE_STATE_DIR/session
```

If an old single-session `bookmarks.json` exists and no saved-session state exists yet, `bmersive` imports those bookmarks into a `default` session.

## Usage

```sh
b session new api    # create a saved session
b session use api    # appoint it and jump to bookmark 0 when available
b session use api --no-jump
b session unset      # clear the appointed session for this shell
b session ls         # list saved sessions
b                    # list bookmarks, or choose a session and jump to bookmark 0
b 2                  # cd to bookmark 2 in the appointed session
b add                # add the current directory to the appointed session
b rm                 # choose a bookmark to remove
b rm 2               # remove bookmark 2
b tmux               # create or attach a tmux workspace using tiled panes
b tmux windows
```

By default, `bmersive` keeps up to 10 bookmarks. Override this with:

```sh
export BMERSIVE_MAX_BOOKMARKS=7
```

## Demo

Create two project sessions:

```sh
b session new web-app
b session new infra
```

Start a shell with no appointed session and run `b`. The wrapper asks which saved session you want to enter. If that session already has bookmarks, the shell jumps to index 0 after selection. With an empty session, it tells you there is nowhere to jump yet:

```text
$ b
[0] infra
[1] web-app
Session index: 1
Using session: web-app
No bookmark at [0] to jump to. Add one with: b add
```

Add the folders that make up the project:

```sh
cd ~/src/web-app/api && b add
cd ~/src/web-app/frontend && b add
cd ~/src/web-app/worker && b add
cd ~/src/web-app/docs && b add
```

Now `b` is the bookmark chooser for the appointed session:

```text
$ b
[0] /Users/me/src/web-app/api
[1] /Users/me/src/web-app/frontend
[2] /Users/me/src/web-app/worker
[3] /Users/me/src/web-app/docs

$ b 1
# shell changes directory to /Users/me/src/web-app/frontend
```

Switching back to a populated session jumps straight to its first bookmark:

```text
$ b session use web-app
Using session: web-app
Jumping to [0] /Users/me/src/web-app/api
# shell changes directory to /Users/me/src/web-app/api
```

Use `--no-jump` when you only want to appoint the session:

```sh
b session use web-app --no-jump
b session choose --no-jump
```

Unset the current shell appointment when you want `b` to ask again next time:

```sh
b session unset
```

Materialize the same session into tmux panes:

```sh
b tmux
```

The four bookmarked folders become a tiled project workspace:

```text
+---------------------------+---------------------------+
| api                       | frontend                  |
| ~/src/web-app/api         | ~/src/web-app/frontend    |
+---------------------------+---------------------------+
| worker                    | docs                      |
| ~/src/web-app/worker      | ~/src/web-app/docs        |
+---------------------------+---------------------------+
```

Outside tmux, the tmux session is named from the appointed `bmersive` session, for example `bmersive-web-app`, so different project sessions do not collide.

## Checks

```sh
cargo fmt --check
cargo test
cargo build
```
