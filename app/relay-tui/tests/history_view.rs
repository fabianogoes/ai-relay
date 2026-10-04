//! What the Histórico view says, asserted on the drawn text (the snapshots in
//! `history_snapshots.rs` hold the exact screens).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use relay_tui::core::{History, RelayFiles, RelayState, derive_state, extract_history};
use relay_tui::nav::{Ctx, Input, Nav};
use relay_tui::view::{Freshness, HistoryScreen, detail_extent, list_geometry};

const SPEC_A: &str = ".specs/20260101-001-a.md";
const SPEC_B: &str = ".specs/20260202-001-b.md";

fn files(items: usize) -> RelayFiles {
    let backlog: String = (1..=items)
        .map(|n| format!("- [x] B-{n:03} - Item numero {n} com um texto bem comprido para cortar (spec: `{SPEC_A}`)\n"))
        .collect();
    RelayFiles {
        changelogs: Default::default(),
        backlog: format!(
            "# Backlog\n\n{backlog}- [ ] B-100 - Aberto (spec: `{SPEC_B}`)\n- [ ] B-101 - Esperando (spec: `{SPEC_B}`) (needs: B-100)\n- [!] B-102 - Preso (spec: `{SPEC_B}`)\n- [ ] B-103 - Orfao\n"
        ),
        changelog: "# Change log\n\n## 2026-01-01 - T-001 - Primeira tarefa com um titulo muito comprido que nao cabe numa linha\n\
                    - Backlog: B-001\n- Spec: .specs/20260101-001-a.md\n- Result: o resultado tem varias palavras e quebra em mais de uma linha quando a largura e pequena\n  e continua aqui\n\
                    - Evidence: `cargo test`: 3 passed\n- Criteria: A-001, A-002\n- Decisions: none\n\n\
                    ## 2026-01-02 - T-002 - Registro antigo sem criterios\n- Backlog: B-001\n- Result: so isso\n"
            .into(),
        specs: [
            (SPEC_A.to_string(), "# 20260101-001 - A primeira spec, com um titulo que tambem e longo demais\n".to_string()),
            (SPEC_B.to_string(), "# 20260202-001 - Segunda\n".to_string()),
        ]
        .into(),
        ..Default::default()
    }
}

struct Fx {
    history: History,
    state: RelayState,
}

impl Fx {
    fn new(items: usize) -> Self {
        let f = files(items);
        Fx {
            history: extract_history(&f),
            state: derive_state(&f),
        }
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

    fn nav(&self, inputs: &[Input]) -> Nav {
        let mut nav = Nav::new();
        nav.handle(Input::Toggle, &self.ctx());
        for i in inputs {
            nav.handle(*i, &self.ctx());
        }
        nav
    }

    fn draw(&self, nav: &Nav, width: u16, height: u16) -> Vec<String> {
        let screen = HistoryScreen {
            workspace: "~/relay",
            freshness: Freshness::Fresh,
            language: relay_tui::language::Language::PtBr,
            nav,
            ctx: self.ctx(),
        };
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        (&screen).render(area, &mut buf);
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
}

fn joined(lines: &[String]) -> String {
    lines.join("\n")
}

#[test]
fn the_specs_list_has_one_row_per_spec_with_id_title_and_count() {
    let fx = Fx::new(3);
    let nav = fx.nav(&[]);
    let text = joined(&fx.draw(&nav, 58, 16));
    assert!(text.contains("Specs"), "{text}");
    assert!(text.contains("▸ 20260202-001 Segunda"), "{text}");
    assert!(text.contains("0/3"), "{text}");
    assert!(text.contains("20260101-001 A primeira spec"), "{text}");
    assert!(text.contains("3/3"), "{text}");
    assert!(text.contains("Sem spec"), "{text}");
}

#[test]
fn a_long_row_is_cut_with_an_ellipsis_and_never_wraps() {
    let fx = Fx::new(3);
    let mut nav = fx.nav(&[]);
    nav.handle(Input::Down, &fx.ctx());
    for width in [58, 40] {
        let lines = fx.draw(&nav, width, 16);
        let row = lines
            .iter()
            .find(|l| l.contains("20260101-001"))
            .expect("spec A row");
        assert!(row.contains('…'), "{row}");
        assert!(
            row.trim_end_matches('│').trim_end().ends_with("3/3"),
            "the count stays flush right: {row}"
        );
        // The card's right border is still the last column: nothing overflowed.
        assert!(
            row.ends_with('│') && row.chars().count() == width as usize,
            "{row}"
        );
    }
}

#[test]
fn items_show_the_same_markers_and_words_as_agora() {
    let fx = Fx::new(1);
    // Spec B is the first row.
    let nav = fx.nav(&[Input::Enter]);
    let text = joined(&fx.draw(&nav, 58, 16));
    assert!(text.contains("Itens · 20260202-001"), "{text}");
    assert!(text.contains("○ disponível"), "{text}");
    assert!(text.contains("◌ aguardando"), "{text}");
    assert!(text.contains("! bloqueado"), "{text}");
    // A done item, in the other spec.
    let nav = fx.nav(&[Input::Down, Input::Enter]);
    assert!(joined(&fx.draw(&nav, 58, 16)).contains("✓ feito"));
    // The group of items with no spec.
    let nav = fx.nav(&[Input::Down, Input::Down, Input::Enter]);
    let text = joined(&fx.draw(&nav, 58, 16));
    assert!(
        text.contains("Itens · Sem spec") && text.contains("B-103"),
        "{text}"
    );
}

#[test]
fn tasks_list_the_records_of_the_item_with_their_date() {
    let fx = Fx::new(1);
    let nav = fx.nav(&[Input::Down, Input::Enter, Input::Enter]);
    let text = joined(&fx.draw(&nav, 58, 16));
    assert!(text.contains("Tarefas · B-001"), "{text}");
    assert!(text.contains("T-001"), "{text}");
    assert!(
        text.contains("2026-01-01") && text.contains("2026-01-02"),
        "{text}"
    );
}

#[test]
fn a_list_longer_than_the_screen_scrolls_and_says_how_much_is_hidden() {
    let fx = Fx::new(30);
    let nav = fx.nav(&[Input::Down, Input::Enter]);
    let (_, rows) = list_geometry(58, 14).unwrap();
    let top = joined(&fx.draw(&nav, 58, 14));
    assert!(top.contains("B-001") && !top.contains("B-030"), "{top}");
    assert!(top.contains(&format!("↓ {} abaixo", 30 - rows)), "{top}");
    assert!(!top.contains("acima"), "{top}");
    // Down to the last item: the selection stays on screen.
    let mut nav = nav;
    for _ in 0..40 {
        nav.handle(Input::Down, &fx.ctx());
    }
    let bottom = joined(&fx.draw(&nav, 58, 14));
    assert!(bottom.contains("▸ B-030"), "{bottom}");
    assert!(
        bottom.contains(&format!("↑ {} acima", 30 - rows)),
        "{bottom}"
    );
    assert!(!bottom.contains("abaixo"), "{bottom}");
    // In the middle of the list with 40 columns both counts do not fit in the
    // words: the numbers alone.
    let mut nav = fx.nav(&[Input::Down, Input::Enter]);
    for _ in 0..12 {
        nav.handle(Input::Down, &fx.ctx());
    }
    let narrow = joined(&fx.draw(&nav, 40, 14));
    assert!(narrow.contains('↑') && narrow.contains('↓'), "{narrow}");
    assert!(
        !narrow.contains("acima") && !narrow.contains("abaixo"),
        "{narrow}"
    );
}

#[test]
fn the_detail_wraps_without_cutting_and_leaves_out_an_absent_field() {
    let fx = Fx::new(1);
    let nav = fx.nav(&[Input::Down, Input::Enter, Input::Enter, Input::Enter]);
    for width in [58u16, 40] {
        let text = joined(&fx.draw(&nav, width, 40));
        assert!(!text.contains('…'), "the detail never cuts: {text}");
        for word in [
            "muito", "comprido", "nao", "cabe", "numa", "linha", "continua", "aqui",
        ] {
            assert!(text.contains(word), "{word} lost at {width}: {text}");
        }
        for label in [
            "Backlog",
            "Spec",
            "Result",
            "Evidence",
            "Criteria",
            "Decisions",
        ] {
            assert!(text.contains(label), "{label}: {text}");
        }
    }
    // The old record has no Criteria, Decisions, Evidence or Spec: not invented.
    let nav = fx.nav(&[
        Input::Down,
        Input::Enter,
        Input::Enter,
        Input::Down,
        Input::Enter,
    ]);
    let text = joined(&fx.draw(&nav, 58, 40));
    assert!(
        text.contains("Result") && text.contains("so isso"),
        "{text}"
    );
    for label in ["Criteria", "Decisions", "Evidence"] {
        assert!(!text.contains(label), "{label} invented: {text}");
    }
}

#[test]
fn a_taller_detail_than_the_screen_scrolls_to_its_end() {
    let fx = Fx::new(1);
    let mut nav = fx.nav(&[Input::Down, Input::Enter, Input::Enter, Input::Enter]);
    let (width, height) = (40u16, 12u16);
    let max = detail_extent(&nav, &fx.ctx(), width, height);
    assert!(max > 0);
    nav.set_detail_max(max);
    let first = joined(&fx.draw(&nav, width, height));
    assert!(
        first.contains("↓") && !first.contains("Decisions"),
        "{first}"
    );
    for _ in 0..max + 3 {
        nav.handle(Input::Down, &fx.ctx());
    }
    let last = joined(&fx.draw(&nav, width, height));
    assert!(
        last.contains("Decisions") && last.contains("none"),
        "{last}"
    );
    assert!(last.contains('↑') && !last.contains('↓'), "{last}");
}

#[test]
fn below_40_columns_only_the_notice_is_drawn_and_below_6_rows_only_header_and_footer() {
    let fx = Fx::new(3);
    let nav = fx.nav(&[]);
    // 31 columns of text on 30: it wraps by words and loses none of them.
    let text = joined(&fx.draw(&nav, 30, 12));
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("Histórico precisa de 40 colunas"), "{text}");
    assert!(!text.contains('…'), "{text}");
    let wide = joined(&fx.draw(&nav, 32, 12));
    assert!(wide.contains("Histórico precisa de 40 colunas"), "{wide}");
    assert!(!text.contains("Specs"), "{text}");
    let short = fx.draw(&nav, 58, 5);
    assert!(short[0].contains("relay"), "{short:?}");
    assert!(short[1..4].iter().all(|l| l.is_empty()), "{short:?}");
}

#[test]
fn an_empty_workspace_says_there_is_no_spec() {
    let empty = RelayFiles::default();
    let history = extract_history(&empty);
    let ctx = Ctx {
        history: &history,
        ok: None,
    };
    let mut nav = Nav::new();
    nav.handle(Input::Toggle, &ctx);
    let screen = HistoryScreen {
        workspace: "~/x",
        freshness: Freshness::Fresh,
        language: relay_tui::language::Language::PtBr,
        nav: &nav,
        ctx,
    };
    let area = Rect::new(0, 0, 58, 12);
    let mut buf = Buffer::empty(area);
    (&screen).render(area, &mut buf);
    let text: String = (0..12)
        .map(|y| (0..58).map(|x| buf[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Nenhuma spec em .specs/"), "{text}");
}

// -- footers (T-002) ----------------------------------------------------------

use relay_tui::view::{Screen, View};

/// The footer's text: on the last row of a short screen, inside the footer card
/// (the row above its bottom border) on a taller one.
fn footer_of(lines: &[String]) -> String {
    let last = lines.last().unwrap();
    let row = if last.starts_with('╰') {
        &lines[lines.len() - 2]
    } else {
        last
    };
    row.trim_matches(|c| c == '│' || c == ' ').to_string()
}

fn agora_footer(width: u16) -> String {
    let state = derive_state(&files(1));
    let view = View {
        history: relay_tui::view::no_history(),
        screen: Screen::State(&state),
        workspace: "~/relay",
        freshness: Freshness::Fresh,
        now_unix: 0,
        language: relay_tui::language::Language::PtBr,
    };
    let area = Rect::new(0, 0, width, 24);
    let mut buf = Buffer::empty(area);
    (&view).render(area, &mut buf);
    let lines: Vec<String> = (0..24)
        .map(|y| {
            (0..width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect();
    footer_of(&lines)
}

#[test]
fn the_footer_of_agora_is_whole_wide_and_cut_by_whole_hints_narrow() {
    assert_eq!(
        agora_footer(80),
        "Tab histórico · r recarregar · c config. · q sair"
    );
    assert_eq!(
        agora_footer(58),
        "Tab histórico · r recarregar · c config. · q sair"
    );
    // The whole line is 37 columns; the cards' content at 40 is 36.
    assert_eq!(agora_footer(40), "Tab histórico · c config. · q sair");
    assert_eq!(agora_footer(30), "c config. · q sair");
}

#[test]
fn the_footer_of_the_lists_has_the_text_of_the_design_system() {
    let fx = Fx::new(3);
    let nav = fx.nav(&[]);
    assert_eq!(
        footer_of(&fx.draw(&nav, 80, 16)),
        "↑↓ mover · Enter abrir · Esc voltar · Tab agora · c config. · q sair"
    );
    // Configuration stays available while less important hints are dropped.
    assert_eq!(
        footer_of(&fx.draw(&nav, 58, 16)),
        "Esc voltar · Tab agora · c config. · q sair"
    );
    assert_eq!(
        footer_of(&fx.draw(&nav, 40, 16)),
        "Tab agora · c config. · q sair"
    );
    assert_eq!(footer_of(&fx.draw(&nav, 30, 12)), "c config. · q sair");
}

#[test]
fn the_footer_of_the_detail_says_scroll_and_has_no_enter() {
    let fx = Fx::new(1);
    let nav = fx.nav(&[Input::Down, Input::Enter, Input::Enter, Input::Enter]);
    assert_eq!(
        footer_of(&fx.draw(&nav, 80, 16)),
        "↑↓ rolar · Esc voltar · Tab agora · r recarregar · c config. · q sair"
    );
    assert_eq!(
        footer_of(&fx.draw(&nav, 58, 16)),
        "↑↓ rolar · Esc voltar · Tab agora · c config. · q sair"
    );
    assert_eq!(
        footer_of(&fx.draw(&nav, 40, 16)),
        "Tab agora · c config. · q sair"
    );
    assert!(!footer_of(&fx.draw(&nav, 80, 16)).contains("Enter"));
}

#[test]
fn q_sair_is_never_cut_even_on_a_tiny_terminal() {
    let fx = Fx::new(3);
    let nav = fx.nav(&[]);
    for width in [12u16, 9, 7] {
        assert!(
            footer_of(&fx.draw(&nav, width, 8)).ends_with("q sair"),
            "width {width}"
        );
    }
}
