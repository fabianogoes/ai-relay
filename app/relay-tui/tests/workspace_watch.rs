//! The watcher against a real directory. Margins are generous: the operating
//! system decides when an event arrives, and what is asserted is the order and
//! the count of signals, not exact timing.

use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use relay_tui::workspace::{QUIESCENCE, WorkspaceEvent, WorkspaceWatcher, read_workspace, watch_workspace};
use tempfile::TempDir;
use WorkspaceEvent::{Dirty, Settled};

const LONG: Duration = Duration::from_secs(5);

fn workspace_with_records() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(".orchestration")).unwrap();
    fs::create_dir_all(dir.path().join(".specs")).unwrap();
    fs::write(dir.path().join(".orchestration/TODO.md"), "# Active task\n").unwrap();
    dir
}

// The backend (FSEvents on macOS) takes a moment to start delivering.
fn started(root: &Path) -> WorkspaceWatcher {
    let watcher = watch_workspace(root, QUIESCENCE).unwrap();
    thread::sleep(Duration::from_millis(300));
    watcher
}

fn next(watcher: &WorkspaceWatcher, within: Duration) -> Option<WorkspaceEvent> {
    watcher.events().recv_timeout(within).ok()
}

// Drains whatever the start-up itself produced, so a test sees only its own.
fn quiet(watcher: &WorkspaceWatcher) {
    while watcher.events().recv_timeout(Duration::from_millis(600)).is_ok() {}
}

/// Five writes 20 ms apart. Returns when the last one finished and the longest
/// gap between two of them: on a loaded machine a write can be descheduled for
/// longer than the quiescence, and then it is no longer one burst.
fn burst(dir: &Path) -> (Instant, Duration) {
    let mut last_write = Instant::now();
    let mut longest = Duration::ZERO;
    for i in 0..5 {
        let before = Instant::now();
        fs::write(dir.join(".orchestration/TODO.md"), format!("# Active task: B-{i}\n")).unwrap();
        // The reference is the last write, not the end of the sleep after it.
        let now = Instant::now();
        if i > 0 {
            longest = longest.max(before.duration_since(last_write));
        }
        last_write = now;
        thread::sleep(Duration::from_millis(20));
    }
    (last_write, longest)
}

#[test]
fn a_burst_of_real_writes_yields_one_dirty_and_one_settled() {
    let dir = workspace_with_records();
    let watcher = started(dir.path());

    // A stall of the machine in the middle of the writes (a shared CI runner)
    // splits the burst in two, which says nothing about the watcher: try again
    // with a burst that really was one.
    let mut checked = false;
    for _ in 0..6 {
        quiet(&watcher);
        let (last_write, longest_gap) = burst(dir.path());
        if longest_gap > QUIESCENCE / 2 {
            continue;
        }
        assert_eq!(next(&watcher, LONG), Some(Dirty));
        assert_eq!(next(&watcher, LONG), Some(Settled));
        assert!(last_write.elapsed() >= QUIESCENCE - Duration::from_millis(30));
        // The burst is one snapshot: nothing else follows it.
        assert_eq!(next(&watcher, Duration::from_millis(700)), None);
        // And the settled snapshot has the last write.
        assert_eq!(read_workspace(dir.path()).todo, "# Active task: B-4\n");
        checked = true;
        break;
    }
    assert!(checked, "the machine was too busy to write five files in a burst, six times");
}

#[test]
fn a_write_in_the_specs_directory_is_noticed_too() {
    let dir = workspace_with_records();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::write(dir.path().join(".specs/20261002-001-x.md"), "# x\n").unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn reading_the_workspace_does_not_wake_the_watcher() {
    let dir = workspace_with_records();
    let watcher = started(dir.path());
    quiet(&watcher);

    for _ in 0..20 {
        read_workspace(dir.path());
    }
    // A watcher that reacted to its own reads would loop forever.
    assert_eq!(next(&watcher, Duration::from_millis(800)), None);
}

#[test]
fn a_directory_that_appears_after_start_up_is_picked_up() {
    let dir = TempDir::new().unwrap();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::create_dir_all(dir.path().join(".orchestration")).unwrap();
    fs::write(dir.path().join(".orchestration/TODO.md"), "# Active task\n").unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));

    // From then on its writes are seen as in any other workspace.
    quiet(&watcher);
    fs::write(dir.path().join(".orchestration/TODO.md"), "# Active task: B-1\n").unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn a_workspace_that_does_not_exist_is_not_an_error() {
    let watcher = watch_workspace(Path::new("/definitely/not/a/relay/workspace"), QUIESCENCE).unwrap();
    assert_eq!(next(&watcher, Duration::from_millis(500)), None);
}

#[test]
fn dropping_the_watcher_stops_it_promptly() {
    let dir = workspace_with_records();
    let watcher = started(dir.path());
    let asked = Instant::now();
    drop(watcher);
    assert!(asked.elapsed() < Duration::from_secs(1));
}
