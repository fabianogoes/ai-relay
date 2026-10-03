use std::env;
use std::io::{self, IsTerminal};
use std::panic;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;

use relay_tui::app::{self, App, AppEvent};
use relay_tui::cli::{self, Action};
use relay_tui::workspace::{QUIESCENCE, resolve_workspace, watch_workspace};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn home() -> Option<PathBuf> {
    env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")).map(PathBuf::from)
}

fn run(workspace: Option<PathBuf>) -> io::Result<()> {
    if !io::stdout().is_terminal() {
        return Err(io::Error::other("a saída precisa ser um terminal"));
    }
    let workspace = resolve_workspace(workspace.as_deref(), &env::current_dir()?);
    let mut app = App::new(workspace.clone(), home().as_deref());

    // One channel for everything that can happen: keys and resizes from the
    // terminal, and the workspace changing on disk.
    let (tx, rx) = mpsc::channel();
    let watcher = watch_workspace(&workspace, QUIESCENCE).map_err(io::Error::other)?;
    let from_workspace = tx.clone();
    thread::spawn(move || {
        while let Ok(change) = watcher.events().recv() {
            if from_workspace.send(AppEvent::Workspace(change)).is_err() {
                break;
            }
        }
    });
    thread::spawn(move || {
        while let Ok(input) = event::read() {
            if tx.send(AppEvent::Terminal(input)).is_err() {
                break;
            }
        }
    });

    // `init` enters raw mode and the alternate screen, and installs a panic
    // hook that puts the terminal back before the message is printed.
    let mut terminal = ratatui::init();
    // The mouse is reported only while Histórico is open (so that Agora keeps
    // selectable text), and is turned off again on every way out: going back to
    // Agora, quitting, and a panic. `init` put a hook that restores the
    // terminal; this one runs first and then hands over to it.
    let restoring = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        restoring(info);
    }));
    if cfg!(debug_assertions) {
        if let Some(mode) = env::var_os("RELAY_TUI_TEST_PANIC") {
            // Only in debug builds: lets the end-to-end test prove the restore,
            // with Agora on screen or, for `history`, with the mouse reported.
            if mode == "history" {
                app.on_event(AppEvent::Terminal(event::Event::Key(
                    event::KeyCode::Tab.into(),
                )));
                execute!(io::stdout(), EnableMouseCapture)?;
            }
            terminal.draw(|frame| {
                let area = frame.area();
                app.render(area, frame.buffer_mut(), now_unix());
            })?;
            panic!("RELAY_TUI_TEST_PANIC");
        }
    }
    let result = app::run_with_mouse(&mut terminal, &mut app, &rx, now_unix, |on| {
        let _ = if on {
            execute!(io::stdout(), EnableMouseCapture)
        } else {
            execute!(io::stdout(), DisableMouseCapture)
        };
    });
    // Before the terminal is given back, so the shell never inherits a mouse
    // that reports into the prompt.
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result.map_err(io::Error::other)
}

fn main() -> ExitCode {
    match cli::parse_args(env::args().skip(1)) {
        Ok(Action::Version) => {
            println!("{}", cli::version());
            ExitCode::SUCCESS
        }
        Ok(Action::Help) => {
            print!("{}", cli::HELP);
            ExitCode::SUCCESS
        }
        Ok(Action::Run { workspace }) => match run(workspace) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("relay-tui: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("relay-tui: {error}\nUse --help para ver as opções.");
            ExitCode::from(2)
        }
    }
}
