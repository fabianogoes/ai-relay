//! Agora grouped by spec (spec 20261002-002, A-007): the current spec with its
//! open items and the other specs that still have work, drawn from a workspace
//! built by hand.

use std::fs;
use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::Widget;

use relay_tui::core::{History, RelayFiles, RelayState, derive_state, extract_history};
use relay_tui::theme;
use relay_tui::view::text::parse_rfc3339;
use relay_tui::view::{Freshness, Screen, View};

const SPEC_A: &str = ".specs/20260901-001-contrato-do-core.md";
const SPEC_B: &str = ".specs/20261002-001-relay-tui-observador-em-rust.md";
const SPEC_C: &str = ".specs/20261002-002-navegacao-pelo-historico.md";
const SPEC_D: &str = ".specs/20261002-003-skill-que-abre-em-split.md";

fn specs() -> std::collections::BTreeMap<String, String> {
    [
        (
            SPEC_A,
            "# 20260901-001 - Contrato do estado derivado entre o core e a interface\n",
        ),
        (
            SPEC_B,
            "# 20261002-001 - relay-tui: observador de terminal em Rust\n",
        ),
        (
            SPEC_C,
            "# 20261002-002 - relay-tui: navegacao pelo historico do trabalho\n",
        ),
        (
            SPEC_D,
            "# 20261002-003 - Skill que abre o relay-tui em um split\n",
        ),
    ]
    .into_iter()
    .map(|(p, t)| (p.to_string(), t.to_string()))
    .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    InProgress,
    Blocked,
    Ready,
    /// State `backlog` with an available entry: nothing active.
    Next,
    /// State `backlog` where every open entry waits for a blocked one.
    NoneAvailable,
}

/// The backlog: spec A all done, spec B with two items pending (one done), spec C
/// the one in progress (every kind of state), spec D with one item, and one item
/// with no spec.
fn files(mode: Mode) -> RelayFiles {
    let working = !matches!(mode, Mode::Next | Mode::NoneAvailable);
    let c_active = if working { "•" } else { " " };
    let backlog = if mode == Mode::NoneAvailable {
        format!(
            "# Backlog\n\n\
             - [x] B-001 - Contrato do estado derivado (spec: `{SPEC_A}`)\n\
             - [!] B-011 - Release e documentacao de uso (spec: `{SPEC_B}`)\n\
             - [ ] B-012 - Assinatura dos binarios no macOS (spec: `{SPEC_B}`) (needs: B-011)\n\
             - [ ] B-042 - Estado da navegacao sem terminal (spec: `{SPEC_C}`) (needs: B-011)\n\
             - [ ] B-045 - Script open-split.sh com deteccao do terminal (spec: `{SPEC_D}`) (needs: B-011)\n\
             - [ ] B-099 - Tarefa avulsa, sem spec (needs: B-011)\n"
        )
    } else {
        format!(
            "# Backlog\n\n\
             - [x] B-001 - Contrato do estado derivado (spec: `{SPEC_A}`)\n\
             - [x] B-010 - Core em Rust passa a suite (spec: `{SPEC_B}`)\n\
             - [ ] B-011 - Release e documentacao de uso (spec: `{SPEC_B}`)\n\
             - [!] B-012 - Assinatura dos binarios no macOS (spec: `{SPEC_B}`)\n\
             - [x] B-040 - ADR-0010 e a visao Historico (spec: `{SPEC_C}`)\n\
             - [{c_active}] B-041 - Core em Rust extrai specs, backlog e tarefas dos registros (spec: `{SPEC_C}`)\n\
             - [ ] B-042 - Estado da navegacao sem terminal (spec: `{SPEC_C}`) (needs: B-041)\n\
             - [ ] B-044 - Binario com captura de mouse (spec: `{SPEC_C}`)\n\
             - [ ] B-045 - Script open-split.sh com deteccao do terminal (spec: `{SPEC_D}`)\n\
             - [ ] B-099 - Tarefa avulsa, sem spec\n"
        )
    };
    let todo = match mode {
        Mode::InProgress | Mode::Blocked => {
            "# Active task: B-041\n\n- [x] T-001 - Extrair titulos\n- [•] T-002 - Agrupar por spec\n- [ ] T-003 - Testar\n"
        }
        Mode::Ready => {
            "# Active task: B-041\n\n- [x] T-001 - Extrair titulos\n- [ ] T-002 - Agrupar por spec\n- [ ] T-003 - Testar\n"
        }
        _ => "",
    };
    let status = if mode == Mode::Blocked {
        "blocked"
    } else {
        "in_progress"
    };
    RelayFiles {
        changelogs: Default::default(),
        backlog,
        todo: todo.to_string(),
        handoff: if matches!(mode, Mode::InProgress | Mode::Blocked) {
            format!(
                "# Handoff\n\n- Status: {status}\n- Backlog: B-041\n- TODO: T-002\n- Spec: {SPEC_C}\n\
                 - Harness: claude-code\n- Updated: 2026-10-02T12:00:00Z\n\n## Objective\nAgrupar.\n\n## Next step\nTestar.\n\n## Context\nAguardando o registro de uma dependencia.\n"
            )
        } else {
            String::new()
        },
        changelog: if working {
            format!(
                "# Change log\n\n## 2026-10-02 - T-001 - Extrair titulos\n- Backlog: B-041\n- Spec: {SPEC_C}\n- Result: r\n- Criteria: none\n"
            )
        } else {
            String::new()
        },
        specs: specs(),
    }
}

struct World {
    state: RelayState,
    history: History,
}

fn world_mode(mode: Mode) -> World {
    let f = files(mode);
    World {
        state: derive_state(&f),
        history: extract_history(&f),
    }
}

fn world(active: bool) -> World {
    world_mode(if active { Mode::InProgress } else { Mode::Next })
}

fn draw(w: &World, width: u16, height: u16) -> Vec<String> {
    let view = View {
        screen: Screen::State(&w.state),
        history: &w.history,
        workspace: "~/Developer/relay",
        freshness: Freshness::Fresh,
        now_unix: parse_rfc3339("2026-10-02T12:05:00Z").unwrap(),
        language: relay_tui::language::Language::PtBr,
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    (&view).render(area, &mut buf);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn text(lines: &[String]) -> String {
    lines.join("\n")
}

#[test]
fn the_current_spec_card_names_the_spec_counts_and_lists_the_open_items() {
    let w = world(true);
    let t = text(&draw(&w, 58, 40));
    // The title on the border: id and title; on the right the word and the count.
    assert!(t.contains("╭ 20261002-002 relay-tui: navegacao"), "{t}");
    assert!(t.contains("em curso · 1/4"), "{t}");
    // Open items with their word; the done one only counts.
    assert!(t.contains("● B-041") && t.contains("em curso"), "{t}");
    assert!(t.contains("◌ B-042") && t.contains("aguardando"), "{t}");
    assert!(t.contains("após B-041"), "{t}");
    assert!(t.contains("○ B-044") && t.contains("disponível"), "{t}");
    assert!(!t.contains("B-040"), "a done item is not listed: {t}");
}

#[test]
fn the_pending_specs_follow_the_backlog_with_their_counts_and_the_sem_spec_line_last() {
    let w = world(true);
    let lines = draw(&w, 58, 40);
    let t = text(&lines);
    assert!(t.contains("╭ Specs pendentes"), "{t}");
    let b = lines
        .iter()
        .position(|l| l.contains("20261002-001"))
        .expect("spec B listed");
    let d = lines
        .iter()
        .position(|l| l.contains("20261002-003"))
        .expect("spec D listed");
    let none = lines
        .iter()
        .position(|l| l.contains("Sem spec"))
        .expect("Sem spec listed");
    assert!(b < d && d < none, "{t}");
    assert!(
        lines[b].contains("1/3") && lines[d].contains("0/1") && lines[none].contains("0/1"),
        "{t}"
    );
    // A spec with everything done is not pending, and the current is not repeated.
    assert!(!t.contains("20260901-001"), "{t}");
    assert_eq!(t.matches("20261002-002").count(), 1, "{t}");
    // The old Backlog card is gone.
    assert!(!t.contains("╭ Backlog"), "{t}");
}

#[test]
fn without_an_active_item_the_card_says_a_seguir_and_names_no_priority() {
    let w = world(false);
    assert!(
        matches!(&w.state, RelayState::Ok(ok) if ok.status == relay_tui::core::WorkStatus::Backlog)
    );
    let t = text(&draw(&w, 58, 40));
    assert!(t.contains("● A escolher"), "{t}");
    // B-011 is the first available entry, in spec B.
    assert!(t.contains("╭ 20261002-001"), "{t}");
    assert!(t.contains("a seguir · 1/3"), "{t}");
    assert!(!t.contains("em curso ·"), "{t}");
    assert!(t.contains("╭ Specs pendentes"), "{t}");
}

#[test]
fn rows_and_titles_are_cut_with_an_ellipsis_at_40_columns() {
    let w = world(true);
    for width in [58u16, 40] {
        for line in draw(&w, width, 40) {
            assert!(line.chars().count() <= width as usize, "{line}");
        }
    }
    let t = text(&draw(&w, 40, 40));
    assert!(t.contains('…'), "{t}");
}

/// Which form the backlog area has: 0 both cards whole, 1 Specs pendentes as a
/// line, 2 Spec atual cut in `+N itens`, 3 the count line of the backlog.
fn stage(lines: &[String]) -> Option<u8> {
    let t = text(lines);
    if t.contains("╭ Specs pendentes") {
        Some(0)
    } else if t.contains("╭ 20261002-002") && t.contains("specs pendentes") {
        let cut = lines
            .iter()
            .any(|l| l.starts_with("│ +") && l.contains("iten"));
        Some(if cut { 2 } else { 1 })
    } else if lines.iter().any(|l| l.trim_start().starts_with("Backlog ")) {
        Some(3)
    } else {
        None
    }
}

fn todo_is_cut(lines: &[String]) -> bool {
    let Some(top) = lines.iter().position(|l| l.starts_with("╭ TODO")) else {
        return true;
    };
    let end = lines[top..]
        .iter()
        .position(|l| l.starts_with('╰'))
        .map_or(lines.len(), |e| top + e);
    lines[top..end]
        .iter()
        .any(|l| l.starts_with("│ +") && l.contains("iten"))
}

fn handoff_is_compact(lines: &[String]) -> bool {
    let Some(top) = lines.iter().position(|l| l.starts_with("╭ Handoff")) else {
        return false;
    };
    !lines[top + 2]
        .trim_matches(|c| c == '│' || c == ' ')
        .is_empty()
}

#[test]
fn the_backlog_area_gives_way_in_the_order_of_the_design_system() {
    let w = world(true);
    let mut seen = Vec::new();
    let mut last = 0;
    for height in (14..=44).rev() {
        let lines = draw(&w, 58, height);
        let Some(now) = stage(&lines) else {
            // Past the count line the area is gone and the TODO is being cut.
            assert!(
                todo_is_cut(&lines) || handoff_is_compact(&lines),
                "height {height}:\n{}",
                text(&lines)
            );
            continue;
        };
        assert!(
            now >= last,
            "stage went back from {last} to {now} at height {height}"
        );
        last = now;
        if !seen.contains(&now) {
            seen.push(now);
        }
        // The Handoff is compacted and the TODO cut only after the area is down
        // to the count line.
        if now < 3 {
            assert!(
                !todo_is_cut(&lines),
                "the TODO was cut at height {height} in stage {now}"
            );
            assert!(
                !handoff_is_compact(&lines),
                "the Handoff was compact at height {height} in stage {now}"
            );
        }
    }
    assert_eq!(seen, [0, 1, 2, 3], "every step of the order must appear");
}

#[test]
fn a_cut_current_spec_still_shows_an_item_and_says_how_many_are_hidden() {
    let w = world(true);
    for height in 20..=44 {
        let lines = draw(&w, 58, height);
        if stage(&lines) == Some(2) {
            let t = text(&lines);
            assert!(
                t.contains("● B-041") && t.contains("+2 itens"),
                "height {height}:\n{t}"
            );
            return;
        }
    }
    panic!("the cut stage never appeared");
}

// -- snapshots ----------------------------------------------------------------
// The text of the screen and, below it, a map of the colors (see
// `snapshots/README.md`). `UPDATE_SNAPSHOTS=1 cargo test --test agora_specs`
// rewrites them; read the diff before committing.

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
    if bold {
        letter.to_ascii_uppercase()
    } else {
        letter
    }
}

fn snapshot(w: &World, name: &str, width: u16, height: u16) {
    let view = View {
        screen: Screen::State(&w.state),
        history: &w.history,
        workspace: "~/Developer/relay",
        freshness: Freshness::Fresh,
        now_unix: parse_rfc3339("2026-10-02T12:05:00Z").unwrap(),
        language: relay_tui::language::Language::PtBr,
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    (&view).render(area, &mut buf);
    let mut out = String::new();
    let mut colors = String::new();
    for y in 0..height {
        let mut row = String::new();
        let mut map = String::new();
        for x in 0..width {
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
        out.push_str(row.trim_end());
        out.push('\n');
        colors.push_str(map.trim_end_matches('.'));
        colors.push('\n');
    }
    let actual = format!("{out}\n--- colors ---\n{colors}");
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing snapshot {}; run with UPDATE_SNAPSHOTS=1",
            path.display()
        )
    });
    assert!(
        actual == expected,
        "snapshot {name} differs\n--- expected\n{expected}\n--- actual\n{actual}"
    );
}

#[test]
fn the_work_states_show_the_current_spec_and_the_pending_ones() {
    let in_progress = world_mode(Mode::InProgress);
    snapshot(&in_progress, "agora-specs-in_progress-58", 58, 40);
    snapshot(&in_progress, "agora-specs-in_progress-40", 40, 40);
    snapshot(&world_mode(Mode::Blocked), "agora-specs-blocked-58", 58, 40);
    snapshot(&world_mode(Mode::Ready), "agora-specs-ready-58", 58, 40);
    snapshot(&world_mode(Mode::Ready), "agora-specs-ready-40", 40, 40);
}

#[test]
fn the_backlog_state_says_a_seguir_and_without_an_available_entry_has_no_current_card() {
    let next = world_mode(Mode::Next);
    snapshot(&next, "agora-specs-backlog-58", 58, 30);
    snapshot(&next, "agora-specs-backlog-40", 40, 30);
    let none = world_mode(Mode::NoneAvailable);
    snapshot(&none, "agora-specs-backlog-none-58", 58, 30);
    let t = text(&draw(&none, 58, 30));
    assert!(
        t.contains("╭ Specs pendentes") && !t.contains("a seguir"),
        "{t}"
    );
}

#[test]
fn the_height_makes_the_area_give_way_in_order() {
    let w = world_mode(Mode::InProgress);
    // Both cards whole, the pending ones as a line, the current spec cut, and
    // the count line of the backlog.
    for height in [34, 27, 25, 23] {
        snapshot(
            &w,
            &format!("agora-specs-in_progress-58-h{height}"),
            58,
            height,
        );
    }
    // In the backlog state, where the same order applies without Handoff or TODO.
    let next = world_mode(Mode::Next);
    for height in [17, 16, 15, 11] {
        snapshot(
            &next,
            &format!("agora-specs-backlog-58-h{height}"),
            58,
            height,
        );
    }
}

// A dropped entry is not work to do (DESIGN.md, "Specs fechadas e itens
// descartados"): Agora neither lists nor counts it, and a spec with nothing
// open is not pending.
fn world_of(backlog: &str) -> World {
    let f = RelayFiles {
        backlog: backlog.to_string(),
        specs: specs(),
        ..Default::default()
    };
    World {
        state: derive_state(&f),
        history: extract_history(&f),
    }
}

#[test]
fn agora_does_not_show_or_count_dropped_entries() {
    let w = world_of(&format!(
        "# Backlog\n\n\
         - [x] B-001 - Contrato do estado derivado (spec: `{SPEC_A}`)\n\
         - [ ] B-011 - Release e documentacao de uso (spec: `{SPEC_B}`)\n\
         - [-] B-012 - Assinatura dos binarios no macOS (spec: `{SPEC_B}`) (dropped: fora de escopo)\n\
         - [-] B-045 - Script open-split.sh com deteccao do terminal (spec: `{SPEC_D}`) (dropped: outra spec)\n"
    ));
    for width in [58, 40] {
        let screen = text(&draw(&w, width, 24));
        assert!(screen.contains("B-011"), "{screen}");
        for hidden in [
            "B-012",
            "B-045",
            "descartado",
            "Skill que abre",
            "fora de escopo",
        ] {
            assert!(!screen.contains(hidden), "{hidden} at {width}\n{screen}");
        }
    }
    // The current spec counts the one entry that is work: 0/1, not 0/2.
    let wide = text(&draw(&w, 58, 24));
    assert!(wide.contains("0/1") && !wide.contains("0/2"), "{wide}");
}

#[test]
fn everything_done_or_dropped_is_done_and_counts_only_the_work() {
    let w = world_of(&format!(
        "# Backlog\n\n\
         - [x] B-001 - Contrato do estado derivado (spec: `{SPEC_A}`)\n\
         - [-] B-012 - Assinatura dos binarios no macOS (spec: `{SPEC_B}`) (dropped: fora de escopo)\n"
    ));
    let screen = text(&draw(&w, 58, 12));
    assert!(
        screen.contains("Concluído") && screen.contains("1 de 1 itens"),
        "{screen}"
    );
}
