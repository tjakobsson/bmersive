use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const DEFAULT_MAX_BOOKMARKS: usize = 10;
const SESSION_NAME: &str = "bmersive";
const DEFAULT_IMPORTED_SESSION: &str = "default";

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Emit shell integration code.
    Init { shell: Shell },
    /// Add the current directory or an explicit path.
    Add { path: Option<PathBuf> },
    /// List bookmarked directories.
    Ls,
    /// Print the bookmarked path for an index.
    Path { index: usize },
    /// Remove a bookmark by index, or prompt when omitted.
    Rm { index: Option<usize> },
    /// Manage saved bookmark sessions.
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Create or attach to a tmux workspace.
    Tmux { mode: Option<TmuxMode> },
}

#[derive(Debug, Subcommand)]
enum SessionCommand {
    /// List saved sessions.
    Ls,
    /// Create a saved session.
    New { name: String },
    /// Appoint a saved session for this shell.
    Use {
        name: String,
        /// Appoint the session without jumping to bookmark 0.
        #[arg(long)]
        no_jump: bool,
    },
    /// Choose and appoint a saved session interactively.
    Choose {
        /// Appoint the session without jumping to bookmark 0.
        #[arg(long)]
        no_jump: bool,
    },
    /// Clear the appointed session for this shell.
    Unset,
    /// Print the appointed session.
    Current,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Shell {
    Zsh,
    Bash,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TmuxMode {
    Windows,
    Panes,
}

impl TmuxMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Panes => "panes",
        }
    }
}

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

#[derive(Debug, Default, Serialize, Deserialize)]
struct SessionsState {
    sessions: BTreeMap<String, BookmarkState>,
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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };

    match execute(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bmersive: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Some(CliCommand::Init { shell }) => init(shell),
        Some(CliCommand::Add { path }) => add(path),
        Some(CliCommand::Ls) => list(),
        Some(CliCommand::Path { index }) => path(index),
        Some(CliCommand::Rm { index }) => remove(index),
        Some(CliCommand::Session { command }) => session(command),
        Some(CliCommand::Tmux { mode }) => tmux(mode),
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

fn state_path() -> PathBuf {
    state_path_from(
        env::var_os("BMERSIVE_STATE_DIR"),
        env::var_os("XDG_RUNTIME_DIR"),
        env::var("USER").ok(),
    )
}

fn sessions_state_path() -> PathBuf {
    sessions_state_path_from(
        env::var_os("XDG_STATE_HOME"),
        env::var_os("HOME"),
        env::var("USER").ok(),
    )
}

fn sessions_state_path_from(
    xdg_state_home: Option<OsString>,
    home: Option<OsString>,
    user: Option<String>,
) -> PathBuf {
    if let Some(state_home) = xdg_state_home.filter(|value| !value.is_empty()) {
        return PathBuf::from(state_home)
            .join("bmersive")
            .join("sessions.json");
    }

    if let Some(home) = home.filter(|value| !value.is_empty()) {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("bmersive")
            .join("sessions.json");
    }

    let user = user.unwrap_or_else(|| "unknown".to_string());
    PathBuf::from(format!("/tmp/bmersive-{user}")).join("sessions.json")
}

fn appointment_path() -> Result<PathBuf, CliError> {
    appointment_path_from(env::var_os("BMERSIVE_STATE_DIR"))
}

fn appointment_path_from(state_dir: Option<OsString>) -> Result<PathBuf, CliError> {
    state_dir
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join("session"))
        .ok_or_else(|| CliError::new("no session selected. Run: b session choose"))
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

#[cfg(test)]
fn save_state(path: &Path, state: &BookmarkState) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(state)?;
    fs::write(path, format!("{contents}\n"))?;
    Ok(())
}

fn load_sessions_state() -> Result<SessionsState, CliError> {
    load_sessions_state_from(&sessions_state_path(), &state_path())
}

fn load_sessions_state_from(path: &Path, legacy_path: &Path) -> Result<SessionsState, CliError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => import_legacy_state(legacy_path),
        Err(error) => Err(error.into()),
    }
}

fn import_legacy_state(legacy_path: &Path) -> Result<SessionsState, CliError> {
    let legacy = load_state(legacy_path)?;
    let mut state = SessionsState::default();

    if !legacy.bookmarks.is_empty() {
        state
            .sessions
            .insert(DEFAULT_IMPORTED_SESSION.to_string(), legacy);
    }

    Ok(state)
}

fn save_sessions_state(state: &SessionsState) -> Result<(), CliError> {
    save_sessions_state_to(&sessions_state_path(), state)
}

fn save_sessions_state_to(path: &Path, state: &SessionsState) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(state)?;
    fs::write(path, format!("{contents}\n"))?;
    Ok(())
}

fn load_appointed_session() -> Result<String, CliError> {
    load_appointed_session_from(&appointment_path()?)
}

fn load_appointed_session_from(path: &Path) -> Result<String, CliError> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let name = contents.trim();
            if name.is_empty() {
                Err(CliError::new("no session selected. Run: b session choose"))
            } else {
                Ok(name.to_string())
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(CliError::new("no session selected. Run: b session choose"))
        }
        Err(error) => Err(error.into()),
    }
}

fn save_appointed_session(name: &str) -> Result<(), CliError> {
    save_appointed_session_to(&appointment_path()?, name)
}

fn save_appointed_session_to(path: &Path, name: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, format!("{name}\n"))?;
    Ok(())
}

fn unset_appointed_session() -> Result<(), CliError> {
    unset_appointed_session_from(&appointment_path()?)
}

fn unset_appointed_session_from(path: &Path) -> Result<(), CliError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn appointed_bookmarks<'a>(
    state: &'a SessionsState,
    name: &str,
) -> Result<&'a BookmarkState, CliError> {
    state
        .sessions
        .get(name)
        .ok_or_else(|| CliError::new(format!("selected session '{name}' does not exist")))
}

fn appointed_bookmarks_mut<'a>(
    state: &'a mut SessionsState,
    name: &str,
) -> Result<&'a mut BookmarkState, CliError> {
    state
        .sessions
        .get_mut(name)
        .ok_or_else(|| CliError::new(format!("selected session '{name}' does not exist")))
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

fn add(path_arg: Option<PathBuf>) -> Result<(), CliError> {
    let target = path_arg.unwrap_or(env::current_dir()?);
    let normalized = normalize_path(&target)?;
    let bookmark = path_to_string(&normalized)?;
    let session_name = load_appointed_session()?;
    let mut sessions = load_sessions_state()?;
    let state = appointed_bookmarks_mut(&mut sessions, &session_name)?;

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
    let index = state.bookmarks.len() - 1;
    save_sessions_state(&sessions)?;
    println!("Added [{index}] {bookmark}");
    Ok(())
}

fn list() -> Result<(), CliError> {
    let session_name = load_appointed_session()?;
    let sessions = load_sessions_state()?;
    let state = appointed_bookmarks(&sessions, &session_name)?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks. Add one with: b add");
        return Ok(());
    }

    for (index, bookmark) in state.bookmarks.iter().enumerate() {
        println!("[{index}] {bookmark}");
    }

    Ok(())
}

fn path(index: usize) -> Result<(), CliError> {
    let session_name = load_appointed_session()?;
    let sessions = load_sessions_state()?;
    let state = appointed_bookmarks(&sessions, &session_name)?;
    let bookmark = state
        .bookmarks
        .get(index)
        .ok_or_else(|| CliError::new(format!("bookmark index {index} does not exist")))?;

    println!("{bookmark}");
    Ok(())
}

fn remove(index_arg: Option<usize>) -> Result<(), CliError> {
    let session_name = load_appointed_session()?;
    let mut sessions = load_sessions_state()?;
    let state = appointed_bookmarks_mut(&mut sessions, &session_name)?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks to remove.");
        return Ok(());
    }

    let index = match index_arg {
        Some(index) => index,
        None => prompt_remove_index(state)?,
    };

    if index >= state.bookmarks.len() {
        return Err(CliError::new(format!(
            "bookmark index {index} does not exist"
        )));
    }

    let removed = state.bookmarks.remove(index);
    save_sessions_state(&sessions)?;
    println!("Removed [{index}] {removed}");
    Ok(())
}

fn session(command: SessionCommand) -> Result<(), CliError> {
    match command {
        SessionCommand::Ls => list_sessions(),
        SessionCommand::New { name } => create_session(&name),
        SessionCommand::Use { name, no_jump } => use_session(&name, no_jump),
        SessionCommand::Choose { no_jump } => choose_session(no_jump),
        SessionCommand::Unset => unset_session(),
        SessionCommand::Current => {
            println!("{}", load_appointed_session()?);
            Ok(())
        }
    }
}

fn validate_session_name(name: &str) -> Result<(), CliError> {
    let valid = !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');

    if valid {
        Ok(())
    } else {
        Err(CliError::new(
            "session name must use lowercase letters, numbers, and hyphens",
        ))
    }
}

fn create_session(name: &str) -> Result<(), CliError> {
    validate_session_name(name)?;
    let mut state = load_sessions_state()?;

    if state.sessions.contains_key(name) {
        return Err(CliError::new(format!("session '{name}' already exists")));
    }

    state
        .sessions
        .insert(name.to_string(), BookmarkState::default());
    save_sessions_state(&state)?;
    println!("Created session: {name}");
    Ok(())
}

fn list_sessions() -> Result<(), CliError> {
    let state = load_sessions_state()?;

    if state.sessions.is_empty() {
        println!("No sessions. Create one with: b session new <name>");
        return Ok(());
    }

    for (index, name) in state.sessions.keys().enumerate() {
        println!("[{index}] {name}");
    }

    Ok(())
}

fn use_session(name: &str, no_jump: bool) -> Result<(), CliError> {
    validate_session_name(name)?;
    let state = load_sessions_state()?;

    if !state.sessions.contains_key(name) {
        return Err(CliError::new(format!("session '{name}' does not exist")));
    }

    save_appointed_session(name)?;
    println!("Using session: {name}");
    print_jump_target(&state, name, no_jump)?;
    Ok(())
}

fn choose_session(no_jump: bool) -> Result<(), CliError> {
    let state = load_sessions_state()?;
    let names: Vec<&String> = state.sessions.keys().collect();

    if names.is_empty() {
        println!("No sessions. Create one with: b session new <name>");
        return Ok(());
    }

    let index = prompt_session_index(&names)?;
    let name = names
        .get(index)
        .ok_or_else(|| CliError::new(format!("session index {index} does not exist")))?;
    save_appointed_session(name)?;
    println!("Using session: {name}");
    print_jump_target(&state, name, no_jump)?;
    Ok(())
}

fn unset_session() -> Result<(), CliError> {
    unset_appointed_session()?;
    println!("Session unset");
    Ok(())
}

fn print_jump_target(state: &SessionsState, name: &str, no_jump: bool) -> Result<(), CliError> {
    if no_jump {
        return Ok(());
    }

    let bookmarks = appointed_bookmarks(state, name)?;
    if let Some(bookmark) = bookmarks.bookmarks.first() {
        println!("Jumping to [0] {bookmark}");
    } else {
        println!("No bookmark at [0] to jump to. Add one with: b add");
    }

    Ok(())
}

fn prompt_session_index(names: &[&String]) -> Result<usize, CliError> {
    for (index, name) in names.iter().enumerate() {
        println!("[{index}] {name}");
    }

    print!("Session index: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    parse_index(Some(input.trim()))
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

fn init(_shell: Shell) -> Result<(), CliError> {
    println!("{}", shell_function());
    Ok(())
}

fn shell_function() -> &'static str {
    r#"if [ -z "${BMERSIVE_STATE_DIR:-}" ]; then
  export BMERSIVE_STATE_DIR="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}/bmersive-${USER:-unknown}-$$"
fi

b() {
  if [ "$#" -eq 0 ]; then
    if ! bmersive session current >/dev/null 2>&1; then
      bmersive session choose || return
      _bmersive_target="$(bmersive path 0 2>/dev/null)" && cd "$_bmersive_target"
      unset _bmersive_target
      return
    fi
    bmersive ls
    return
  fi

  case "$1" in
    session)
      _bmersive_session_command="${2:-}"
      shift
      bmersive session "$@" || return
      case "$_bmersive_session_command" in
        use|choose)
          case " $* " in
            *" --no-jump "*) ;;
            *)
              _bmersive_target="$(bmersive path 0 2>/dev/null)" && cd "$_bmersive_target"
              unset _bmersive_target
              ;;
          esac
          ;;
      esac
      unset _bmersive_session_command
      ;;
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

fn tmux(mode_arg: Option<TmuxMode>) -> Result<(), CliError> {
    let mode = mode_arg
        .map(TmuxMode::as_str)
        .unwrap_or(default_tmux_mode());
    let session_name = load_appointed_session()?;
    let sessions = load_sessions_state()?;
    let state = appointed_bookmarks(&sessions, &session_name)?;

    if state.bookmarks.is_empty() {
        println!("No bookmarks to materialize. Add one with: b add");
        return Ok(());
    }

    let inside_tmux = env::var_os("TMUX").is_some();

    match (mode, inside_tmux) {
        ("windows", true) => {
            run_tmux_actions(tmux_windows_actions(&session_name, &state.bookmarks, true))
        }
        ("panes", true) => {
            run_tmux_actions(tmux_panes_actions(&session_name, &state.bookmarks, true))
        }
        ("windows", false) => tmux_outside_windows(&session_name, &state.bookmarks),
        ("panes", false) => tmux_outside_panes(&session_name, &state.bookmarks),
        (other, _) => Err(CliError::new(format!("unsupported tmux mode: {other}"))),
    }
}

fn default_tmux_mode() -> &'static str {
    "panes"
}

fn tmux_windows_actions(
    session_name: &str,
    bookmarks: &[String],
    inside_tmux: bool,
) -> Vec<TmuxAction> {
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

    let tmux_session = tmux_session_name(session_name);
    let mut actions = vec![TmuxAction::HasSession {
        session: tmux_session.clone(),
    }];

    if let Some(first) = bookmarks.first() {
        actions.push(TmuxAction::NewSessionDetached {
            session: tmux_session.clone(),
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
        session: tmux_session,
    });
    actions
}

fn tmux_panes_actions(
    session_name: &str,
    bookmarks: &[String],
    inside_tmux: bool,
) -> Vec<TmuxAction> {
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
        let tmux_session = tmux_session_name(session_name);
        actions.push(TmuxAction::HasSession {
            session: tmux_session.clone(),
        });
        actions.push(TmuxAction::NewSessionDetached {
            session: tmux_session.clone(),
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
            session: tmux_session,
        });
    }

    actions
}

fn tmux_outside_windows(session_name: &str, bookmarks: &[String]) -> Result<(), CliError> {
    let tmux_session = tmux_session_name(session_name);
    if tmux_session_exists(&tmux_session)? {
        return run_tmux(["attach-session", "-t", &tmux_session]);
    }

    run_tmux_dynamic(tmux_outside_windows_args(session_name, bookmarks))
}

fn tmux_outside_panes(session_name: &str, bookmarks: &[String]) -> Result<(), CliError> {
    let tmux_session = tmux_session_name(session_name);
    if tmux_session_exists(&tmux_session)? {
        return run_tmux(["attach-session", "-t", &tmux_session]);
    }

    run_tmux_dynamic(tmux_outside_panes_args(session_name, bookmarks))
}

fn tmux_outside_windows_args(session_name: &str, bookmarks: &[String]) -> Vec<String> {
    let mut args = Vec::new();
    let tmux_session = tmux_session_name(session_name);

    if let Some(first) = bookmarks.first() {
        args.extend(strings([
            "new-session",
            "-d",
            "-s",
            &tmux_session,
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
        args.extend(strings(["attach-session", "-t", &tmux_session]));
    }

    args
}

fn tmux_outside_panes_args(session_name: &str, bookmarks: &[String]) -> Vec<String> {
    let mut args = Vec::new();
    let tmux_session = tmux_session_name(session_name);

    if let Some(first) = bookmarks.first() {
        args.extend(strings([
            "new-session",
            "-d",
            "-s",
            &tmux_session,
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
        args.extend(strings(["attach-session", "-t", &tmux_session]));
    }

    args
}

fn tmux_session_name(session_name: &str) -> String {
    format!("{SESSION_NAME}-{session_name}")
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
    fn sessions_state_path_prefers_xdg_state_home() {
        assert_eq!(
            sessions_state_path_from(
                Some(OsString::from("/state")),
                Some(OsString::from("/home/me")),
                Some("me".to_string())
            ),
            PathBuf::from("/state/bmersive/sessions.json")
        );
    }

    #[test]
    fn sessions_state_path_uses_home_local_state() {
        assert_eq!(
            sessions_state_path_from(
                None,
                Some(OsString::from("/home/me")),
                Some("me".to_string())
            ),
            PathBuf::from("/home/me/.local/state/bmersive/sessions.json")
        );
    }

    #[test]
    fn sessions_state_path_falls_back_to_tmp_user_dir() {
        assert_eq!(
            sessions_state_path_from(None, None, Some("testuser".to_string())),
            PathBuf::from("/tmp/bmersive-testuser/sessions.json")
        );
    }

    #[test]
    fn appointment_path_requires_state_dir() {
        assert_eq!(
            appointment_path_from(Some(OsString::from("/tmp/bmersive-shell")))
                .expect("appointment path"),
            PathBuf::from("/tmp/bmersive-shell/session")
        );
        assert!(appointment_path_from(None).is_err());
    }

    #[test]
    fn load_sessions_imports_legacy_bookmarks_when_new_state_missing() {
        let root = temp_test_dir("legacy-import");
        let sessions_path = root.join("sessions.json");
        let legacy_path = root.join("bookmarks.json");
        save_state(
            &legacy_path,
            &BookmarkState {
                bookmarks: vec!["/work/api".to_string()],
            },
        )
        .expect("save legacy state");

        let state = load_sessions_state_from(&sessions_path, &legacy_path).expect("load sessions");
        assert_eq!(
            state
                .sessions
                .get(DEFAULT_IMPORTED_SESSION)
                .expect("default session")
                .bookmarks,
            vec!["/work/api".to_string()]
        );
    }

    #[test]
    fn load_sessions_prefers_new_state_over_legacy() {
        let root = temp_test_dir("new-state-precedence");
        let sessions_path = root.join("sessions.json");
        let legacy_path = root.join("bookmarks.json");
        save_state(
            &legacy_path,
            &BookmarkState {
                bookmarks: vec!["/work/legacy".to_string()],
            },
        )
        .expect("save legacy state");

        let mut sessions = SessionsState::default();
        sessions.sessions.insert(
            "api".to_string(),
            BookmarkState {
                bookmarks: vec!["/work/api".to_string()],
            },
        );
        save_sessions_state_to(&sessions_path, &sessions).expect("save sessions state");

        let state = load_sessions_state_from(&sessions_path, &legacy_path).expect("load sessions");
        assert!(state.sessions.contains_key("api"));
        assert!(!state.sessions.contains_key(DEFAULT_IMPORTED_SESSION));
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
    fn validates_session_names() {
        assert!(validate_session_name("api").is_ok());
        assert!(validate_session_name("api-web2").is_ok());
        assert!(validate_session_name("Api").is_err());
        assert!(validate_session_name("api_web").is_err());
        assert!(validate_session_name("").is_err());
    }

    #[test]
    fn parser_accepts_session_commands() {
        assert!(Cli::try_parse_from(["bmersive", "session", "ls"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "new", "api"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "use", "api"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "use", "api", "--no-jump"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "choose"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "choose", "--no-jump"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "unset"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "current"]).is_ok());
        assert!(Cli::try_parse_from(["bmersive", "session", "new"]).is_err());
    }

    #[test]
    fn unset_appointed_session_removes_marker_if_present() {
        let root = temp_test_dir("unset-session");
        let path = root.join("session");
        save_appointed_session_to(&path, "api").expect("save appointment");
        unset_appointed_session_from(&path).expect("unset appointment");
        assert!(!path.exists());
        unset_appointed_session_from(&path).expect("unset missing appointment");
    }

    #[test]
    fn jump_target_output_uses_first_bookmark_unless_disabled() {
        let mut sessions = SessionsState::default();
        sessions.sessions.insert(
            "api".to_string(),
            BookmarkState {
                bookmarks: vec!["/work/api".to_string()],
            },
        );

        assert!(print_jump_target(&sessions, "api", false).is_ok());
        assert!(print_jump_target(&sessions, "api", true).is_ok());
    }

    #[test]
    fn shell_function_chooses_and_delegates_sessions() {
        let shell = shell_function();
        assert!(shell.contains("if ! bmersive session current >/dev/null 2>&1"));
        assert!(shell.contains("bmersive session choose || return"));
        assert!(shell.contains("bmersive session \"$@\""));
        assert!(shell.contains("--no-jump"));
        assert!(shell.contains(
            "_bmersive_target=\"$(bmersive path 0 2>/dev/null)\" && cd \"$_bmersive_target\""
        ));
        assert!(shell.contains("bmersive add \"$@\""));
        assert!(shell.contains("cd \"$(bmersive path \"$1\")\""));
    }

    #[test]
    fn tmux_windows_inside_current_session_creates_windows() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_windows_actions("api", &bookmarks, true),
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
            tmux_windows_actions("api", &bookmarks, false),
            vec![
                TmuxAction::HasSession {
                    session: "bmersive-api".to_string()
                },
                TmuxAction::NewSessionDetached {
                    session: "bmersive-api".to_string(),
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::NewWindow {
                    target: None,
                    cwd: "/work/web".to_string(),
                    name: "web".to_string()
                },
                TmuxAction::AttachSession {
                    session: "bmersive-api".to_string()
                }
            ]
        );
    }

    #[test]
    fn tmux_panes_inside_splits_additional_bookmarks_and_tiles() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_panes_actions("api", &bookmarks, true),
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
            tmux_panes_actions("api", &bookmarks, false),
            vec![
                TmuxAction::HasSession {
                    session: "bmersive-api".to_string()
                },
                TmuxAction::NewSessionDetached {
                    session: "bmersive-api".to_string(),
                    cwd: "/work/api".to_string(),
                    name: "api".to_string()
                },
                TmuxAction::SplitWindow {
                    target: None,
                    cwd: "/work/web".to_string()
                },
                TmuxAction::SelectLayoutTiled { target: None },
                TmuxAction::AttachSession {
                    session: "bmersive-api".to_string()
                }
            ]
        );
    }

    #[test]
    fn tmux_outside_windows_uses_single_command_sequence() {
        let bookmarks = vec!["/work/api".to_string(), "/work/web".to_string()];
        assert_eq!(
            tmux_outside_windows_args("api", &bookmarks),
            vec![
                "new-session",
                "-d",
                "-s",
                "bmersive-api",
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
                "bmersive-api"
            ]
        );
    }

    #[test]
    fn tmux_outside_panes_uses_single_command_sequence_for_four_bookmarks() {
        let bookmarks = vec![
            "/work/api".to_string(),
            "/work/web".to_string(),
            "/work/worker".to_string(),
            "/work/infra".to_string(),
        ];
        assert_eq!(
            tmux_outside_panes_args("api", &bookmarks),
            vec![
                "new-session",
                "-d",
                "-s",
                "bmersive-api",
                "-c",
                "/work/api",
                "-n",
                "api",
                ";",
                "split-window",
                "-c",
                "/work/web",
                ";",
                "split-window",
                "-c",
                "/work/worker",
                ";",
                "split-window",
                "-c",
                "/work/infra",
                ";",
                "select-layout",
                "tiled",
                ";",
                "attach-session",
                "-t",
                "bmersive-api"
            ]
        );
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = env::temp_dir().join(format!("bmersive-test-{name}-{nanos}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }
}
