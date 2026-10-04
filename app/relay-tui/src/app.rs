//! The application: what is on screen and how events change it. Everything
//! here is testable without a terminal; `main` only wires it to one.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::core::{History, RelayFiles, RelayState, derive_state, extract_history};
use crate::language::{self, Language, Locale};
use crate::nav::{Ctx, Effect, Input, Nav, Pane};
use crate::view::{
    Freshness, HistoryScreen, Screen, SettingsScreen, View, detail_extent, list_geometry, row_at,
};
use crate::workspace::{WorkspaceEvent, is_relay_workspace, read_language, read_workspace};

#[derive(Debug)]
pub enum AppEvent {
    Terminal(Event),
    Workspace(WorkspaceEvent),
    Signal(i32),
}

// One value lives at a time and it is replaced whole on every reload, so the size
// of the largest variant costs nothing worth an indirection.
#[allow(clippy::large_enum_variant)]
enum Loaded {
    NotRelay,
    State(RelayState),
}

pub struct App {
    workspace: PathBuf,
    shown: String,
    loaded: Loaded,
    /// The Histórico lists, extracted from the same read as `loaded`.
    history: History,
    nav: Nav,
    /// The terminal size of the last `fit`, to place a click on a row.
    size: (u16, u16),
    freshness: Freshness,
    quit: bool,
    signal: Option<i32>,
    settings_open: bool,
    settings_selection: Language,
    runtime_language: Option<Language>,
    explicit_language: Option<Language>,
    locale: Locale,
    language: Language,
}

/// `path` as it is shown: the home directory is written `~`.
pub fn display_path(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|h| path.strip_prefix(h).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => {
            // Rebuilt from components so every separator is the platform's own
            // (a path given with `/` on Windows would otherwise mix both).
            let sep = std::path::MAIN_SEPARATOR_STR;
            let tail: Vec<_> = rest
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            format!("~{sep}{}", tail.join(sep))
        }
        None => path.display().to_string(),
    }
}

/// One read of the workspace, from which both the derived state and the
/// Histórico lists come.
fn load(workspace: &Path) -> (Loaded, History) {
    if is_relay_workspace(workspace) {
        let files: RelayFiles = read_workspace(workspace);
        (Loaded::State(derive_state(&files)), extract_history(&files))
    } else {
        (Loaded::NotRelay, History::default())
    }
}

fn ctx<'a>(loaded: &'a Loaded, history: &'a History) -> Ctx<'a> {
    Ctx {
        history,
        ok: match loaded {
            Loaded::State(RelayState::Ok(ok)) => Some(ok),
            _ => None,
        },
    }
}

/// What a key means to the navigation, or nothing.
fn key_input(key: &KeyEvent) -> Option<Input> {
    // Windows reports a key twice, on press and on release.
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    Some(match key.code {
        KeyCode::Esc => Input::Esc,
        KeyCode::Backspace => Input::Backspace,
        KeyCode::Enter => Input::Enter,
        KeyCode::Tab => Input::Toggle,
        KeyCode::Up => Input::Up,
        KeyCode::Down => Input::Down,
        KeyCode::PageUp => Input::PageUp,
        KeyCode::PageDown => Input::PageDown,
        // In raw mode Ctrl-C is a key, not a signal.
        KeyCode::Char('c') | KeyCode::Char('C') if ctrl => Input::Quit,
        // The other letters are plain keys; with Ctrl they are not ours.
        KeyCode::Char(_) if ctrl => return None,
        KeyCode::Char('q') | KeyCode::Char('Q') => Input::Quit,
        KeyCode::Char('t') | KeyCode::Char('T') => Input::Toggle,
        KeyCode::Char('r') | KeyCode::Char('R') => Input::Reload,
        KeyCode::Char('c') | KeyCode::Char('C') => Input::Settings,
        KeyCode::Char('k') | KeyCode::Char('K') => Input::Up,
        KeyCode::Char('j') | KeyCode::Char('J') => Input::Down,
        KeyCode::Char('y') | KeyCode::Char('Y') => Input::Confirm,
        _ => return None,
    })
}

impl App {
    pub fn new(workspace: PathBuf, home: Option<&Path>) -> Self {
        // The convenience constructor is deterministic for tests and examples;
        // the executable passes the real locale explicitly.
        Self::new_with_language(
            workspace,
            home,
            None,
            Locale {
                lang: Some("pt_BR".into()),
                ..Locale::default()
            },
        )
    }

    pub fn new_with_language(
        workspace: PathBuf,
        home: Option<&Path>,
        explicit_language: Option<Language>,
        locale: Locale,
    ) -> Self {
        let shown = display_path(&workspace, home);
        let (loaded, history) = load(&workspace);
        let language = language::resolve(explicit_language, read_language(&workspace), &locale);
        App {
            workspace,
            shown,
            loaded,
            history,
            nav: Nav::new(),
            size: (0, 0),
            freshness: Freshness::Fresh,
            quit: false,
            signal: None,
            settings_open: false,
            settings_selection: language,
            runtime_language: None,
            explicit_language,
            locale,
            language,
        }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn on_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Terminal(Event::Key(key)) => {
                // While "leave?" is asked, a key that means nothing else still
                // answers it (by cancelling).
                let answers = self.nav.confirming_quit() && key.kind == KeyEventKind::Press;
                let input = key_input(&key);
                if self.nav.confirming_quit() {
                    if let Some(input) = input.or(answers.then_some(Input::Other)) {
                        self.navigate(input);
                    }
                } else if self.settings_open {
                    if let Some(input) = input {
                        self.handle_settings_input(input);
                    }
                } else if input == Some(Input::Settings) {
                    self.settings_selection = self.language;
                    self.settings_open = true;
                } else if let Some(input) = input {
                    self.navigate(input);
                }
            }
            AppEvent::Terminal(Event::Mouse(_)) if self.settings_open => {}
            AppEvent::Terminal(Event::Mouse(mouse)) => match mouse.kind {
                MouseEventKind::ScrollUp => self.navigate(Input::WheelUp),
                MouseEventKind::ScrollDown => self.navigate(Input::WheelDown),
                // A click means a row only where the list is drawn, so it is
                // placed on the screen first; in Agora there is nothing to click.
                MouseEventKind::Down(MouseButton::Left) if self.nav.pane() == Pane::History => {
                    let (width, height) = self.size;
                    let hit = row_at(
                        &self.nav,
                        &ctx(&self.loaded, &self.history),
                        width,
                        height,
                        mouse.column,
                        mouse.row,
                    );
                    if let Some(index) = hit {
                        self.navigate(Input::Click(index));
                    }
                }
                _ => {}
            },
            // A resize or any other input only needs a redraw, which the
            // loop does after every event.
            AppEvent::Terminal(_) => {}
            // The last snapshot stays on screen, marked as in flight.
            AppEvent::Workspace(WorkspaceEvent::Dirty) => self.freshness = Freshness::Updating,
            // The workspace is read again as a whole, never patched per event.
            AppEvent::Workspace(WorkspaceEvent::Settled) => self.reload(),
            AppEvent::Signal(signal) => {
                self.signal = Some(signal);
                self.quit = true;
            }
        }
    }

    fn navigate(&mut self, input: Input) {
        match self.nav.handle(input, &ctx(&self.loaded, &self.history)) {
            Effect::None => {}
            Effect::Quit => self.quit = true,
            // The same read as after the watcher settles, asked for by hand
            // (`r`), for when the watcher delivers nothing.
            Effect::Reload => self.reload(),
        }
    }

    fn handle_settings_input(&mut self, input: Input) {
        match input {
            Input::Up | Input::Down => {
                self.settings_selection = match self.settings_selection {
                    Language::En => Language::PtBr,
                    Language::PtBr => Language::En,
                };
            }
            Input::Enter => {
                self.runtime_language = Some(self.settings_selection);
                self.language = self.settings_selection;
                self.settings_open = false;
            }
            Input::Esc | Input::Backspace => self.settings_open = false,
            Input::Quit => self.quit = true,
            _ => {}
        }
    }

    /// Reads the workspace again; what is selected survives by id.
    fn reload(&mut self) {
        (self.loaded, self.history) = load(&self.workspace);
        self.language = self.runtime_language.unwrap_or_else(|| {
            language::resolve(
                self.explicit_language,
                read_language(&self.workspace),
                &self.locale,
            )
        });
        self.freshness = Freshness::Fresh;
        self.nav.reconcile(&ctx(&self.loaded, &self.history));
    }

    /// Draws the open view: Agora, or Histórico when it is the one open.
    pub fn render(&self, area: Rect, buf: &mut Buffer, now_unix: i64) {
        if self.settings_open {
            (&SettingsScreen {
                workspace: &self.shown,
                freshness: self.freshness,
                language: self.language,
                selected: self.settings_selection,
            })
                .render(area, buf);
            return;
        }
        match self.nav.pane() {
            Pane::Now => {
                (&self.view(now_unix)).render(area, buf);
                if self.nav.confirming_quit() {
                    crate::view::quit_prompt(area, buf, self.language);
                }
            }
            Pane::History => (&HistoryScreen {
                workspace: &self.shown,
                freshness: self.freshness,
                language: self.language,
                nav: &self.nav,
                ctx: ctx(&self.loaded, &self.history),
            })
                .render(area, buf),
        }
    }

    /// Tells the navigation how big the lists are on a terminal of this size:
    /// the page keys move by the rows that fit and the detail scrolls only as
    /// far as its wrapped lines go.
    pub fn fit(&mut self, width: u16, height: u16) {
        self.size = (width, height);
        if let Some((_, rows)) = list_geometry(width, height) {
            self.nav.set_page(rows);
        }
        let max = detail_extent(&self.nav, &ctx(&self.loaded, &self.history), width, height);
        self.nav.set_detail_max(max);
    }

    /// Whether the terminal should report the mouse: only while Histórico is
    /// open, so that in Agora the text can still be selected and copied.
    pub fn wants_mouse(&self) -> bool {
        !self.settings_open && self.nav.pane() == Pane::History
    }

    pub fn nav(&self) -> &Nav {
        &self.nav
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn signal_exit_code(&self) -> Option<u8> {
        self.signal.map(|signal| 128 + signal as u8)
    }

    pub fn view(&self, now_unix: i64) -> View<'_> {
        View {
            screen: match &self.loaded {
                Loaded::NotRelay => Screen::NotARelayWorkspace,
                Loaded::State(state) => Screen::State(state),
            },
            history: &self.history,
            workspace: &self.shown,
            freshness: self.freshness,
            now_unix,
            language: self.language,
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
    run_with_mouse(terminal, app, events, clock, |_| {})
}

/// `run`, and `set_mouse(true)` or `set_mouse(false)` whenever the app starts
/// or stops wanting the mouse reported. A terminal that was told to report it
/// is not told to stop here when the loop ends: whoever supplied `set_mouse`
/// turns it off (the end of the loop and a panic both need that).
pub fn run_with_mouse<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    events: &Receiver<AppEvent>,
    clock: impl Fn() -> i64,
    mut set_mouse: impl FnMut(bool),
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    B: Backend,
    B::Error: Error + Send + Sync + 'static,
{
    let mut mouse = false;
    loop {
        if app.wants_mouse() != mouse {
            mouse = app.wants_mouse();
            set_mouse(mouse);
        }
        let size = terminal.size()?;
        app.fit(size.width, size.height);
        terminal.draw(|frame| {
            let area = frame.area();
            app.render(area, frame.buffer_mut(), clock());
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
    fn explicit_language_wins_and_workspace_language_is_reloaded() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            ".orchestration/SETTINGS.md",
            "# Settings\n\n- Language: pt-BR\n",
        );
        let locale = Locale {
            lang: Some("en_US.UTF-8".into()),
            ..Locale::default()
        };
        let mut app = App::new_with_language(dir.path().to_path_buf(), None, None, locale.clone());
        assert_eq!(app.language(), Language::PtBr);
        write(
            dir.path(),
            ".orchestration/SETTINGS.md",
            "# Settings\n\n- Language: en\n",
        );
        app.reload();
        assert_eq!(app.language(), Language::En);
        let explicit =
            App::new_with_language(dir.path().to_path_buf(), None, Some(Language::PtBr), locale);
        assert_eq!(explicit.language(), Language::PtBr);
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
        write(
            dir.path(),
            ".orchestration/BACKLOG.md",
            "# Backlog\n\n- [x] B-001 - Algo (spec: .specs/x.md)\n",
        );
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
    fn q_and_ctrl_c_quit_at_once() {
        for event in [
            key(KeyCode::Char('q'), KeyModifiers::NONE),
            key(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let mut app = App::new(PathBuf::from("/nowhere"), None);
            app.on_event(event);
            assert!(app.should_quit());
        }
    }

    #[test]
    fn esc_in_agora_asks_and_only_esc_enter_or_y_leave() {
        for yes in [
            KeyCode::Esc,
            KeyCode::Enter,
            KeyCode::Char('y'),
            KeyCode::Char('Y'),
        ] {
            let mut app = App::new(PathBuf::from("/nowhere"), None);
            press(&mut app, KeyCode::Esc);
            assert!(!app.should_quit() && app.nav().confirming_quit(), "{yes:?}");
            press(&mut app, yes);
            assert!(app.should_quit(), "{yes:?}");
        }
        // A key with no meaning of its own cancels too, and does not leave.
        for no in [
            KeyCode::Char('x'),
            KeyCode::Char('n'),
            KeyCode::Char('c'),
            KeyCode::F(5),
            KeyCode::Left,
            KeyCode::Tab,
        ] {
            let mut app = App::new(PathBuf::from("/nowhere"), None);
            press(&mut app, KeyCode::Esc);
            press(&mut app, no);
            assert!(!app.should_quit() && !app.nav().confirming_quit(), "{no:?}");
        }
        // A key release answers nothing.
        let mut app = App::new(PathBuf::from("/nowhere"), None);
        press(&mut app, KeyCode::Esc);
        let mut release = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        app.on_event(AppEvent::Terminal(Event::Key(release)));
        assert!(app.nav().confirming_quit());
    }

    #[test]
    fn the_question_replaces_the_footer_and_goes_away_when_answered() {
        let mut app = App::new(PathBuf::from("/nowhere"), None);
        let draw = |app: &App, width: u16| {
            let area = Rect::new(0, 0, width, 16);
            let mut buf = Buffer::empty(area);
            app.render(area, &mut buf, 0);
            // The footer is a card: its text is on the row above the bottom border,
            // between the two side borders.
            (0..16)
                .map(|y| {
                    (0..width)
                        .map(|x| buf[(x, y)].symbol())
                        .collect::<String>()
                        .trim_end()
                        .to_string()
                })
                .collect::<Vec<_>>()
        };
        let footer_text = |lines: Vec<String>| {
            lines[lines.len() - 2]
                .trim_matches(|c| c == '│' || c == ' ')
                .to_string()
        };
        assert!(footer_text(draw(&app, 58)).contains("Tab histórico"));
        press(&mut app, KeyCode::Esc);
        assert_eq!(
            footer_text(draw(&app, 80)),
            "Sair? Esc, Enter ou y confirmam · outra tecla cancela"
        );
        assert_eq!(
            footer_text(draw(&app, 58)),
            "Sair? Esc, Enter ou y confirmam · outra tecla cancela"
        );
        assert_eq!(
            footer_text(draw(&app, 40)),
            "Sair? Esc, Enter ou y confirmam"
        );
        assert_eq!(footer_text(draw(&app, 30)), "Sair? Esc ou y confirmam");
        assert_eq!(footer_text(draw(&app, 12)), "Sair? y");
        press(&mut app, KeyCode::Char('x'));
        assert!(footer_text(draw(&app, 58)).contains("Tab histórico"));
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

    fn press(app: &mut App, code: KeyCode) {
        app.on_event(key(code, KeyModifiers::NONE));
    }

    fn history_workspace(dir: &Path) {
        write(
            dir,
            ".orchestration/BACKLOG.md",
            "# Backlog\n\n- [x] B-001 - Um (spec: `.specs/20260101-001-a.md`)\n\
             - [ ] B-002 - Dois (spec: `.specs/20260101-001-a.md`)\n",
        );
        write(
            dir,
            ".specs/20260101-001-a.md",
            "# 20260101-001 - Primeira\n",
        );
        write(
            dir,
            ".orchestration/CHANGELOG.md",
            "# Change log\n\n## 2026-01-01 - T-001 - Feito\n- Backlog: B-001\n- Criteria: none\n",
        );
    }

    #[test]
    fn tab_and_t_toggle_the_view_and_esc_in_historico_goes_back_instead_of_quitting() {
        use crate::nav::{Level, Pane};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        assert_eq!(app.nav().pane(), Pane::Now);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Enter);
        assert_eq!(
            (app.nav().pane(), app.nav().level()),
            (Pane::History, Level::Items)
        );
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.nav().pane(), Pane::Now);
        assert!(!app.should_quit());
        // In Agora `Esc` asks first (see the test below); `t` toggles like Tab.
        press(&mut app, KeyCode::Char('t'));
        assert_eq!(app.nav().pane(), Pane::History);
        press(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit());
    }

    #[test]
    fn the_open_view_is_the_one_drawn_and_fit_sizes_the_page() {
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        let draw = |app: &App| {
            let area = Rect::new(0, 0, 58, 16);
            let mut buf = Buffer::empty(area);
            app.render(area, &mut buf, 0);
            (0..16)
                .map(|y| (0..58).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let now = draw(&app);
        assert!(
            now.contains("Tab histórico") && now.contains("c config.") && !now.contains("Specs"),
            "{now}"
        );
        press(&mut app, KeyCode::Tab);
        app.fit(58, 16);
        let history = draw(&app);
        assert!(
            history.contains("Specs") && history.contains("20260101-001"),
            "{history}"
        );
        assert!(history.contains("Esc voltar"), "{history}");
        assert!(history.contains("c config."), "{history}");
        press(&mut app, KeyCode::Tab);
        assert!(draw(&app).contains("Tab histórico"));
    }

    fn rendered_text(app: &App) -> String {
        let area = Rect::new(0, 0, 58, 16);
        let mut buf = Buffer::empty(area);
        app.render(area, &mut buf, 0);
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn c_opens_settings_and_escape_restores_the_previous_history_location() {
        use crate::nav::{Level, Pane};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Down);
        let before = (
            app.nav().pane(),
            app.nav().level(),
            app.nav().selected_index(Level::Items),
        );
        assert!(app.wants_mouse());

        press(&mut app, KeyCode::Char('c'));
        assert!(!app.wants_mouse());
        let settings = rendered_text(&app);
        assert!(settings.contains("Configurações"), "{settings}");
        assert!(settings.contains("Idioma"), "{settings}");
        assert!(settings.contains("English"), "{settings}");
        assert!(settings.contains("Português (Brasil)"), "{settings}");

        press(&mut app, KeyCode::Esc);
        assert_eq!(app.nav().pane(), Pane::History);
        assert_eq!(app.nav().level(), Level::Items);
        assert_eq!(app.nav().selected_index(Level::Items), before.2);
        assert!(app.wants_mouse());
        assert!(!app.should_quit());
    }

    #[test]
    fn settings_applies_language_for_this_run_and_keeps_it_after_workspace_reload() {
        use crate::language::Language;
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().to_path_buf(), None);
        assert_eq!(app.language(), Language::PtBr);

        press(&mut app, KeyCode::Char('c'));
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Enter);

        assert_eq!(app.language(), Language::En);
        app.on_event(AppEvent::Workspace(WorkspaceEvent::Settled));
        assert_eq!(app.language(), Language::En);
        assert!(!dir.path().join(".orchestration/SETTINGS.md").exists());
    }

    #[test]
    fn settings_uses_english_labels_and_can_apply_portuguese_too() {
        use crate::language::Language;
        let dir = TempDir::new().unwrap();
        let mut app = App::new_with_language(
            dir.path().to_path_buf(),
            None,
            Some(Language::En),
            Locale::default(),
        );

        press(&mut app, KeyCode::Char('c'));
        let settings = rendered_text(&app);
        assert!(settings.contains("Settings"), "{settings}");
        assert!(settings.contains("Language"), "{settings}");
        assert!(settings.contains("Portuguese (Brazil)"), "{settings}");
        assert!(settings.contains("· current"), "{settings}");
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Enter);

        assert_eq!(app.language(), Language::PtBr);
    }

    #[test]
    fn leaving_settings_without_enter_discards_the_pending_language() {
        use crate::language::Language;
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().to_path_buf(), None);

        press(&mut app, KeyCode::Char('c'));
        assert!(rendered_text(&app).contains("Configurações"));
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Esc);

        assert_eq!(app.language(), Language::PtBr);
        assert!(!app.should_quit());
    }

    #[test]
    fn q_and_ctrl_c_still_quit_from_settings() {
        for event in [
            key(KeyCode::Char('q'), KeyModifiers::NONE),
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let dir = TempDir::new().unwrap();
            let mut app = App::new(dir.path().to_path_buf(), None);
            press(&mut app, KeyCode::Char('c'));
            assert!(rendered_text(&app).contains("Configurações"));
            app.on_event(event);
            assert!(app.should_quit());
        }
    }

    fn click(app: &mut App, column: u16, row: u16) {
        use ratatui::crossterm::event::{MouseButton, MouseEvent};
        app.on_event(AppEvent::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })));
    }

    #[test]
    fn a_click_on_a_row_selects_it_and_opens_the_next_level() {
        use crate::nav::{Level, RowKey};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        press(&mut app, KeyCode::Tab);
        app.fit(58, 16);
        // At 58x16 the card starts on row 2 and its first row is row 3.
        click(&mut app, 5, 3);
        assert_eq!(app.nav().level(), Level::Items);
        click(&mut app, 5, 4);
        assert_eq!(app.nav().level(), Level::Tasks);
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-002".into()))
        );
    }

    #[test]
    fn a_click_outside_a_row_is_ignored() {
        use crate::nav::{Level, Pane};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        // In Agora there is nothing to click, even where Histórico has a row.
        app.fit(58, 16);
        click(&mut app, 5, 3);
        assert_eq!(
            (app.nav().pane(), app.nav().level()),
            (Pane::Now, Level::Specs)
        );
        press(&mut app, KeyCode::Tab);
        // Header, the card's top border, its side borders, the empty area below
        // the one row, the footer, and past the screen.
        for (column, row) in [
            (5, 0),
            (5, 1),
            (5, 2),
            (0, 3),
            (57, 3),
            (5, 4),
            (5, 9),
            (5, 15),
            (5, 40),
            (200, 3),
        ] {
            click(&mut app, column, row);
            assert_eq!(app.nav().level(), Level::Specs, "({column}, {row})");
        }
        // Another button does not click.
        use ratatui::crossterm::event::{MouseButton, MouseEvent};
        app.on_event(AppEvent::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 5,
            row: 3,
            modifiers: KeyModifiers::NONE,
        })));
        assert_eq!(app.nav().level(), Level::Specs);
    }

    #[test]
    fn a_click_in_the_detail_is_ignored_and_a_scrolled_list_maps_through_its_window() {
        use crate::nav::{Level, RowKey};
        let dir = TempDir::new().unwrap();
        let backlog: String = (1..=30)
            .map(|n| format!("- [ ] B-{n:03} - Item {n} (spec: `.specs/20260101-001-a.md`)\n"))
            .collect();
        write(
            dir.path(),
            ".orchestration/BACKLOG.md",
            &format!("# Backlog\n\n{backlog}"),
        );
        write(
            dir.path(),
            ".specs/20260101-001-a.md",
            "# 20260101-001 - A\n",
        );
        write(
            dir.path(),
            ".orchestration/CHANGELOG.md",
            "# Change log\n\n## 2026-01-01 - T-001 - Feito\n- Backlog: B-020\n- Criteria: none\n",
        );
        let mut app = App::new(dir.path().to_path_buf(), None);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Enter);
        app.fit(58, 18);
        for _ in 0..20 {
            press(&mut app, KeyCode::Down);
        }
        // 10 rows on screen, B-021 selected: the window starts at the sixteenth
        // item (B-016), on screen row 3.
        click(&mut app, 5, 3);
        assert_eq!(app.nav().level(), Level::Tasks);
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-016".into()))
        );
        // Back: B-016 is the selection now, so the window moved to start at B-011
        // and its tenth and last row, on screen row 12, is B-020.
        press(&mut app, KeyCode::Esc);
        click(&mut app, 5, 12);
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-020".into()))
        );
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.nav().level(), Level::Detail);
        click(&mut app, 5, 3);
        assert_eq!(app.nav().level(), Level::Detail);
    }

    #[test]
    fn j_k_and_the_wheel_move_the_selection() {
        use crate::nav::{Level, RowKey};
        use ratatui::crossterm::event::{MouseEvent, MouseEventKind};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-002".into()))
        );
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-001".into()))
        );
        let wheel = |kind| {
            AppEvent::Terminal(Event::Mouse(MouseEvent {
                kind,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            }))
        };
        app.on_event(wheel(MouseEventKind::ScrollDown));
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-002".into()))
        );
        app.on_event(wheel(MouseEventKind::ScrollUp));
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-001".into()))
        );
    }

    #[test]
    fn r_reads_the_workspace_again_without_a_watcher_event_and_keeps_the_place() {
        use crate::nav::{Level, RowKey};
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Char('j'));
        let before = app.history().specs[0].items.len();
        // A new item arrives on disk; the watcher says nothing.
        let backlog = fs::read_to_string(dir.path().join(".orchestration/BACKLOG.md")).unwrap();
        write(
            dir.path(),
            ".orchestration/BACKLOG.md",
            &format!("{backlog}- [ ] B-003 - Tres (spec: `.specs/20260101-001-a.md`)\n"),
        );
        assert_eq!(app.history().specs[0].items.len(), before);
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.history().specs[0].items.len(), before + 1);
        assert_eq!(app.view(0).freshness, Freshness::Fresh);
        // Same level, same selection, by id.
        assert_eq!(app.nav().level(), Level::Items);
        assert_eq!(
            app.nav().selected(Level::Items),
            Some(&RowKey::Item("B-002".into()))
        );
        assert!(!app.should_quit());
    }

    #[test]
    fn navigating_and_reloading_never_change_a_record() {
        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let snapshot = |root: &Path| {
            let mut all = Vec::new();
            for name in [
                ".orchestration/BACKLOG.md",
                ".orchestration/CHANGELOG.md",
                ".specs/20260101-001-a.md",
            ] {
                all.push(fs::read_to_string(root.join(name)).unwrap());
            }
            all
        };
        let before = snapshot(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        for code in [
            KeyCode::Tab,
            KeyCode::Enter,
            KeyCode::Down,
            KeyCode::Enter,
            KeyCode::Char('r'),
            KeyCode::Backspace,
            KeyCode::Tab,
            KeyCode::Char('r'),
        ] {
            press(&mut app, code);
        }
        assert_eq!(snapshot(dir.path()), before);
    }

    #[test]
    fn a_control_letter_is_not_a_navigation_key() {
        use crate::nav::Pane;
        let mut app = App::new(PathBuf::from("/nowhere"), None);
        for c in ['t', 'r', 'j', 'k', 'q'] {
            app.on_event(key(KeyCode::Char(c), KeyModifiers::CONTROL));
        }
        assert_eq!(app.nav().pane(), Pane::Now);
        assert!(!app.should_quit());
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
        assert_eq!(
            display_path(Path::new("/srv/relay"), Some(home)),
            "/srv/relay"
        );
        assert_eq!(display_path(Path::new("/Users/me/x"), None), "/Users/me/x");
        // A sibling that merely starts with the same letters is not inside home.
        assert_eq!(
            display_path(Path::new("/Users/meow/x"), Some(home)),
            "/Users/meow/x"
        );
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
        tx.send(AppEvent::Workspace(WorkspaceEvent::Settled))
            .unwrap();
        tx.send(key(KeyCode::Char('q'), KeyModifiers::NONE))
            .unwrap();

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
    fn the_mouse_is_wanted_only_while_historico_is_open() {
        use ratatui::backend::TestBackend;
        use std::sync::mpsc;
        use std::thread;

        let dir = TempDir::new().unwrap();
        history_workspace(dir.path());
        let mut app = App::new(dir.path().to_path_buf(), None);
        assert!(!app.wants_mouse());
        let mut terminal = Terminal::new(TestBackend::new(58, 16)).unwrap();
        let (tx, rx) = mpsc::channel();
        let (told, calls) = mpsc::channel();
        // Each event is sent only after the loop reacted to the one before, so
        // none is folded into the same burst.
        let driver = thread::spawn(move || {
            let mut seen = Vec::new();
            for code in [KeyCode::Tab, KeyCode::Esc, KeyCode::Tab, KeyCode::Tab] {
                tx.send(key(code, KeyModifiers::NONE)).unwrap();
                seen.push(calls.recv().unwrap());
            }
            // Agora again; now quit from Agora with the mouse already off.
            tx.send(key(KeyCode::Char('q'), KeyModifiers::NONE))
                .unwrap();
            seen
        });
        run_with_mouse(
            &mut terminal,
            &mut app,
            &rx,
            || 0,
            |on| told.send(on).unwrap(),
        )
        .unwrap();
        // Tab opens (on), Esc goes back from the specs level to Agora (off),
        // Tab opens (on), Tab goes back to Agora (off).
        assert_eq!(driver.join().unwrap(), [true, false, true, false]);
        assert!(!app.wants_mouse());
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
