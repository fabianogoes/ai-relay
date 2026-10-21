//! The watcher against a real directory. The operating system decides when
//! events arrive. These checks cover delivery and re-reading; the deterministic
//! debounce tests cover burst grouping and the quiescence window.

use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use WorkspaceEvent::{Dirty, Settled};
use relay_tui::workspace::{
    QUIESCENCE, WorkspaceEvent, WorkspaceWatcher, read_workspace, watch_workspace,
};
use tempfile::TempDir;

const LONG: Duration = Duration::from_secs(5);
// FSEvents is shared host state on macOS. Starting several streams at once in
// this test binary makes delivery flaky, so exercise one real watcher at a time.
static WATCH_TEST_LOCK: Mutex<()> = Mutex::new(());

fn serial_watcher_test() -> std::sync::MutexGuard<'static, ()> {
    WATCH_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

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
    while watcher
        .events()
        .recv_timeout(Duration::from_millis(600))
        .is_ok()
    {}
}

#[test]
fn a_real_write_is_noticed_and_the_settled_workspace_has_its_content() {
    let _serial = serial_watcher_test();
    let dir = workspace_with_records();
    let watcher = started(dir.path());

    quiet(&watcher);
    fs::write(
        dir.path().join(".orchestration/TODO.md"),
        "# Active task: B-4\n",
    )
    .unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
    assert_eq!(read_workspace(dir.path()).todo, "# Active task: B-4\n");
}

#[test]
fn a_write_in_the_specs_directory_is_noticed_too() {
    let _serial = serial_watcher_test();
    let dir = workspace_with_records();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::write(dir.path().join(".specs/20261002-001-x.md"), "# x\n").unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn a_write_in_the_changelog_directory_is_noticed_too() {
    let _serial = serial_watcher_test();
    let dir = workspace_with_records();
    fs::create_dir_all(dir.path().join(".orchestration/changelog")).unwrap();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::write(
        dir.path().join(".orchestration/changelog/20261002-001.md"),
        "# c\n",
    )
    .unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn a_changelog_directory_created_after_start_up_is_picked_up() {
    let _serial = serial_watcher_test();
    let dir = workspace_with_records();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::create_dir_all(dir.path().join(".orchestration/changelog")).unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));

    quiet(&watcher);
    fs::write(
        dir.path().join(".orchestration/changelog/20261002-001.md"),
        "# c\n",
    )
    .unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn reading_the_workspace_does_not_wake_the_watcher() {
    let _serial = serial_watcher_test();
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
    let _serial = serial_watcher_test();
    let dir = TempDir::new().unwrap();
    let watcher = started(dir.path());
    quiet(&watcher);

    fs::create_dir_all(dir.path().join(".orchestration")).unwrap();
    fs::write(dir.path().join(".orchestration/TODO.md"), "# Active task\n").unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));

    // From then on its writes are seen as in any other workspace.
    quiet(&watcher);
    fs::write(
        dir.path().join(".orchestration/TODO.md"),
        "# Active task: B-1\n",
    )
    .unwrap();
    assert_eq!(next(&watcher, LONG), Some(Dirty));
    assert_eq!(next(&watcher, LONG), Some(Settled));
}

#[test]
fn a_workspace_that_does_not_exist_is_not_an_error() {
    let _serial = serial_watcher_test();
    let watcher =
        watch_workspace(Path::new("/definitely/not/a/relay/workspace"), QUIESCENCE).unwrap();
    assert_eq!(next(&watcher, Duration::from_millis(500)), None);
}
