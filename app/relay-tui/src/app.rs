//! The application: what is on screen and how events change it. Everything
//! here is testable without a terminal; `main` only wires it to one.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::Widget;

use crate::core::{RelayState, derive_state};
use crate::view::{Freshness, Screen, View};
use crate::workspace::{WorkspaceEvent, is_relay_workspace, read_workspace};

#[derive(Debug)]
pub enum AppEvent {
    Terminal(Event),
    Workspace(WorkspaceEvent),
}

enum Loaded {
    NotRelay,
    State(RelayState),
}

pub struct App {
    workspace: PathBuf,
    shown: String,
    loaded: Loaded,
    freshness: Freshness,
    quit: bool,
}

/// `path` as it is shown: the home directory is written `~`.
pub fn display_path(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|h| path.strip_prefix(h).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => {
            // Rebuilt from components so every separator is the platform's own
            // (a path given with `/` on Windows would otherwise mix both).
            let sep = std::path::MAIN_SEPARATOR_STR;
            let tail: Vec<_> = rest.components().map(|c| c.as_os_str().to_string_lossy()).collect();
            format!("~{sep}{}", tail.join(sep))
        }
        None => path.display().to_string(),
    }
}

fn load(workspace: &Path) -> Loaded {
    if is_relay_workspace(workspace) {
        Loaded::State(derive_state(&read_workspace(workspace)))
    } else {
        Loaded::NotRelay
    }
}

fn is_quit(key: &KeyEvent) -> bool {
    // Windows reports a key twice, on press and on release.
    if key.kind != KeyEventKind::Press {
        return false;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Esc => true,
        // In raw mode Ctrl-C is a key, not a signal.
        KeyCode::Char('c') | KeyCode::Char('C') => ctrl,
        KeyCode::Char('q') | KeyCode::Char('Q') => !ctrl,
        _ => false,
    }
}

impl App {
    pub fn new(workspace: PathBuf, home: Option<&Path>) -> Self {
        let shown = display_path(&workspace, home);
        let loaded = load(&workspace);
        App { workspace, shown, loaded, freshness: Freshness::Fresh, quit: false }
    }

    pub fn on_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Terminal(Event::Key(key)) if is_quit(&key) => self.quit = true,
            // A resize or any other input only needs a redraw, which the
            // loop does after every event.
            AppEvent::Terminal(_) => {}
            // The last snapshot stays on screen, marked as in flight.
            AppEvent::Workspace(WorkspaceEvent::Dirty) => self.freshness = Freshness::Updating,
            // The workspace is read again as a whole (ADR-0007 decision 4).
            AppEvent::Workspace(WorkspaceEvent::Settled) => {
                self.loaded = load(&self.workspace);
                self.freshness = Freshness::Fresh;
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn view(&self, now_unix: i64) -> View<'_> {
        View {
            screen: match &self.loaded {
                Loaded::NotRelay => Screen::NotARelayWorkspace,
                Loaded::State(state) => Screen::State(state),
            },
            workspace: &self.shown,
            freshness: self.freshness,
            now_unix,
        }
    }
}

/// How long the loop waits for an event before drawing anyway, so that
/// "há 4 min" keeps up with the clock.
pub const TICK: Duration = Duration::from_secs(15);

/// Draws, waits for an event, applies it and every one already waiting, and
/// draws again, until the user quits or the event source is gone. `clock`
/// gives the Unix time, which the screen needs for "há 4 min".
pub fn run<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    events: &Receiver<AppEvent>,
    clock: impl Fn() -> i64,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    B: Backend,
    B::Error: Error + Send + Sync + 'static,
{
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            (&app.view(clock())).render(area, frame.buffer_mut());
        })?;
        match events.recv_timeout(TICK) {
            Ok(event) => app.on_event(event),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        // A burst of events costs one redraw, not one each.
        while let Ok(event) = events.try_recv() {
            app.on_event(event);
        }
        if app.should_quit() {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::core::WorkStatus;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> AppEvent {
        AppEvent::Terminal(Event::Key(KeyEvent::new(code, modifiers)))
    }

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn status_of(app: &App) -> Option<WorkStatus> {
        match app.view(0).screen {
            Screen::State(RelayState::Ok(ok)) => Some(ok.status),
            _ => None,
        }
    }

    fn is_not_relay(app: &App) -> bool {
        matches!(app.view(0).screen, Screen::NotARelayWorkspace)
    }

    const BACKLOG: &str = "# Backlog\n\n- [ ] B-001 - Algo (spec: .specs/20261002-001-x.md)\n";

    #[test]
    fn a_directory_without_orchestration_is_not_a_relay_workspace() {
        let dir = TempDir::new().unwrap();
        assert!(is_not_relay(&App::new(dir.path().to_path_buf(), None)));
    }

    #[test]
    fn a_workspace_with_records_derives_its_state() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".orchestration/BACKLOG.md", BACKLOG);
        let app = App::new(dir.path().to_path_buf(), None);
        assert_eq!(status_of(&app), Some(WorkStatus::Backlog));
        assert_eq!(app.view(0).freshness, Freshness::Fresh);
    }

    #[test]
    fn dirty_marks_the_snapshot_as_updating_and_keeps_it() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".orchestration/BACKLOG.md", BACKLOG);
        let mut app = App::new(dir.path().to_path_buf(), None);
        // The files change, but nothing is read until the burst settles.
        write(dir.path(), ".orchestration/BACKLOG.md", "# Backlog\n");
        app.on_event(AppEvent::Workspace(WorkspaceEvent::Dirty));
        assert_eq!(app.view(0).freshness, Freshness::Updating);
        assert_eq!(status_of(&app), Some(WorkStatus::Backlog));
    }

    #[test]
    fn settled_reads_the_workspace_again_and_is_fresh() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), ".orchestration/BACKLOG.md", BACKLOG);
        let mut app = App::new(dir.path().to_path_buf(), None);
        write(dir.path(), ".orchestration/BACKLOG.md", "# Backlog\n\n- [x] B-001 - Algo (spec: .specs/x.md)\n");
        app.on_event(AppEvent::Workspace(WorkspaceEvent::Dirty));
        app.on_event(AppEvent::Workspace(WorkspaceEvent::Settled));
        assert_eq!(app.view(0).freshness, Freshness::Fresh);
        // The one entry is done now, and the spec has no criteria to demand.
        assert_eq!(status_of(&app), Some(WorkStatus::Done));
    }

    #[test]
    fn a_workspace_that_appears_later_replaces_the_not_relay_card() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().to_path_buf(), None);
        assert!(is_not_relay(&app));
        write(dir.path(), ".orchestration/BACKLOG.md", BACKLOG);
        app.on_event(AppEvent::Workspace(WorkspaceEvent::Settled));
        assert_eq!(status_of(&app), Some(WorkStatus::Backlog));
    }

    #[test]
    fn q_escape_and_ctrl_c_quit() {
        for event in [
            key(KeyCode::Char('q'), KeyModifiers::NONE),
            key(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            key(KeyCode::Esc, KeyModifiers::NONE),
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let mut app = App::new(PathBuf::from("/nowhere"), None);
            app.on_event(event);
            assert!(app.should_quit());
        }
    }

    #[test]
    fn nothing_else_quits() {
        let mut app = App::new(PathBuf::from("/nowhere"), None);
        for event in [
            key(KeyCode::Char('c'), KeyModifiers::NONE),
            key(KeyCode::Char('q'), KeyModifiers::CONTROL),
            key(KeyCode::Char('x'), KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
            AppEvent::Terminal(Event::Resize(80, 24)),
            AppEvent::Workspace(WorkspaceEvent::Dirty),
        ] {
            app.on_event(event);
            assert!(!app.should_quit());
        }
    }

    #[test]
    fn a_key_release_does_not_quit() {
        let mut app = App::new(PathBuf::from("/nowhere"), None);
        let mut release = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        app.on_event(AppEvent::Terminal(Event::Key(release)));
        assert!(!app.should_quit());
    }

    #[test]
    fn the_home_directory_is_written_as_a_tilde() {
        let home = Path::new("/Users/me");
        let sep = std::path::MAIN_SEPARATOR_STR;
        assert_eq!(
            display_path(Path::new("/Users/me/Developer/relay"), Some(home)),
            format!("~{sep}Developer{sep}relay")
        );
        assert_eq!(display_path(Path::new("/Users/me"), Some(home)), "~");
        assert_eq!(display_path(Path::new("/srv/relay"), Some(home)), "/srv/relay");
        assert_eq!(display_path(Path::new("/Users/me/x"), None), "/Users/me/x");
        // A sibling that merely starts with the same letters is not inside home.
        assert_eq!(display_path(Path::new("/Users/meow/x"), Some(home)), "/Users/meow/x");
    }

    #[test]
    fn the_loop_draws_each_state_and_stops_on_quit() {
        use ratatui::backend::TestBackend;
        use std::sync::mpsc;

        let dir = TempDir::new().unwrap();
        write(dir.path(), ".orchestration/BACKLOG.md", BACKLOG);
        let mut app = App::new(dir.path().to_path_buf(), None);
        let mut terminal = Terminal::new(TestBackend::new(58, 12)).unwrap();
        let (tx, rx) = mpsc::channel();
        // Everything is already waiting: the loop applies it as one burst.
        write(dir.path(), ".orchestration/BACKLOG.md", "# Backlog\n");
        tx.send(AppEvent::Workspace(WorkspaceEvent::Dirty)).unwrap();
        tx.send(AppEvent::Workspace(WorkspaceEvent::Settled)).unwrap();
        tx.send(key(KeyCode::Char('q'), KeyModifiers::NONE)).unwrap();

        run(&mut terminal, &mut app, &rx, || 0).unwrap();

        assert!(app.should_quit());
        let screen: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        // The first draw, before any event, showed the A escolher card; the
        // loop stopped before drawing the quit.
        assert!(screen.contains("A escolher"), "{screen}");
        assert!(screen.contains("atualizado") && !screen.contains("atualizando"));
    }

    #[test]
    fn the_loop_ends_when_the_event_source_is_gone() {
        use ratatui::backend::TestBackend;
        use std::sync::mpsc;

        let mut app = App::new(PathBuf::from("/nowhere"), None);
        let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
        let (tx, rx) = mpsc::channel::<AppEvent>();
        drop(tx);
        run(&mut terminal, &mut app, &rx, || 0).unwrap();
        assert!(!app.should_quit());
    }
}
