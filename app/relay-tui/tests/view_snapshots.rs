//! Snapshots of the screen, drawn into a ratatui buffer.
//!
//! A snapshot has the text of the screen and, below it, a map of the colors
//! (see `snapshots/README.md`). `UPDATE_SNAPSHOTS=1 cargo test` rewrites them;
//! read the diff before committing, since the files are the review.

use std::fs;
use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::Widget;

use relay_tui::core::{
    ChecklistEntry, Handoff, OkState, RelayState, Violation, WorkStatus, derive_state,
};
use relay_tui::theme;
use relay_tui::view::{Freshness, Screen, View};
use relay_tui::view::text::parse_rfc3339;
use relay_tui::workspace::read_workspace;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn conformance_case(name: &str) -> RelayState {
    derive_state(&read_workspace(&root().join("../conformance").join(name).join("workspace")))
}

/// 2026-09-07T14:10:00Z: 10 minutes after the `blocked` handoff and a little
/// over 7 hours after the others.
fn now() -> i64 {
    parse_rfc3339("2026-09-07T14:10:00Z").unwrap()
}

fn color_letter(color: Color, bold: bool) -> char {
    let letter = match color {
        c if c == theme::FG => 'f',
        c if c == theme::META => 'm',
        c if c == theme::GREEN => 'g',
        c if c == theme::BLUE => 'u',
        c if c == theme::YELLOW => 'y',
        c if c == theme::RED => 'r',
        c if c == theme::ID => 'i',
        c if c == theme::DIM => 'd',
        c if c == theme::BAR_EMPTY => 'e',
        _ => '?',
    };
    if bold { letter.to_ascii_uppercase() } else { letter }
}

fn snapshot_of(buf: &Buffer, area: Rect) -> String {
    let mut text = String::new();
    let mut colors = String::new();
    for y in area.top()..area.bottom() {
        let mut row = String::new();
        let mut map = String::new();
        for x in area.left()..area.right() {
            let cell = &buf[(x, y)];
            row.push_str(cell.symbol());
            map.push(if cell.bg != Color::Reset {
                '#'
            } else if cell.symbol() == " " {
                '.'
            } else {
                color_letter(cell.fg, cell.modifier.contains(Modifier::BOLD))
            });
        }
        text.push_str(row.trim_end());
        text.push('\n');
        colors.push_str(map.trim_end_matches('.'));
        colors.push('\n');
    }
    format!("{text}\n--- colors ---\n{colors}")
}

fn check(name: &str, view: &View, width: u16, height: u16) {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    view.render(area, &mut buf);
    let actual = snapshot_of(&buf, area);

    let path: PathBuf = root().join("tests/snapshots").join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing snapshot {}; run with UPDATE_SNAPSHOTS=1", path.display()));
    assert!(
        actual == expected,
        "snapshot {name} differs\n--- expected\n{expected}\n--- actual\n{actual}"
    );
}

fn view<'a>(state: &'a RelayState, freshness: Freshness) -> View<'a> {
    View { history: relay_tui::view::no_history(), screen: Screen::State(state), workspace: "~/Developer/relay", freshness, now_unix: now() }
}

const STATUS_CASES: [&str; 7] = ["idle", "backlog", "ready", "in_progress", "blocked", "done", "inconsistent"];

#[test]
fn every_status_at_58_columns() {
    for case in STATUS_CASES {
        let state = conformance_case(&format!("status-{case}"));
        check(&format!("{case}-58"), &view(&state, Freshness::Fresh), 58, 24);
    }
}

#[test]
fn every_status_at_40_columns() {
    for case in STATUS_CASES {
        let state = conformance_case(&format!("status-{case}"));
        check(&format!("{case}-40"), &view(&state, Freshness::Fresh), 40, 24);
    }
}

#[test]
fn a_change_in_flight_says_so_in_words() {
    let state = conformance_case("status-in_progress");
    check("in_progress-58-updating", &view(&state, Freshness::Updating), 58, 24);
}

#[test]
fn a_directory_without_orchestration() {
    let view = View { history: relay_tui::view::no_history(),
        screen: Screen::NotARelayWorkspace,
        workspace: "~/Developer/sem-relay",
        freshness: Freshness::Fresh,
        now_unix: now(),
    };
    check("not-relay-58", &view, 58, 12);
}

// ------------------------------------------------------- built by hand

fn entry(id: &str, text: &str, marker: char, needs: &[&str], available: bool) -> ChecklistEntry {
    ChecklistEntry {
        id: id.into(),
        text: text.into(),
        marker,
        needs: needs.iter().map(|s| s.to_string()).collect(),
        available,
        spec: None,
    }
}

fn long_todo() -> RelayState {
    let todo = vec![
        entry("T-001", "Formato do caso de conformidade", 'x', &[], false),
        entry("T-002", "Extrair as entradas dos fixtures para workspaces em disco", 'x', &[], false),
        entry("T-003", "Runner TS lê a suíte", 'x', &["T-001"], false),
        entry("T-004", "Runner Rust lê a suíte", '•', &["T-003"], false),
        entry("T-005", "CI quebra em divergência", ' ', &["T-004"], false),
        entry("T-006", "Documentar o formato", ' ', &[], true),
        entry("T-007", "Casos extras para os ramos não cobertos", ' ', &["T-004", "T-006"], false),
        entry("T-008", "Revisão final", '!', &[], false),
        entry("T-009", "Publicar", ' ', &["T-007"], false),
        entry("T-010", "Anunciar", ' ', &["T-009"], false),
    ];
    let backlog = vec![
        entry("B-001", "Contrato", 'x', &[], false),
        entry("B-002", "Core em Rust", '•', &[], false),
        entry("B-003", "Watcher", ' ', &[], true),
        entry("B-004", "View", ' ', &[], true),
        entry("B-005", "Release", ' ', &["B-004"], false),
        entry("B-006", "Bloqueado de verdade", '!', &[], false),
    ];
    RelayState::Ok(OkState {
        status: WorkStatus::InProgress,
        handoff: Some(Handoff {
            backlog_id: "B-002".into(),
            todo_id: "T-004".into(),
            spec: ".specs/20261002-001-x.md".into(),
            harness: "claude-code".into(),
            updated: "2026-09-07T14:05:00Z".into(),
            objective: "Escrever o runner em Rust que lê a suíte de conformidade e compara cada caso com o expected.json, listando todas as divergências de uma vez.".into(),
            next_step: "Rodar cargo test e corrigir as divergências pela mensagem do runner.".into(),
            context: "T-001 a T-003 concluídos.".into(),
        }),
        active_backlog_id: Some("B-002".into()),
        completed: 3,
        total: 10,
        todo,
        backlog,
    })
}

#[test]
fn a_long_todo_is_cut_with_the_count_of_what_is_hidden() {
    let state = long_todo();
    check("long-todo-58-h24", &view(&state, Freshness::Fresh), 58, 24);
    // Not enough height for the whole TODO: the Backlog gives up its frame,
    // then the TODO is cut and still says where the active item went.
    check("long-todo-58-h16", &view(&state, Freshness::Fresh), 58, 16);
    check("long-todo-58-h12", &view(&state, Freshness::Fresh), 58, 12);
    check("long-todo-40-h16", &view(&state, Freshness::Fresh), 40, 16);
}

#[test]
fn several_violations_are_listed_and_cut_by_height() {
    let violations = vec![
        Violation {
            check: "handoff-names-no-pending-todo".into(),
            detail: "O handoff aponta T-001, que está [x]; T-002 está [•].".into(),
            records: vec!["handoff".into(), "todo".into()],
        },
        Violation {
            check: "needs-cycle".into(),
            detail: "Ciclo em todo: T-001 -> T-002 -> T-001.".into(),
            records: vec!["todo".into()],
        },
        Violation {
            check: "criteria-without-evidence".into(),
            detail: "Spec .specs/20260907-001-ui-primeiro-marco-visual.md: critérios sem evidência: A-001, A-002.".into(),
            records: vec!["backlog".into(), "changelog".into(), "spec".into()],
        },
    ];
    let state = RelayState::Inconsistent { violations };
    check("violations-58-h24", &view(&state, Freshness::Fresh), 58, 24);
    check("violations-40-h24", &view(&state, Freshness::Fresh), 40, 24);
    check("violations-58-h11", &view(&state, Freshness::Fresh), 58, 11);
}

#[test]
fn narrow_and_short_screens_degrade_to_the_status() {
    let state = conformance_case("status-blocked");
    check("blocked-30-h10", &view(&state, Freshness::Fresh), 30, 10);
    let bad = conformance_case("status-inconsistent");
    check("inconsistent-30-h10", &view(&bad, Freshness::Fresh), 30, 10);
    check("blocked-58-h5", &view(&state, Freshness::Fresh), 58, 5);
}

#[test]
fn a_tall_or_wide_terminal_shows_the_handoff_nearly_whole() {
    // Room to spare: the fields use it instead of stopping at three lines.
    let long = long_todo();
    check("long-todo-58-h40", &view(&long, Freshness::Fresh), 58, 40);
    let blocked = conformance_case("status-blocked");
    check("blocked-120-h30", &view(&blocked, Freshness::Fresh), 120, 30);
}

#[test]
fn the_next_step_line_yields_with_the_height() {
    let state = conformance_case("status-in_progress");
    for height in [12, 17, 18, 20] {
        check(&format!("in_progress-58-h{height}"), &view(&state, Freshness::Fresh), 58, height);
    }
}
