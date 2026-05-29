use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bmersive")
}

fn temp_runtime(name: &str) -> PathBuf {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("bmersive-test-{name}-{id}"))
}

fn run(runtime: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .env("XDG_RUNTIME_DIR", runtime)
        .env_remove("BMERSIVE_MAX_BOOKMARKS")
        .output()
        .expect("run bmersive test command")
}

#[test]
fn add_list_path_and_remove_bookmarks() {
    let runtime = temp_runtime("crud");
    let first = runtime.join("one");
    let second = runtime.join("two");
    fs::create_dir_all(&first).expect("create first test bookmark directory");
    fs::create_dir_all(&second).expect("create second test bookmark directory");

    assert!(run(
        &runtime,
        &["add", first.to_str().expect("first path is UTF-8")]
    )
    .status
    .success());
    assert!(run(
        &runtime,
        &["add", second.to_str().expect("second path is UTF-8")]
    )
    .status
    .success());

    let list = run(&runtime, &["ls"]);
    assert!(list.status.success());
    let stdout = String::from_utf8(list.stdout).expect("list output is UTF-8");
    assert!(stdout.contains("[0]"));
    assert!(stdout.contains(first.to_str().expect("first path is UTF-8")));
    assert!(stdout.contains("[1]"));
    assert!(stdout.contains(second.to_str().expect("second path is UTF-8")));

    let path = run(&runtime, &["path", "1"]);
    assert!(path.status.success());
    assert_eq!(
        String::from_utf8(path.stdout)
            .expect("path output is UTF-8")
            .trim(),
        second.to_str().expect("second path is UTF-8")
    );

    assert!(run(&runtime, &["rm", "0"]).status.success());
    let path = run(&runtime, &["path", "0"]);
    assert!(path.status.success());
    assert_eq!(
        String::from_utf8(path.stdout)
            .expect("path output is UTF-8")
            .trim(),
        second.to_str().expect("second path is UTF-8")
    );
}

#[test]
fn duplicate_add_does_not_create_second_entry() {
    let runtime = temp_runtime("duplicate");
    let project = runtime.join("project");
    fs::create_dir_all(&project).expect("create duplicate test bookmark directory");

    assert!(run(
        &runtime,
        &["add", project.to_str().expect("project path is UTF-8")]
    )
    .status
    .success());
    let project_dot = project.join(".");
    assert!(run(
        &runtime,
        &[
            "add",
            project_dot.to_str().expect("project dot path is UTF-8")
        ]
    )
    .status
    .success());

    let list = run(&runtime, &["ls"]);
    let stdout = String::from_utf8(list.stdout).expect("list output is UTF-8");
    assert_eq!(
        stdout.lines().filter(|line| line.starts_with('[')).count(),
        1
    );
}

#[test]
fn configured_max_rejects_full_list() {
    let runtime = temp_runtime("max");
    let first = runtime.join("one");
    let second = runtime.join("two");
    fs::create_dir_all(&first).expect("create first max test directory");
    fs::create_dir_all(&second).expect("create second max test directory");

    let add_first = Command::new(binary())
        .args(["add", first.to_str().expect("first path is UTF-8")])
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("BMERSIVE_MAX_BOOKMARKS", "1")
        .output()
        .expect("run first max add command");
    assert!(add_first.status.success());

    let add_second = Command::new(binary())
        .args(["add", second.to_str().expect("second path is UTF-8")])
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("BMERSIVE_MAX_BOOKMARKS", "1")
        .output()
        .expect("run second max add command");
    assert!(!add_second.status.success());
    assert!(String::from_utf8(add_second.stderr)
        .expect("stderr is UTF-8")
        .contains("bookmark list is full"));
}

#[test]
fn init_outputs_shell_wrapper() {
    let zsh = Command::new(binary())
        .args(["init", "zsh"])
        .output()
        .expect("run zsh init command");
    assert!(zsh.status.success());
    let stdout = String::from_utf8(zsh.stdout).expect("zsh init output is UTF-8");
    assert!(stdout.contains("b()"));
    assert!(stdout.contains("bmersive ls"));
    assert!(stdout.contains("cd \"$(bmersive path \"$1\")\""));

    let bash = Command::new(binary())
        .args(["init", "bash"])
        .output()
        .expect("run bash init command");
    assert!(bash.status.success());
    assert!(String::from_utf8(bash.stdout)
        .expect("bash init output is UTF-8")
        .contains("b()"));
}
