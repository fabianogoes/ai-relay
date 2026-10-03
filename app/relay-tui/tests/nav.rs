//! The navigation state of Histórico, without a terminal (spec 20261002-002,
//! A-003): inputs in, effects and selection out.

use relay_tui::core::{RelayFiles, RelayState, derive_state, extract_history};
use relay_tui::nav::{Ctx, Effect, Input, Level, Nav, Pane, RowKey};

const SPEC_A: &str = ".specs/20260101-001-a.md";
const SPEC_B: &str = ".specs/20260202-001-b.md";

fn files() -> RelayFiles {
    RelayFiles {
        backlog: format!(
            "# Backlog\n\n\
             - [x] B-001 - Um (spec: `{SPEC_A}`)\n\
             - [x] B-002 - Dois (spec: `{SPEC_A}`)\n\
             - [ ] B-003 - Tres (spec: `{SPEC_B}`)\n\
             - [ ] B-004 - Orfao\n"
        ),
        todo: "# Active task: B-003\n\n- [x] T-001 - Feito\n- [•] T-002 - Em curso\n- [ ] T-003 - Falta\n".into(),
        handoff: format!(
            "# Handoff\n\n- Status: in_progress\n- Backlog: B-003\n- TODO: T-002\n- Spec: {SPEC_B}\n\
             - Harness: claude-code\n- Updated: 2026-02-02T00:00:00Z\n\n## Objective\nx\n\n## Next step\ny\n"
        ),
        changelog: format!(
            "# Change log\n\n\
             ## 2026-01-01 - T-001 - Primeiro\n- Backlog: B-001\n- Spec: {SPEC_A}\n- Result: r1\n- Criteria: none\n\n\
             ## 2026-01-02 - T-001 - Segundo item\n- Backlog: B-002\n- Spec: {SPEC_A}\n- Result: r2\n- Criteria: none\n\n\
             ## 2026-02-01 - T-001 - Feito\n- Backlog: B-003\n- Spec: {SPEC_B}\n- Result: r3\n- Criteria: none\n"
        ),
        specs: [
            (SPEC_A.to_string(), "# 20260101-001 - Primeira\n".to_string()),
            (SPEC_B.to_string(), "# 20260202-001 - Segunda\n".to_string()),
        ]
        .into(),
    }
}

struct Fixture {
    history: relay_tui::core::History,
    state: RelayState,
}

impl Fixture {
    fn new() -> Self {
        let f = files();
        Fixture { history: extract_history(&f), state: derive_state(&f) }
    }

    fn ctx(&self) -> Ctx<'_> {
        Ctx {
            history: &self.history,
            ok: match &self.state {
                RelayState::Ok(ok) => Some(ok),
                RelayState::Inconsistent { .. } => None,
            },
        }
    }
}

fn press(nav: &mut Nav, fx: &Fixture, inputs: &[Input]) -> Vec<Effect> {
    inputs.iter().map(|i| nav.handle(*i, &fx.ctx())).collect()
}

#[test]
fn it_opens_in_agora_and_tab_toggles_without_losing_the_place() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    assert_eq!(nav.pane(), Pane::Now);
    press(&mut nav, &fx, &[Input::Toggle, Input::Down, Input::Enter]);
    assert_eq!((nav.pane(), nav.level()), (Pane::History, Level::Items));
    let at = nav.selected(Level::Specs).cloned();
    // Back to Agora and again to Histórico: same level, same selection.
    press(&mut nav, &fx, &[Input::Toggle]);
    assert_eq!(nav.pane(), Pane::Now);
    press(&mut nav, &fx, &[Input::Toggle]);
    assert_eq!((nav.pane(), nav.level()), (Pane::History, Level::Items));
    assert_eq!(nav.selected(Level::Specs).cloned(), at);
}

#[test]
fn the_specs_level_lists_newest_first_then_the_sem_spec_group() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    nav.handle(Input::Toggle, &fx.ctx());
    let keys: Vec<RowKey> = nav.rows(&fx.ctx(), Level::Specs).into_iter().map(|r| r.key).collect();
    assert_eq!(keys, [RowKey::Spec(SPEC_B.into()), RowKey::Spec(SPEC_A.into()), RowKey::NoSpec]);
    // The first row is selected from the start.
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::Spec(SPEC_B.into())));
}

#[test]
fn enter_goes_down_one_level_at_a_time_and_does_nothing_in_the_detail() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    press(&mut nav, &fx, &[Input::Toggle]);
    let levels: Vec<Level> = [Input::Enter, Input::Enter, Input::Enter, Input::Enter]
        .iter()
        .map(|i| {
            nav.handle(*i, &fx.ctx());
            nav.level()
        })
        .collect();
    assert_eq!(levels, [Level::Items, Level::Tasks, Level::Detail, Level::Detail]);
    // Spec B -> B-003 -> its first task.
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-003".into())));
    assert_eq!(nav.selected(Level::Tasks), Some(&RowKey::Record { todo_id: "T-001".into(), nth: 0 }));
}

#[test]
fn the_tasks_of_the_current_item_end_with_the_todo_items_without_a_record() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    press(&mut nav, &fx, &[Input::Toggle, Input::Enter, Input::Enter]);
    let keys: Vec<RowKey> = nav.rows(&fx.ctx(), Level::Tasks).into_iter().map(|r| r.key).collect();
    assert_eq!(
        keys,
        [
            RowKey::Record { todo_id: "T-001".into(), nth: 0 },
            RowKey::Pending("T-002".into()),
            RowKey::Pending("T-003".into()),
        ]
    );
    // An item that is not the current one has only its records.
    press(&mut nav, &fx, &[Input::Esc, Input::Esc, Input::Down, Input::Enter, Input::Enter]);
    assert_eq!(nav.level(), Level::Tasks);
    let other: Vec<RowKey> = nav.rows(&fx.ctx(), Level::Tasks).into_iter().map(|r| r.key).collect();
    assert_eq!(other, [RowKey::Record { todo_id: "T-001".into(), nth: 0 }]);
}

#[test]
fn esc_and_backspace_go_back_one_level_and_from_the_first_to_agora() {
    let fx = Fixture::new();
    for back in [Input::Esc, Input::Backspace] {
        let mut nav = Nav::new();
        press(&mut nav, &fx, &[Input::Toggle, Input::Enter, Input::Enter, Input::Enter]);
        let levels: Vec<(Pane, Level)> = [back, back, back, back]
            .iter()
            .map(|i| {
                assert_eq!(nav.handle(*i, &fx.ctx()), Effect::None);
                (nav.pane(), nav.level())
            })
            .collect();
        assert_eq!(
            levels,
            [
                (Pane::History, Level::Tasks),
                (Pane::History, Level::Items),
                (Pane::History, Level::Specs),
                (Pane::Now, Level::Specs),
            ]
        );
    }
}

#[test]
fn esc_asks_before_leaving_only_in_agora_and_q_and_ctrl_c_quit_from_anywhere() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    // In Agora `Esc` asks first; it does not leave by itself.
    assert_eq!(nav.handle(Input::Esc, &fx.ctx()), Effect::None);
    assert!(nav.confirming_quit());
    nav.handle(Input::Other, &fx.ctx());
    assert!(!nav.confirming_quit());
    // Backspace and Enter do nothing in Agora.
    assert_eq!(press(&mut nav, &fx, &[Input::Backspace, Input::Enter]), [Effect::None, Effect::None]);
    assert_eq!((nav.pane(), nav.level()), (Pane::Now, Level::Specs));
    for depth in 0..4 {
        let mut nav = Nav::new();
        press(&mut nav, &fx, &[Input::Toggle]);
        for _ in 0..depth {
            nav.handle(Input::Enter, &fx.ctx());
        }
        assert_eq!(nav.handle(Input::Quit, &fx.ctx()), Effect::Quit, "from depth {depth}");
        // Esc, in Histórico, never quits and never asks.
        assert_eq!(nav.handle(Input::Esc, &fx.ctx()), Effect::None);
        assert!(!nav.confirming_quit());
    }
}

#[test]
fn the_question_is_answered_by_esc_enter_or_y_and_cancelled_by_any_other_key() {
    let fx = Fixture::new();
    for yes in [Input::Esc, Input::Enter, Input::Confirm] {
        let mut nav = Nav::new();
        assert_eq!(nav.handle(Input::Esc, &fx.ctx()), Effect::None);
        assert_eq!(nav.handle(yes, &fx.ctx()), Effect::Quit, "{yes:?}");
    }
    // `q` and `Ctrl-C` leave at once, with or without the question.
    let mut nav = Nav::new();
    nav.handle(Input::Esc, &fx.ctx());
    assert_eq!(nav.handle(Input::Quit, &fx.ctx()), Effect::Quit);

    // Any other key cancels, and is consumed: it does not do what it normally
    // does, and the next Esc asks again.
    for no in [Input::Other, Input::Toggle, Input::Reload, Input::Down, Input::Backspace, Input::WheelDown] {
        let mut nav = Nav::new();
        nav.handle(Input::Esc, &fx.ctx());
        assert_eq!(nav.handle(no, &fx.ctx()), Effect::None, "{no:?}");
        assert!(!nav.confirming_quit(), "{no:?}");
        assert_eq!(nav.pane(), Pane::Now, "{no:?} is consumed by the answer");
        assert_eq!(nav.handle(Input::Esc, &fx.ctx()), Effect::None);
        assert!(nav.confirming_quit());
    }
}

#[test]
fn y_and_other_keys_do_nothing_when_nothing_was_asked() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    assert_eq!(press(&mut nav, &fx, &[Input::Confirm, Input::Other]), [Effect::None, Effect::None]);
    nav.handle(Input::Toggle, &fx.ctx());
    assert_eq!(press(&mut nav, &fx, &[Input::Confirm, Input::Other]), [Effect::None, Effect::None]);
    assert_eq!((nav.pane(), nav.level()), (Pane::History, Level::Specs));
}

#[test]
fn arrows_move_the_selection_and_stop_at_the_ends() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    press(&mut nav, &fx, &[Input::Toggle, Input::Up]);
    assert_eq!(nav.selected_index(Level::Specs), 0);
    press(&mut nav, &fx, &[Input::Down, Input::Down, Input::Down, Input::Down]);
    // Three rows (two specs and Sem spec): the last one holds.
    assert_eq!(nav.selected_index(Level::Specs), 2);
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::NoSpec));
    press(&mut nav, &fx, &[Input::Up]);
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::Spec(SPEC_A.into())));
}

#[test]
fn reload_is_an_effect_and_leaves_the_place_alone() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    press(&mut nav, &fx, &[Input::Toggle, Input::Down, Input::Enter]);
    let before = nav.clone();
    assert_eq!(nav.handle(Input::Reload, &fx.ctx()), Effect::Reload);
    assert_eq!(nav, before);
}

#[test]
fn an_empty_list_cannot_be_opened() {
    let empty = RelayFiles::default();
    let history = extract_history(&empty);
    let ctx = Ctx { history: &history, ok: None };
    let mut nav = Nav::new();
    for input in [Input::Toggle, Input::Enter, Input::Down, Input::Up] {
        assert_eq!(nav.handle(input, &ctx), Effect::None);
    }
    assert_eq!((nav.pane(), nav.level()), (Pane::History, Level::Specs));
    assert_eq!(nav.selected(Level::Specs), None);
}

// -- reload by id, wheel, page keys, detail scroll and click (T-002) ----------

impl Fixture {
    fn from(f: RelayFiles) -> Self {
        Fixture { history: extract_history(&f), state: derive_state(&f) }
    }

    fn changed(edit: impl FnOnce(&mut RelayFiles)) -> Self {
        let mut f = files();
        edit(&mut f);
        Fixture::from(f)
    }
}

/// Opens Histórico on the spec at `row`, then goes `down` levels.
fn at(fx: &Fixture, row: usize, down: usize) -> Nav {
    let mut nav = Nav::new();
    nav.handle(Input::Toggle, &fx.ctx());
    for _ in 0..row {
        nav.handle(Input::Down, &fx.ctx());
    }
    for _ in 0..down {
        nav.handle(Input::Enter, &fx.ctx());
    }
    nav
}

#[test]
fn a_selection_is_kept_by_id_when_rows_appear_above_it() {
    let before = Fixture::new();
    let mut nav = at(&before, 1, 1); // spec A, items level
    nav.handle(Input::Down, &before.ctx()); // B-002
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-002".into())));
    // A newer spec appears and A gets a new first item: B-002 moves down.
    let after = Fixture::changed(|f| {
        f.specs.insert(".specs/20260303-001-c.md".into(), "# 20260303-001 - Terceira\n".into());
        f.backlog = f.backlog.replace(
            "- [x] B-001 -",
            &format!("- [x] B-000 - Novo (spec: `{SPEC_A}`)\n- [x] B-001 -"),
        );
    });
    nav.reconcile(&after.ctx());
    assert_eq!(nav.level(), Level::Items);
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::Spec(SPEC_A.into())));
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-002".into())));
    assert_eq!((nav.selected_index(Level::Specs), nav.selected_index(Level::Items)), (2, 2));
}

#[test]
fn a_vanished_item_falls_to_the_nearest_row() {
    let before = Fixture::new();
    let mut nav = at(&before, 1, 1);
    nav.handle(Input::Down, &before.ctx());
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-002".into())));
    // B-002 is gone; B-001 is the nearest row left (the list shrank under it).
    let after = Fixture::changed(|f| {
        f.backlog = f.backlog.replace(&format!("- [x] B-002 - Dois (spec: `{SPEC_A}`)\n"), "");
    });
    nav.reconcile(&after.ctx());
    assert_eq!(nav.level(), Level::Items);
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-001".into())));
    assert_eq!(nav.selected_index(Level::Items), 0);
}

#[test]
fn a_vanished_parent_takes_the_view_up_to_the_first_level_that_exists() {
    let before = Fixture::new();
    // Spec A -> B-001 -> its task, in the detail.
    let mut nav = at(&before, 1, 3);
    assert_eq!(nav.level(), Level::Detail);
    // Spec A's file is deleted: its items fall to Sem spec and no level below
    // the specs list survives.
    let after = Fixture::changed(|f| {
        f.specs.remove(SPEC_A);
    });
    nav.reconcile(&after.ctx());
    assert_eq!(nav.level(), Level::Specs);
    // The selection fell to a row that exists, nearest to where it was.
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::NoSpec));
    assert_eq!(nav.selected_index(Level::Specs), 1);
}

#[test]
fn a_vanished_task_in_the_detail_lands_on_a_neighbour_and_an_empty_list_goes_up() {
    let before = Fixture::new();
    // Spec B -> B-003 -> T-002 (pending, no record yet), in the detail.
    let mut nav = at(&before, 0, 2);
    nav.handle(Input::Down, &before.ctx());
    nav.handle(Input::Enter, &before.ctx());
    assert_eq!(nav.selected(Level::Tasks), Some(&RowKey::Pending("T-002".into())));
    assert_eq!(nav.level(), Level::Detail);
    // T-002 gets its record: the same task is now a Record, found by position.
    let done = Fixture::changed(|f| {
        f.changelog.push_str(&format!(
            "\n## 2026-02-02 - T-002 - Em curso\n- Backlog: B-003\n- Spec: {SPEC_B}\n- Result: r\n- Criteria: none\n"
        ));
    });
    nav.reconcile(&done.ctx());
    assert_eq!(nav.level(), Level::Detail);
    assert_ne!(nav.selected(Level::Tasks), Some(&RowKey::Pending("T-002".into())));
    // B-003 itself disappears: the tasks list is gone, up to the items list.
    let gone = Fixture::changed(|f| {
        f.backlog = f.backlog.replace(&format!("- [ ] B-003 - Tres (spec: `{SPEC_B}`)\n"), "");
    });
    nav.reconcile(&gone.ctx());
    assert!(nav.level() <= Level::Items, "{:?}", nav.level());
}

#[test]
fn reload_survives_inconsistent_records_without_a_todo() {
    let before = Fixture::new();
    let mut nav = at(&before, 0, 2);
    assert_eq!(nav.level(), Level::Tasks);
    // The state turns inconsistent (no `ok`), but the history is still read.
    let inconsistent = Fixture::changed(|f| f.todo = "# Active task: B-999\n\n- [•] T-001 - x\n".into());
    assert!(matches!(inconsistent.state, RelayState::Inconsistent { .. }));
    nav.reconcile(&inconsistent.ctx());
    assert_eq!(nav.level(), Level::Tasks);
    // The pending TODO items are gone with `ok`; the record stays.
    let keys: Vec<RowKey> = nav.rows(&inconsistent.ctx(), Level::Tasks).into_iter().map(|r| r.key).collect();
    assert_eq!(keys, [RowKey::Record { todo_id: "T-001".into(), nth: 0 }]);
}

#[test]
fn the_wheel_moves_the_selection_like_the_arrows_and_stops_at_the_ends() {
    let fx = Fixture::new();
    let mut nav = at(&fx, 0, 0);
    press(&mut nav, &fx, &[Input::WheelUp]);
    assert_eq!(nav.selected_index(Level::Specs), 0);
    press(&mut nav, &fx, &[Input::WheelDown, Input::WheelDown, Input::WheelDown, Input::WheelDown]);
    assert_eq!(nav.selected_index(Level::Specs), 2);
}

#[test]
fn page_keys_move_by_the_page_and_stop_at_the_ends() {
    let backlog: String = (1..=30)
        .map(|n| format!("- [ ] B-{n:03} - Item {n} (spec: `{SPEC_A}`)\n"))
        .collect();
    let fx = Fixture::changed(|f| f.backlog = format!("# Backlog\n\n{backlog}"));
    let mut nav = at(&fx, 1, 1);
    nav.set_page(8);
    press(&mut nav, &fx, &[Input::PageDown]);
    assert_eq!(nav.selected_index(Level::Items), 8);
    press(&mut nav, &fx, &[Input::PageDown, Input::PageDown, Input::PageDown, Input::PageDown]);
    assert_eq!(nav.selected_index(Level::Items), 29);
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-030".into())));
    press(&mut nav, &fx, &[Input::PageUp]);
    assert_eq!(nav.selected_index(Level::Items), 21);
    press(&mut nav, &fx, &[Input::PageUp, Input::PageUp, Input::PageUp]);
    assert_eq!(nav.selected_index(Level::Items), 0);
}

#[test]
fn the_detail_scrolls_within_the_limit_the_view_reports() {
    let fx = Fixture::new();
    let mut nav = at(&fx, 0, 3);
    assert_eq!(nav.level(), Level::Detail);
    nav.set_detail_max(5);
    nav.set_page(3);
    press(&mut nav, &fx, &[Input::Up]);
    assert_eq!(nav.scroll(), 0);
    press(&mut nav, &fx, &[Input::Down, Input::WheelDown, Input::PageDown]);
    assert_eq!(nav.scroll(), 5);
    press(&mut nav, &fx, &[Input::Down]);
    assert_eq!(nav.scroll(), 5);
    press(&mut nav, &fx, &[Input::PageUp]);
    assert_eq!(nav.scroll(), 2);
    // A smaller extent (a wider terminal) pulls the scroll back in range, and
    // leaving the detail resets it.
    nav.set_detail_max(1);
    assert_eq!(nav.scroll(), 1);
    press(&mut nav, &fx, &[Input::Esc]);
    assert_eq!((nav.level(), nav.scroll()), (Level::Tasks, 0));
}

#[test]
fn a_click_on_a_row_selects_it_and_opens_the_next_level() {
    let fx = Fixture::new();
    let mut nav = at(&fx, 0, 0);
    // Spec A is row 1.
    assert_eq!(nav.handle(Input::Click(1), &fx.ctx()), Effect::None);
    assert_eq!(nav.level(), Level::Items);
    assert_eq!(nav.selected(Level::Specs), Some(&RowKey::Spec(SPEC_A.into())));
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-001".into())));
    // B-002 is row 1 of the items.
    nav.handle(Input::Click(1), &fx.ctx());
    assert_eq!(nav.level(), Level::Tasks);
    assert_eq!(nav.selected(Level::Items), Some(&RowKey::Item("B-002".into())));
    nav.handle(Input::Click(0), &fx.ctx());
    assert_eq!(nav.level(), Level::Detail);
}

#[test]
fn a_click_outside_the_rows_or_in_the_detail_or_in_agora_is_ignored() {
    let fx = Fixture::new();
    let mut nav = Nav::new();
    // In Agora.
    nav.handle(Input::Click(0), &fx.ctx());
    assert_eq!((nav.pane(), nav.level()), (Pane::Now, Level::Specs));
    nav.handle(Input::Toggle, &fx.ctx());
    // Past the last row (header, footer and empty area never reach a row).
    let before = nav.clone();
    nav.handle(Input::Click(3), &fx.ctx());
    nav.handle(Input::Click(99), &fx.ctx());
    assert_eq!(nav, before);
    // In the detail.
    let mut nav = at(&fx, 0, 3);
    let before = nav.clone();
    nav.handle(Input::Click(0), &fx.ctx());
    assert_eq!(nav, before);
}
