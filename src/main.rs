use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const DEFAULT_MAX_BOOKMARKS: usize = 10;
const SESSION_NAME: &str = "bmersive";

#[derive(Debug)]
struct CliError(String);

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct BookmarkState {
    bookmarks: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum TmuxAction {
    HasSession {
        session: String,
    },
    AttachSession {
        session: String,
    },
    NewSessionDetached {
        session: String,
        cwd: String,
        name: String,
    },
    NewWindow {
        target: Option<String>,
        cwd: String,
        name: String,
    },
    SplitWindow {
        target: Option<String>,
        cwd: String,
    },
    SelectLayoutTiled {
        target: Option<String>,
    },
}

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bmersive: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<OsString>) -> Result<(), CliError> {
    let args = args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| CliError::new("arguments must be valid UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    match args.first().map(String::as_str) {
        Some("init") => init(args.get(1).map(String::as_str)),
        Some("add") => add(args.get(1).map(String::as_str)),
        Some("ls") => list(),
        Some("path") => path(args.get(1).map(String::as_str)),
        Some("rm") => remove(args.get(1).map(String::as_str)),
        Some("tmux") => tmux(args.get(1).map(String::as_str)),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(CliError::new(format!("unknown command: {command}"))),
    }
}

fn print_help() {
    println!("bmersive commands:");
    println!("  init <zsh|bash>");
    println!("  add [path]");
    println!("  ls");
    println!("  path <index>");
    println!("  rm [index]");
    println!("  tmux [windows|panes]");
}

fn state_path() -> PathBuf {
    state_path_from(
        env::var_os("BMERSIVE_STATE_DIR"),
        env::var_os("XDG_RUNTIME_DIR"),
        env::var("USER").ok(),
    )
}

fn state_path_from(
    state_dir: Option<OsString>,
    runtime_dir: Option<OsString>,
    user: Option<String>,
) -> PathBuf {
    if let Some(state_dir) = state_dir.filter(|value| !value.is_empty()) {
        return PathBuf::from(state_dir).join("bookmarks.json");
    }

    if let Some(runtime_dir) = runtime_dir.filter(|value| !value.is_empty()) {
        return PathBuf::from(runtime_dir)
            .join("bmersive")
            .join("bookmarks.json");
    }

    let user = user.unwrap_or_else(|| "unknown".to_string());
    PathBuf::from(format!("/tmp/bmersive-{user}")).join("bookmarks.json")
}

fn load_state(path: &Path) -> Result<BookmarkState, CliError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BookmarkState::default()),
        Err(error) => Err(error.into()),
    }
}

fn save_state(path: &Path, state: &BookmarkState) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(state)?;
    fs::write(path, format!("{contents}\n"))?;
    Ok(())
}

fn max_bookmarks() -> usize {
    max_bookmarks_from(env::var("BMERSIVE_MAX_BOOKMARKS").ok().as_deref())
}

fn max_bookmarks_from(value: Option<&str>) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_BOOKMARKS)
}

fn normalize_path(path: &Path) -> Result<PathBuf, CliError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    Ok(normalize_absolute_path(&absolute))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
        }
    }

    normalized
}

fn add(path_arg: Option<&str>) -> Result<(), CliError> {
    let target = path_arg.map(PathBuf::from).unwrap_or(env::current_dir()?);
    let normalized = normalize_path(&target)?;
    let bookmark = path_to_string(&normalized)?;
    let path = state_path();
    let mut state = load_state(&path)?;

    if state.bookmarks.contains(&bookmark) {
        println!("Already bookmarked: {bookmark}");
        return Ok(());
    }

    let max = max_bookmarks();
    if state.bookmarks.len() >= max {
        return Err(CliError::new(format!(
            "bookmark list is full ({max} max). Remove one with: b rm"
        )));
    }

    state.bookmarks.push(bookmark.clone());
    save_state(&path, &state)?;
    println!(
        "Added [{index}] {bookmark}",
        index = state.bookmarks.len() - 1
    );
    Ok(())
}

fn list() -> Result<(), CliError> {
    let state = load_state(&state_path())?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks. Add one with: b add");
        return Ok(());
    }

    for (index, bookmark) in state.bookmarks.iter().enumerate() {
        println!("[{index}] {bookmark}");
    }

    Ok(())
}

fn path(index_arg: Option<&str>) -> Result<(), CliError> {
    let index = parse_index(index_arg)?;
    let state = load_state(&state_path())?;
    let bookmark = state
        .bookmarks
        .get(index)
        .ok_or_else(|| CliError::new(format!("bookmark index {index} does not exist")))?;

    println!("{bookmark}");
    Ok(())
}

fn remove(index_arg: Option<&str>) -> Result<(), CliError> {
    let path = state_path();
    let mut state = load_state(&path)?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks to remove.");
        return Ok(());
    }

    let index = match index_arg {
        Some(_) => parse_index(index_arg)?,
        None => prompt_remove_index(&state)?,
    };

    if index >= state.bookmarks.len() {
        return Err(CliError::new(format!(
            "bookmark index {index} does not exist"
        )));
    }

    let removed = state.bookmarks.remove(index);
    save_state(&path, &state)?;
    println!("Removed [{index}] {removed}");
    Ok(())
}

fn prompt_remove_index(state: &BookmarkState) -> Result<usize, CliError> {
    for (index, bookmark) in state.bookmarks.iter().enumerate() {
        println!("[{index}] {bookmark}");
    }

    print!("Remove index: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    parse_index(Some(input.trim()))
}

fn parse_index(index_arg: Option<&str>) -> Result<usize, CliError> {
    index_arg
        .ok_or_else(|| CliError::new("missing bookmark index"))?
        .parse::<usize>()
        .map_err(|_| CliError::new("bookmark index must be a non-negative integer"))
}

fn init(shell: Option<&str>) -> Result<(), CliError> {
    match shell {
        Some("zsh") | Some("bash") => {
            println!("{}", shell_function());
            Ok(())
        }
        Some(other) => Err(CliError::new(format!("unsupported shell: {other}"))),
        None => Err(CliError::new("missing shell: expected zsh or bash")),
    }
}

fn shell_function() -> &'static str {
    r#"if [ -z "${BMERSIVE_STATE_DIR:-}" ]; then
  export BMERSIVE_STATE_DIR="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}/bmersive-${USER:-unknown}-$$"
fi

b() {
  if [ "$#" -eq 0 ]; then
    bmersive ls
    return
  fi

  case "$1" in
    add)
      shift
      bmersive add "$@"
      ;;
    rm)
      shift
      bmersive rm "$@"
      ;;
    tmux)
      shift
      bmersive tmux "$@"
      ;;
    *)
      case "$1" in
        *[!0-9]*)
          bmersive "$@"
          ;;
        *)
          cd "$(bmersive path "$1")"
          ;;
      esac
      ;;
  esac
}"#
}

fn tmux(mode_arg: Option<&str>) -> Result<(), CliError> {
    let mode = mode_arg.unwrap_or(default_tmux_mode());
    let state = load_state(&state_path())?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks to materialize. Add one with: b add");
        return Ok(());
    }

    let inside_tmux = env::var_os("TMUX").is_some();

    match (mode, inside_tmux) {
        ("windows", true) => run_tmux_actions(tmux_windows_actions(&state.bookmarks, true)),
        ("panes", true) => run_tmux_actions(tmux_panes_actions(&state.bookmarks, true)),
        ("windows", false) => tmux_outside_windows(&state.bookmarks),
        ("panes", false) => tmux_outside_panes(&state.bookmarks),
        (other, _) => Err(CliError::new(format!("unsupported tmux mode: {other}"))),
    }
}

fn default_tmux_mode() -> &'static str {
    "panes"
}

fn tmux_windows_actions(bookmarks: &[String], inside_tmux: bool) -> Vec<TmuxAction> {
    if inside_tmux {
        return bookmarks
            .iter()
            .map(|bookmark| TmuxAction::NewWindow {
                target: None,
                cwd: bookmark.clone(),
                name: basename(bookmark),
            })
            .collect();
    }

    let mut actions = vec![TmuxAction::HasSession {
        session: SESSION_NAME.to_string(),
    }];

    if let Some(first) = bookmarks.first() {
        actions.push(TmuxAction::NewSessionDetached {
            session: SESSION_NAME.to_string(),
            cwd: first.clone(),
            name: basename(first),
        });

        actions.extend(
            bookmarks
                .iter()
                .skip(1)
                .map(|bookmark| TmuxAction::NewWindow {
                    target: None,
                    cwd: bookmark.clone(),
                    name: basename(bookmark),
                }),
        );
    }

    actions.push(TmuxAction::AttachSession {
        session: SESSION_NAME.to_string(),
    });
    actions
}

fn tmux_panes_actions(bookmarks: &[String], inside_tmux: bool) -> Vec<TmuxAction> {
    let mut actions = Vec::new();

    if inside_tmux {
        if let Some(first) = bookmarks.first() {
            actions.push(TmuxAction::NewWindow {
                target: None,
                cwd: first.clone(),
                name: basename(first),
            });
        }
        actions.extend(
            bookmarks
                .iter()
                .skip(1)
                .map(|bookmark| TmuxAction::SplitWindow {
                    target: None,
                    cwd: bookmark.clone(),
                }),
        );
        actions.push(TmuxAction::SelectLayoutTiled { target: None });
        return actions;
    }

    if let Some(first) = bookmarks.first() {
        actions.push(TmuxAction::HasSession {
            session: SESSION_NAME.to_string(),
        });
        actions.push(TmuxAction::NewSessionDetached {
            session: SESSION_NAME.to_string(),
            cwd: first.clone(),
            name: basename(first),
        });
        actions.extend(
            bookmarks
                .iter()
                .skip(1)
                .map(|bookmark| TmuxAction::SplitWindow {
                    target: None,
                    cwd: bookmark.clone(),
                }),
        );
        actions.push(TmuxAction::SelectLayoutTiled { target: None });
        actions.push(TmuxAction::AttachSession {
            session: SESSION_NAME.to_string(),
        });
    }

    actions
}

fn tmux_outside_windows(bookmarks: &[String]) -> Result<(), CliError> {
    if tmux_session_exists(SESSION_NAME)? {
        return run_tmux(["attach-session", "-t", SESSION_NAME]);
    }

    run_tmux_dynamic(tmux_outside_windows_args(bookmarks))
}

fn tmux_outside_panes(bookmarks: &[String]) -> Result<(), CliError> {
    if tmux_session_exists(SESSION_NAME)? {
        return run_tmux(["attach-session", "-t", SESSION_NAME]);
    }

    run_tmux_dynamic(tmux_outside_panes_args(bookmarks))
}

fn tmux_outside_windows_args(bookmarks: &[String]) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(first) = bookmarks.first() {
        args.extend(strings([
            "new-session",
            "-d",
            "-s",
            SESSION_NAME,
            "-c",
            first,
            "-n",
            &basename(first),
        ]));

        for bookmark in bookmarks.iter().skip(1) {
            args.push(";".to_string());
            args.extend(strings([
                "new-window",
                "-d",
                "-c",
                bookmark,
                "-n",
                &basename(bookmark),
            ]));
        }

        args.push(";".to_string());
        args.extend(strings(["attach-session", "-t", SESSION_NAME]));
    }

    args
}

fn tmux_outside_panes_args(bookmarks: &[String]) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(first) = bookmarks.first() {
        args.extend(strings([
            "new-session",
            "-d",
            "-s",
            SESSION_NAME,
            "-c",
            first,
            "-n",
            &basename(first),
        ]));

        for bookmark in bookmarks.iter().skip(1) {
            args.push(";".to_string());
            args.extend(strings(["split-window", "-c", bookmark]));
        }

        args.push(";".to_string());
        args.extend(strings(["select-layout", "tiled"]));
        args.push(";".to_string());
        args.extend(strings(["attach-session", "-t", SESSION_NAME]));
    }

    args
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn tmux_session_exists(session: &str) -> Result<bool, CliError> {
    Ok(Command::new("tmux")
        .args(["has-session", "-t", session])
        .stderr(Stdio::null())
        .status()
        .map_err(|error| CliError::new(format!("failed to run tmux: {error}")))?
        .success())
}

fn run_tmux_actions(actions: Vec<TmuxAction>) -> Result<(), CliError> {
    let mut session_exists = false;

    for action in actions {
        match action {
            TmuxAction::HasSession { session } => {
                session_exists = Command::new("tmux")
                    .args(["has-session", "-t", &session])
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|error| CliError::new(format!("failed to run tmux: {error}")))?
                    .success();

                if session_exists {
                    continue;
                }
            }
            TmuxAction::AttachSession { session } => {
                run_tmux(["attach-session", "-t", &session])?;
            }
            TmuxAction::NewSessionDetached { session, cwd, name } => {
                if session_exists {
                    continue;
                }
                run_tmux(["new-session", "-d", "-s", &session, "-c", &cwd, "-n", &name])?;
            }
            TmuxAction::NewWindow { target, cwd, name } => {
                if session_exists {
                    continue;
                }
                if let Some(target) = target {
                    run_tmux(["new-window", "-t", &target, "-c", &cwd, "-n", &name])?;
                } else {
                    run_tmux(["new-window", "-c", &cwd, "-n", &name])?;
                }
            }
            TmuxAction::SplitWindow { target, cwd } => {
                if session_exists {
                    continue;
                }
                if let Some(target) = target {
                    run_tmux(["split-window", "-t", &target, "-c", &cwd])?;
                } else {
                    run_tmux(["split-window", "-c", &cwd])?;
                }
            }
            TmuxAction::SelectLayoutTiled { target } => {
                if session_exists {
                    continue;
                }
                if let Some(target) = target {
                    run_tmux(["select-layout", "-t", &target, "tiled"])?;
                } else {
                    run_tmux(["select-layout", "tiled"])?;
                }
            }
        }
    }

    Ok(())
}

fn run_tmux<const N: usize>(args: [&str; N]) -> Result<(), CliError> {
    let status = Command::new("tmux")
        .args(args)
        .status()
        .map_err(|error| CliError::new(format!("failed to run tmux: {error}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::new("tmux command failed"))
    }
}

fn run_tmux_dynamic(args: Vec<String>) -> Result<(), CliError> {
    let status = Command::new("tmux")
        .args(args)
        .status()
        .map_err(|error| CliError::new(format!("failed to run tmux: {error}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::new("tmux command failed"))
    }
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(SESSION_NAME)
        .to_string()
}

fn path_to_string(path: &Path) -> Result<String, CliError> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| CliError::new("path must be valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_relative_segments_without_resolving_symlinks() {
        let path = normalize_absolute_path(Path::new("/work/link/../link/project/./src"));
        assert_eq!(path, PathBuf::from("/work/link/project/src"));
    }

    #[test]
    fn state_path_prefers_runtime_dir() {
        assert_eq!(
            state_path_from(
                None,
                Some(OsString::from("/run/user/123")),
                Some("me".to_string())
            ),
            PathBuf::from("/run/user/123/bmersive/bookmarks.json")
        );
    }

    #[test]
    fn state_path_prefers_explicit_state_dir() {
        assert_eq!(
            state_path_from(
                Some(OsString::from("/tmp/bmersive-session")),
                Some(OsString::from("/run/user/123")),
                Some("me".to_string())
            ),
            PathBuf::from("/tmp/bmersive-session/bookmarks.json")
        );
    }

    #[test]
    fn state_path_falls_back_to_tmp_user_dir() {
        assert_eq!(
            state_path_from(None, None, Some("testuser".to_string())),
            PathBuf::from("/tmp/bmersive-testuser/bookmarks.json")
        );
    }

    #[test]
    fn max_bookmarks_uses_positive_config_or_default() {
        assert_eq!(max_bookmarks_from(Some("5")), 5);
        assert_eq!(max_bookmarks_from(Some("0")), DEFAULT_MAX_BOOKMARKS);
        assert_eq!(max_bookmarks_from(Some("nope")), DEFAULT_MAX_BOOKMARKS);
        assert_eq!(max_bookmarks_from(None), DEFAULT_MAX_BOOKMARKS);
    }

    #[test]
    fn basename_uses_final_path_component() {
        assert_eq!(basename("/work/api"), "api");
    }

    #[test]
    fn default_tmux_mode_is_panes() {
        assert_eq!(default_tmux_mode(), "panes");
    }

    #[test]
    fn parse_index_accepts_non_negative_integer() {
        assert_eq!(parse_index(Some("2")).expect("parse valid index"), 2);
        assert!(parse_index(Some("nope")).is_err());
    }

    #[test]
    fn tmux_windows_inside_current_session_creates_windows() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_windows_actions(&bookmarks, true),
            vec![
                TmuxAction::NewWindow {
                    target: None,
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::NewWindow {
                    target: None,
                    cwd: "/work/web".to_string(),
                    name: "web".to_string()
                }
            ]
        );
    }

    #[test]
    fn tmux_windows_outside_checks_session_then_creates_and_attaches() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_windows_actions(&bookmarks, false),
            vec![
                TmuxAction::HasSession {
                    session: "bmersive".to_string()
                },
                TmuxAction::NewSessionDetached {
                    session: "bmersive".to_string(),
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::NewWindow {
                    target: None,
                    cwd: "/work/web".to_string(),
                    name: "web".to_string()
                },
                TmuxAction::AttachSession {
                    session: "bmersive".to_string()
                }
            ]
        );
    }

    #[test]
    fn tmux_panes_inside_splits_additional_bookmarks_and_tiles() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_panes_actions(&bookmarks, true),
            vec![
                TmuxAction::NewWindow {
                    target: None,
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::SplitWindow {
                    target: None,
                    cwd: "/work/web".to_string()
                },
                TmuxAction::SelectLayoutTiled { target: None }
            ]
        );
    }

    #[test]
    fn tmux_panes_outside_targets_created_session() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_panes_actions(&bookmarks, false),
            vec![
                TmuxAction::HasSession {
                    session: "bmersive".to_string()
                },
                TmuxAction::NewSessionDetached {
                    session: "bmersive".to_string(),
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::SplitWindow {
                    target: None,
                    cwd: "/work/web".to_string()
                },
                TmuxAction::SelectLayoutTiled { target: None },
                TmuxAction::AttachSession {
                    session: "bmersive".to_string()
                }
            ]
        );
    }

    #[test]
    fn tmux_outside_windows_uses_single_command_sequence() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_outside_windows_args(&bookmarks),
            vec![
                "new-session",
                "-d",
                "-s",
                "bmersive",
                "-c",
                "/work/api",
                "-n",
                "api",
                ";",
                "new-window",
                "-d",
                "-c",
                "/work/web",
                "-n",
                "web",
                ";",
                "attach-session",
                "-t",
                "bmersive"
            ]
        );
    }

    #[test]
    fn tmux_outside_panes_uses_single_command_sequence() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_outside_panes_args(&bookmarks),
            vec![
                "new-session",
                "-d",
                "-s",
                "bmersive",
                "-c",
                "/work/api",
                "-n",
                "api",
                ";",
                "split-window",
                "-c",
                "/work/web",
                ";",
                "select-layout",
                "tiled",
                ";",
                "attach-session",
                "-t",
                "bmersive"
            ]
        );
    }
}
