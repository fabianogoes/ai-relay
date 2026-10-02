use std::fs;
use std::path::{Path, PathBuf};

use relay_tui::workspace::{read_workspace, resolve_workspace};
use tempfile::TempDir;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// An absolute path on the platform running the tests.
fn abs(path: &str) -> PathBuf {
    let root = if cfg!(windows) { "C:\\" } else { "/" };
    PathBuf::from(root).join(path)
}

#[test]
fn an_explicit_absolute_path_is_kept() {
    let resolved = resolve_workspace(Some(&abs("work/repo")), &abs("somewhere/else"));
    assert_eq!(resolved, abs("work/repo"));
}

#[test]
fn a_relative_path_resolves_against_the_current_directory() {
    let cwd = abs("home/me/projects");
    assert_eq!(resolve_workspace(Some(Path::new("repo")), &cwd), abs("home/me/projects/repo"));
    // `.` and `..` collapse lexically, like node's path.resolve.
    assert_eq!(
        resolve_workspace(Some(Path::new("../other/./repo")), &cwd),
        abs("home/me/other/repo")
    );
}

#[test]
fn without_an_argument_the_current_directory_is_observed() {
    let cwd = abs("home/me/projects/repo");
    assert_eq!(resolve_workspace(None, &cwd), cwd);
}

#[test]
fn the_four_records_and_the_specs_are_read() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write(root, ".orchestration/BACKLOG.md", "# Backlog\n");
    write(root, ".orchestration/TODO.md", "# Active task\n");
    write(root, ".orchestration/HANDOFF.md", "# Handoff\n");
    write(root, ".orchestration/CHANGELOG.md", "# Change log\n");
    write(root, ".specs/20260101-001-a.md", "# a\n");
    write(root, ".specs/20260101-002-b.md", "# b\n");
    write(root, ".specs/notes.txt", "not a spec");

    let files = read_workspace(root);
    assert_eq!(files.backlog, "# Backlog\n");
    assert_eq!(files.todo, "# Active task\n");
    assert_eq!(files.handoff, "# Handoff\n");
    assert_eq!(files.changelog, "# Change log\n");
    let keys: Vec<&str> = files.specs.keys().map(String::as_str).collect();
    assert_eq!(keys, [".specs/20260101-001-a.md", ".specs/20260101-002-b.md"]);
    assert_eq!(files.specs[".specs/20260101-002-b.md"], "# b\n");
}

#[test]
fn a_missing_record_reads_as_empty_text() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), ".orchestration/TODO.md", "# Active task\n");
    let files = read_workspace(dir.path());
    assert_eq!(files.todo, "# Active task\n");
    assert_eq!((files.backlog.as_str(), files.handoff.as_str(), files.changelog.as_str()), ("", "", ""));
    assert!(files.specs.is_empty());
}

#[test]
fn a_directory_that_does_not_exist_reads_as_an_empty_workspace() {
    let files = read_workspace(Path::new("/definitely/not/a/relay/workspace"));
    assert_eq!(files, Default::default());
}

#[test]
fn a_record_that_is_not_valid_utf8_reads_as_empty_text() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(".orchestration/TODO.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(read_workspace(dir.path()).todo, "");
}
