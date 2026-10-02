//! Watches a workspace's record directories and turns file-system events into
//! `Dirty` / `Settled` signals.
//!
//! A port of `app/relay-host/src/watcher.ts`: `.orchestration/` and `.specs/`
//! are watched, not the whole repository, so a build writing thousands of
//! files elsewhere never wakes it.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::debounce::{WorkspaceEvent, debounce};

pub struct WorkspaceWatcher {
    events: Receiver<WorkspaceEvent>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkspaceWatcher {
    /// `Dirty` when files start changing, `Settled` once they stop.
    pub fn events(&self) -> &Receiver<WorkspaceEvent> {
        &self.events
    }
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Reading a record opens and closes it, which some back ends report as an
/// `Access` event. Reacting to those would make every re-read wake the watcher
/// again, forever.
fn is_content_change(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

/// The record directories, attached as they exist: one missing at start-up (a
/// workspace not set up yet) is attached once it appears.
struct Dirs {
    paths: [PathBuf; 2],
    attached: [bool; 2],
}

impl Dirs {
    fn new(workspace: &Path) -> Self {
        Dirs {
            paths: [workspace.join(".orchestration"), workspace.join(".specs")],
            attached: [false; 2],
        }
    }

    /// Attaches every directory that exists and is not attached yet; whether
    /// any was newly attached.
    fn attach_missing(&mut self, watcher: &mut impl Watcher) -> bool {
        let mut any = false;
        for (path, attached) in self.paths.iter().zip(self.attached.iter_mut()) {
            if !*attached && path.is_dir() && watcher.watch(path, RecursiveMode::NonRecursive).is_ok() {
                *attached = true;
                any = true;
            }
        }
        any
    }
}

/// Starts watching `workspace`. Dropping the returned watcher stops it.
pub fn watch_workspace(workspace: &Path, quiescence: Duration) -> notify::Result<WorkspaceWatcher> {
    let (raw_tx, raw_rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if event.is_ok_and(|e| is_content_change(&e.kind)) {
            let _ = raw_tx.send(());
        }
    })?;
    let mut dirs = Dirs::new(workspace);
    dirs.attach_missing(&mut watcher);

    let (out_tx, events) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let thread = thread::spawn(move || {
        debounce(raw_rx, out_tx, quiescence, &flag, || dirs.attach_missing(&mut watcher));
    });
    Ok(WorkspaceWatcher { events, stop, thread: Some(thread) })
}
