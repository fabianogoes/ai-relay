//! The only place that touches the disk: resolve which workspace to observe,
//! read its records, and (see `watch`) tell when they change.
//!
//! Reading never fails: whatever cannot be read is empty text, and
//! the core then derives what that implies.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::core::RelayFiles;
use crate::language::{self, Language};

/// The absolute path to observe: an explicit `arg` resolved against `cwd`, or
/// `cwd` itself. `.` and `..` collapse lexically, without touching the disk or
/// following symlinks.
pub fn resolve_workspace(arg: Option<&Path>, cwd: &Path) -> PathBuf {
    let joined = match arg {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd.to_path_buf(),
    };
    let mut resolved = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    resolved
}

/// Whether `workspace` has a `.orchestration/` directory: what makes a
/// directory a Relay workspace at all.
pub fn is_relay_workspace(workspace: &Path) -> bool {
    workspace.join(".orchestration").is_dir()
}

fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Read the optional workspace language setting. It is configuration, not a
/// Relay protocol record, so it stays outside `RelayFiles` and core parsing.
pub fn read_language(workspace: &Path) -> Option<Language> {
    fs::read_to_string(workspace.join(".orchestration/SETTINGS.md"))
        .ok()
        .and_then(|contents| language::parse_settings(&contents))
}

/// Every `*.md` file of `dir`, keyed `<prefix>/<file name>`.
fn read_markdown_dir(dir: &Path, prefix: &str) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    if let Ok(dir) = fs::read_dir(dir) {
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".md") {
                files.insert(format!("{prefix}/{name}"), read_or_empty(&entry.path()));
            }
        }
    }
    files
}

/// The four records of `.orchestration/`, the per-spec changelogs of
/// `.orchestration/changelog/*.md` (keyed `changelog/<file name>`) and every
/// `.specs/*.md`, keyed `.specs/<file name>`. A missing record, or one that is
/// not valid UTF-8, reads as empty text.
pub fn read_workspace(workspace: &Path) -> RelayFiles {
    let orchestration = workspace.join(".orchestration");
    let specs = read_markdown_dir(&workspace.join(".specs"), ".specs");
    let changelogs = read_markdown_dir(&orchestration.join("changelog"), "changelog");
    RelayFiles {
        backlog: read_or_empty(&orchestration.join("BACKLOG.md")),
        todo: read_or_empty(&orchestration.join("TODO.md")),
        handoff: read_or_empty(&orchestration.join("HANDOFF.md")),
        changelog: read_or_empty(&orchestration.join("CHANGELOG.md")),
        changelogs,
        specs,
    }
}

mod debounce;
mod watch;

pub use debounce::{QUIESCENCE, WorkspaceEvent};
pub use watch::{WorkspaceWatcher, watch_workspace};

#[cfg(test)]
mod language_tests {
    use super::*;

    #[test]
    fn workspace_language_is_read_separately_from_protocol_records() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".orchestration")).unwrap();
        std::fs::write(
            dir.path().join(".orchestration/SETTINGS.md"),
            "# Settings\n\n- Language: pt-BR\n",
        )
        .unwrap();
        assert_eq!(read_language(dir.path()), Some(Language::PtBr));
        assert_eq!(read_workspace(dir.path()), RelayFiles::default());
    }
}
