//! Snapshots of the Histórico view, drawn into a ratatui buffer: the text of the
//! screen and, below it, a map of the colors (see `snapshots/README.md`).
//! `UPDATE_SNAPSHOTS=1 cargo test --test history_snapshots` rewrites them; read
//! the diff before committing, since the files are the review.

use std::fs;
use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::Widget;

use relay_tui::core::{History, RelayFiles, RelayState, derive_state, extract_history};
use relay_tui::nav::{Ctx, Input, Nav};
use relay_tui::theme;
use relay_tui::view::{Freshness, HistoryScreen, detail_extent};

const SPEC_A: &str = ".specs/20260901-001-contrato-do-core.md";
const SPEC_B: &str = ".specs/20261002-001-relay-tui-observador-em-rust.md";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
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

/// A workspace built by hand, with what the levels need to show: a spec with
/// every state of item, a spec of long text, an item with an old record and a
/// correction, the current task with TODO items that have no record, and an
/// item with no spec.
fn files(many_items: usize) -> RelayFiles {
    let filler: String = (1..=many_items)
        .map(|n| format!("- [x] B-{:03} - Item de rotina numero {n} da lista longa (spec: `{SPEC_A}`)\n", 200 + n))
        .collect();
    RelayFiles {
        backlog: format!(
            "# Backlog\n\n\
             - [x] B-001 - Contrato do estado derivado, com um fixture por status (spec: `{SPEC_A}`)\n\
             {filler}\
             - [x] B-010 - Core em Rust passa a suite de conformidade (spec: `{SPEC_B}`)\n\
             - [•] B-011 - View em cartoes cobre todos os estados e larguras (spec: `{SPEC_B}`)\n\
             - [ ] B-012 - Binario observa um workspace real (spec: `{SPEC_B}`) (needs: B-011)\n\
             - [ ] B-013 - Release e documentacao de uso (spec: `{SPEC_B}`)\n\
             - [!] B-014 - Assinatura dos binarios no macOS (spec: `{SPEC_B}`)\n\
             - [ ] B-099 - Tarefa avulsa, sem spec\n"
        ),
        todo: "# Active task: B-011\n\n- [x] T-001 - Cartoes de status\n- [•] T-002 - Todas as larguras\n- [ ] T-003 - Snapshots (needs: T-002)\n".into(),
        handoff: format!(
            "# Handoff\n\n- Status: in_progress\n- Backlog: B-011\n- TODO: T-002\n- Spec: {SPEC_B}\n\
             - Harness: claude-code\n- Updated: 2026-10-02T12:00:00Z\n\n## Objective\nx\n\n## Next step\ny\n"
        ),
        changelog: format!(
            "# Change log\n\n\
             ## 2026-09-01 - T-001 - Tipos do estado derivado\n- Backlog: B-001\n- Spec: {SPEC_A}\n- Result: os tipos espelham o contrato.\n- Evidence: `cargo test`: 12 passed\n\n\
             ## 2026-09-02 - T-002 - Um fixture por status\n- Backlog: B-001\n- Spec: {SPEC_A}\n- Result: sete fixtures em disco, um por status, com o expected.json de cada um.\n  O runner compara os dois lados e lista todas as divergencias de uma vez.\n\
             - Evidence: o runner TypeScript e o Rust passam os sete casos.\n- Criteria: A-001\n- Decisions: o formato e neutro de linguagem.\n\n\
             ## 2026-09-03 - T-002 - Correcao do fixture blocked\n- Backlog: B-001\n- Spec: {SPEC_A}\n- Result: o fixture blocked ganhou o handoff que faltava.\n- Criteria: none\n\n\
             ## 2026-10-02 - T-001 - Cartoes de status\n- Backlog: B-011\n- Spec: {SPEC_B}\n- Result: um cartao por status, com o tom do design system.\n- Evidence: snapshots em 58 e 40 colunas.\n- Criteria: A-003\n- Decisions: none\n"
        ),
        specs: [
            (SPEC_A.to_string(), "# 20260901-001 - Contrato do estado derivado entre o core e a interface\n".to_string()),
            (SPEC_B.to_string(), "# 20261002-001 - relay-tui\n".to_string()),
            (".specs/20260101-001-sem-itens.md".to_string(), "sem titulo no formato\n".to_string()),
        ]
        .into(),
    }
}

struct Fx {
    history: History,
    state: RelayState,
}

impl Fx {
    fn new(many_items: usize) -> Self {
        let f = files(many_items);
        Fx { history: extract_history(&f), state: derive_state(&f) }
    }

    fn ctx(&self) -> Ctx<'_> {
        Ctx {
            history: &self.history,
            ok: match &self.state {
                RelayState::Ok(ok) => Some(ok),
                _ => None,
            },
        }
    }

    /// Histórico after these inputs, from Agora: `Tab` first.
    fn nav(&self, inputs: &[Input]) -> Nav {
        let mut nav = Nav::new();
        nav.handle(Input::Toggle, &self.ctx());
        for i in inputs {
            nav.handle(*i, &self.ctx());
        }
        nav
    }

    fn check(&self, name: &str, nav: &mut Nav, width: u16, height: u16) {
        // The same sizing the app does before it draws.
        nav.set_detail_max(detail_extent(nav, &self.ctx(), width, height));
        let screen = HistoryScreen { workspace: "~/Developer/relay", freshness: Freshness::Fresh, nav, ctx: self.ctx() };
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        (&screen).render(area, &mut buf);
        let actual = snapshot_of(&buf, area);

        let path: PathBuf = root().join("tests/snapshots").join(format!("{name}.txt"));
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            fs::write(&path, &actual).unwrap();
            return;
        }
        let expected = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing snapshot {}; run with UPDATE_SNAPSHOTS=1", path.display()));
        assert!(actual == expected, "snapshot {name} differs\n--- expected\n{expected}\n--- actual\n{actual}");
    }
}

use Input::{Down, Enter};

#[test]
fn the_specs_level() {
    let fx = Fx::new(0);
    for width in [58, 40] {
        fx.check(&format!("hist-specs-{width}"), &mut fx.nav(&[]), width, 16);
    }
    // The second spec selected: the mark and the bold text follow it.
    fx.check("hist-specs-58-second", &mut fx.nav(&[Down]), 58, 16);
}

#[test]
fn the_items_level() {
    let fx = Fx::new(0);
    // 20261002-001, the newest spec, with an item in every state.
    for width in [58, 40] {
        fx.check(&format!("hist-items-{width}"), &mut fx.nav(&[Enter]), width, 16);
    }
    // The group with no spec.
    fx.check("hist-items-nospec-58", &mut fx.nav(&[Down, Down, Down, Enter]), 58, 12);
}

#[test]
fn the_tasks_level() {
    let fx = Fx::new(0);
    // An old item with a correction: the repeated T-002 is a row of its own.
    for width in [58, 40] {
        fx.check(&format!("hist-tasks-{width}"), &mut fx.nav(&[Down, Enter, Enter]), width, 16);
    }
    // The current item: its record, then the TODO items that have none yet.
    fx.check("hist-tasks-current-58", &mut fx.nav(&[Enter, Down, Enter]), 58, 16);
}

#[test]
fn the_detail_level() {
    let fx = Fx::new(0);
    // The record with every field, long enough to wrap.
    for width in [58, 40] {
        fx.check(&format!("hist-detail-{width}"), &mut fx.nav(&[Down, Enter, Enter, Down, Enter]), width, 24);
    }
    // A TODO item with no record.
    fx.check("hist-detail-pending-58", &mut fx.nav(&[Enter, Down, Enter, Down, Enter]), 58, 14);
}

#[test]
fn a_list_and_a_detail_taller_than_the_screen_scroll() {
    let fx = Fx::new(30);
    let mut top = fx.nav(&[Down, Enter]);
    fx.check("hist-items-long-58-top", &mut top, 58, 14);
    let mut middle = fx.nav(&[Down, Enter]);
    for _ in 0..15 {
        middle.handle(Down, &fx.ctx());
    }
    fx.check("hist-items-long-58-middle", &mut middle, 58, 14);
    fx.check("hist-items-long-40-middle", &mut middle, 40, 14);
    let mut bottom = fx.nav(&[Down, Enter]);
    for _ in 0..40 {
        bottom.handle(Down, &fx.ctx());
    }
    fx.check("hist-items-long-58-bottom", &mut bottom, 58, 14);

    // The detail at 40 columns on a short screen, at the top and at the end.
    let mut detail = fx.nav(&[Down, Enter, Enter, Down, Enter]);
    fx.check("hist-detail-40-h12-top", &mut detail, 40, 12);
    for _ in 0..30 {
        detail.handle(Down, &fx.ctx());
    }
    fx.check("hist-detail-40-h12-end", &mut detail, 40, 12);
}

#[test]
fn narrow_and_short_screens_degrade() {
    let fx = Fx::new(0);
    fx.check("hist-specs-30", &mut fx.nav(&[]), 30, 12);
    fx.check("hist-specs-58-h5", &mut fx.nav(&[]), 58, 5);
    fx.check("hist-specs-58-h6", &mut fx.nav(&[]), 58, 6);
}

#[test]
fn an_empty_workspace_and_a_spec_without_items() {
    let history = extract_history(&RelayFiles::default());
    let fx = Fx { history, state: derive_state(&RelayFiles::default()) };
    fx.check("hist-empty-58", &mut fx.nav(&[]), 58, 10);

    // The oldest spec has no backlog items: `0/0`, and an empty items list.
    let fx = Fx::new(0);
    fx.check("hist-items-empty-58", &mut fx.nav(&[Down, Down, Enter]), 58, 10);
}
