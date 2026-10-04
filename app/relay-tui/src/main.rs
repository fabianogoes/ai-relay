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
use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

use relay_tui::app::{self, App, AppEvent};
use relay_tui::cli::{self, Action};
use relay_tui::language::{self, Language, Locale};
use relay_tui::workspace::{QUIESCENCE, read_language, resolve_workspace, watch_workspace};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn home() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn run(workspace: Option<PathBuf>, explicit: Option<Language>, locale: Locale) -> io::Result<u8> {
    let workspace = resolve_workspace(workspace.as_deref(), &env::current_dir()?);
    let language = language::resolve(explicit, read_language(&workspace), &locale);
    if !io::stdout().is_terminal() {
        return Err(io::Error::other(match language {
            Language::En => "stdout must be a terminal",
            Language::PtBr => "a saída precisa ser um terminal",
        }));
    }
    let mut app = App::new_with_language(workspace.clone(), home().as_deref(), explicit, locale);

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
    let terminal_tx = tx.clone();
    thread::spawn(move || {
        while let Ok(input) = event::read() {
            if terminal_tx.send(AppEvent::Terminal(input)).is_err() {
                break;
            }
        }
    });

    // Receive signals on a normal thread: terminal I/O and application state
    // must not be touched from an async signal handler.
    let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP])?;
    let signal_handle = signals.handle();
    let signal_tx = tx.clone();
    let signal_thread = thread::spawn(move || {
        for signal in signals.forever() {
            if signal_tx.send(AppEvent::Signal(signal)).is_err() {
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
    if cfg!(debug_assertions)
        && let Some(mode) = env::var_os("RELAY_TUI_TEST_PANIC")
    {
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
    signal_handle.close();
    let _ = signal_thread.join();
    result.map_err(io::Error::other)?;
    Ok(app.signal_exit_code().unwrap_or(0))
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = cli::parse_args(args.clone());
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_arg = args
        .windows(2)
        .find(|pair| pair[0] == "--workspace")
        .map(|pair| PathBuf::from(&pair[1]))
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--workspace=").map(PathBuf::from))
        });
    let workspace = resolve_workspace(workspace_arg.as_deref(), &cwd);
    let explicit = args
        .windows(2)
        .find(|pair| pair[0] == "--lang")
        .and_then(|pair| Language::parse(&pair[1]))
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--lang=").and_then(Language::parse))
        });
    let selected = language::resolve(explicit, read_language(&workspace), &Locale::from_env());
    match parsed {
        Ok(Action::Version) => {
            println!("{}", cli::version());
            ExitCode::SUCCESS
        }
        Ok(Action::Help) => {
            print!("{}", cli::help(selected));
            ExitCode::SUCCESS
        }
        Ok(Action::Run {
            workspace,
            language: explicit,
        }) => match run(workspace, explicit, Locale::from_env()) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("relay-tui: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            let tip = match selected {
                Language::En => "Use --help to see available options.",
                Language::PtBr => "Use --help para ver as opções.",
            };
            eprintln!("relay-tui: {}\n{tip}", error.message(selected));
            ExitCode::from(2)
        }
    }
}
