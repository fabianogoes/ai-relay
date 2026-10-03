//! `relay-tui` observes; it never writes and never launches anything. The
//! end-to-end test shows a workspace unchanged after a run; this one shows the
//! code has no way to change it.

use std::fs;
use std::path::{Path, PathBuf};

// Anything that writes to the disk or starts a process.
const FORBIDDEN: [&str; 17] = [
    "fs::write",
    "File::create",
    "OpenOptions",
    "fs::remove_file",
    "fs::remove_dir",
    "fs::create_dir",
    "fs::rename",
    "fs::copy",
    "fs::hard_link",
    "fs::set_permissions",
    "fs::soft_link",
    "symlink",
    "process::Command",
    "Command::new",
    "std::os::unix::process",
    "libc::",
    "unsafe",
];

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|x| x == "rs") {
            found.push(path);
        }
    }
}

/// The code of a file: comments and `#[cfg(test)]` modules are not shipped,
/// but a comment mentioning an API is not a use of it either.
fn code(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_shipped_code_cannot_write_to_disk_or_launch_a_process() {
    let mut files = Vec::new();
    sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    assert!(files.len() >= 10, "src/ is missing sources: {files:?}");
    for path in files {
        // The unit tests of a module set up temporary workspaces, which is
        // writing; only what comes before the test module ships.
        let text = fs::read_to_string(&path).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap();
        let code = code(shipped);
        for needle in FORBIDDEN {
            assert!(
                !code.contains(needle),
                "{} uses `{needle}`: relay-tui must only read",
                path.display()
            );
        }
    }
}

#[test]
fn the_only_thing_that_reads_the_disk_is_the_workspace_module() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in ["core", "view", "nav.rs", "suggest.rs", "theme.rs"] {
        let path = src.join(file);
        let mut files = Vec::new();
        if path.is_dir() {
            sources(&path, &mut files);
        } else {
            files.push(path);
        }
        for path in files {
            let text = fs::read_to_string(&path).unwrap();
            let code = code(text.split("#[cfg(test)]").next().unwrap());
            for needle in ["std::fs", "fs::", "std::path::Path::exists", ".is_dir()"] {
                assert!(!code.contains(needle), "{} reads the disk with `{needle}`", path.display());
            }
        }
    }
}
