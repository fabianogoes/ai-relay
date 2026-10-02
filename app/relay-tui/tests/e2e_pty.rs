//! The real binary in a real pseudo-terminal, read through a terminal
//! emulator (vt100): what the user would see, and what the terminal is left
//! as. This is the evidence for "reflects a change without restarting" and
//! "restores the terminal on q, Esc, Ctrl-C and panic".

use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_relay-tui");
const PATIENCE: Duration = Duration::from_secs(8);

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A workspace on disk, copied from one of the conformance cases.
fn workspace(case: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance").join(case).join("workspace");
    copy_tree(&from, dir.path());
    dir
}

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    parser: Arc<Mutex<vt100::Parser>>,
    /// Every distinct screen the reader saw, so a state that lasts 150 ms is
    /// not missed by polling.
    history: Arc<Mutex<Vec<String>>>,
    reader: Option<JoinHandle<()>>,
}

impl Session {
    fn start(dir: &Path, rows: u16, cols: u16, env: &[(&str, &str)]) -> Session {
        let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
        let pair = native_pty_system().openpty(size).unwrap();
        let mut command = CommandBuilder::new(BIN);
        command.arg("--workspace");
        command.arg(dir);
        command.env("TERM", "xterm-256color");
        for (key, value) in env {
            command.env(key, value);
        }
        let child = pair.slave.spawn_command(command).unwrap();
        drop(pair.slave);

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let history = Arc::new(Mutex::new(Vec::new()));
        let mut stream = pair.master.try_clone_reader().unwrap();
        let (p, h) = (Arc::clone(&parser), Arc::clone(&history));
        let reader = thread::spawn(move || {
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stream.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                let mut parser = p.lock().unwrap();
                parser.process(&chunk[..n]);
                let screen = parser.screen().contents();
                let mut history = h.lock().unwrap();
                if history.last() != Some(&screen) {
                    history.push(screen);
                }
            }
        });
        Session {
            writer: pair.master.take_writer().unwrap(),
            master: pair.master,
            child,
            parser,
            history,
            reader: Some(reader),
        }
    }

    fn screen(&self) -> String {
        self.parser.lock().unwrap().screen().contents()
    }

    fn send(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
    }

    fn resize(&self, rows: u16, cols: u16) {
        self.parser.lock().unwrap().screen_mut().set_size(rows, cols);
        self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).unwrap();
    }

    fn wait_for(&self, what: &str, condition: impl Fn(&str) -> bool) {
        let started = Instant::now();
        while started.elapsed() < PATIENCE {
            if condition(&self.screen()) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out waiting for {what}; the screen was:\n{}", self.screen());
    }

    fn saw(&self, word: &str) -> bool {
        self.history.lock().unwrap().iter().any(|screen| screen.contains(word))
    }

    /// Waits for the process to exit and for all its output to be read.
    fn exit(&mut self) -> portable_pty::ExitStatus {
        let started = Instant::now();
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(started.elapsed() < PATIENCE, "the process did not exit; screen:\n{}", self.screen());
            thread::sleep(Duration::from_millis(10));
        };
        self.reader.take().unwrap().join().unwrap();
        status
    }

    /// What the terminal was left as: the normal screen, the cursor shown.
    fn assert_restored(&self) {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        assert!(!screen.alternate_screen(), "the alternate screen was not left");
        assert!(!screen.hide_cursor(), "the cursor was left hidden");
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn open_in_progress() -> (TempDir, Session) {
    let dir = workspace("status-in_progress");
    let session = Session::start(dir.path(), 24, 58, &[]);
    session.wait_for("the handoff card", |s| s.contains("Em andamento") && s.contains("atualizado"));
    (dir, session)
}

#[test]
fn it_opens_and_shows_the_workspace() {
    let (_dir, session) = open_in_progress();
    let screen = session.screen();
    assert!(screen.contains("relay"));
    assert!(screen.contains("Handoff") && screen.contains("B-001") && screen.contains("T-002"));
    assert!(screen.contains("Validar os fixtures contra o protocolo."));
    assert!(screen.contains("TODO") && screen.contains("Backlog") && screen.contains("q sair"));
}

#[test]
fn a_change_on_disk_shows_up_without_restarting() {
    let dir = workspace("status-idle");
    let session = Session::start(dir.path(), 24, 58, &[]);
    session.wait_for("the empty workspace", |s| s.contains("Sem trabalho") && s.contains("atualizado"));

    fs::write(
        dir.path().join(".orchestration/BACKLOG.md"),
        "# Backlog\n\n- [ ] B-001 - Algo para escolher (spec: .specs/20261002-001-x.md)\n",
    )
    .unwrap();

    session.wait_for("the new backlog", |s| s.contains("A escolher") && s.contains("Algo para escolher"));
    session.wait_for("a settled snapshot", |s| s.contains("atualizado") && !s.contains("atualizando"));
    // The change was in flight for a while, and the screen said so in words.
    assert!(session.saw("atualizando"), "never showed `atualizando`");
}

#[test]
fn a_burst_of_writes_ends_in_a_single_settled_screen() {
    let dir = workspace("status-idle");
    let session = Session::start(dir.path(), 24, 58, &[]);
    session.wait_for("the empty workspace", |s| s.contains("Sem trabalho"));

    for n in 1..=5 {
        fs::write(
            dir.path().join(".orchestration/BACKLOG.md"),
            format!("# Backlog\n\n- [ ] B-001 - Versao {n} (spec: .specs/20261002-001-x.md)\n"),
        )
        .unwrap();
        thread::sleep(Duration::from_millis(20));
    }

    session.wait_for("the last write", |s| s.contains("Versao 5") && s.contains("atualizado") && !s.contains("atualizando"));
    // The intermediate versions were never read: the burst is one snapshot.
    for n in 1..=4 {
        assert!(!session.saw(&format!("Versao {n}")), "Versao {n} was drawn");
    }
}

#[test]
fn a_directory_without_orchestration_says_so_and_keeps_watching() {
    let dir = TempDir::new().unwrap();
    let session = Session::start(dir.path(), 24, 58, &[]);
    session.wait_for("the not-a-workspace card", |s| s.contains("Não é um workspace Relay"));

    fs::create_dir_all(dir.path().join(".orchestration")).unwrap();
    fs::write(dir.path().join(".orchestration/BACKLOG.md"), "# Backlog\n\n- [ ] B-001 - Chegou (spec: .specs/x.md)\n").unwrap();
    session.wait_for("the workspace that appeared", |s| s.contains("A escolher") && s.contains("Chegou"));
}

fn width_of_card_top(screen: &str, title: &str) -> Option<usize> {
    screen
        .lines()
        .find(|line| line.starts_with(&format!("╭ {title}")))
        .map(|line| line.chars().count())
}

#[test]
fn it_redraws_when_the_terminal_is_resized() {
    let (_dir, session) = open_in_progress();
    assert_eq!(width_of_card_top(&session.screen(), "Handoff"), Some(58));

    session.resize(24, 40);
    session.wait_for("the narrow layout", |s| width_of_card_top(s, "Handoff") == Some(40));
    session.resize(24, 80);
    session.wait_for("the wide layout", |s| width_of_card_top(s, "Handoff") == Some(80));
    // Below the card width only the status line remains.
    session.resize(12, 30);
    session.wait_for("the compact status", |s| !s.contains("╭") && s.contains("Em andamento"));
}

fn quits_with(keys: &[u8]) {
    let (_dir, mut session) = open_in_progress();
    session.send(keys);
    let status = session.exit();
    assert!(status.success(), "exit status {status:?}");
    session.assert_restored();
}

#[test]
fn q_quits_and_restores_the_terminal() {
    quits_with(b"q");
}

#[test]
fn escape_quits_and_restores_the_terminal() {
    quits_with(b"\x1b");
}

#[test]
fn ctrl_c_quits_and_restores_the_terminal() {
    quits_with(&[0x03]);
}

#[test]
fn a_panic_restores_the_terminal_before_the_message() {
    let dir = workspace("status-in_progress");
    let mut session = Session::start(dir.path(), 24, 58, &[("RELAY_TUI_TEST_PANIC", "1")]);
    let status = session.exit();
    assert!(!status.success(), "a panic must not exit 0");
    session.assert_restored();
    // The message is on the normal screen, where the user can read it.
    assert!(session.screen().contains("RELAY_TUI_TEST_PANIC"), "{}", session.screen());
}

#[test]
fn nothing_else_is_written_to_the_workspace() {
    let dir = workspace("status-in_progress");
    let before = snapshot(dir.path());
    let mut session = Session::start(dir.path(), 24, 58, &[]);
    session.wait_for("the handoff card", |s| s.contains("Em andamento"));
    session.send(b"q");
    session.exit();
    assert_eq!(snapshot(dir.path()), before, "the workspace changed while it was only observed");
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                files.push((path.strip_prefix(root).unwrap().display().to_string(), fs::read(&path).unwrap()));
            }
        }
    }
    walk(root, root, &mut files);
    files.sort();
    files
}

/// `cargo test --test e2e_pty -- --ignored --nocapture` prints what the
/// terminal actually shows.
#[test]
#[ignore]
fn print_the_real_screens() {
    for (case, rows, cols) in [("status-in_progress", 24, 58), ("status-blocked", 24, 58), ("status-in_progress", 20, 40)] {
        let dir = workspace(case);
        let mut session = Session::start(dir.path(), rows, cols, &[]);
        session.wait_for("a screen", |s| s.contains("atualizado"));
        println!("\n===== {case} {cols}x{rows} =====\n{}", session.screen());
        session.send(b"q");
        session.exit();
    }
}
