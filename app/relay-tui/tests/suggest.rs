//! The suggestion table of spec 20261002-004, case by case.
//!
//! The seven status fixtures of `app/conformance/` give the real states; the
//! four cases the status alone does not name are built from record text.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use relay_tui::core::{RelayFiles, RelayState, derive_state};
use relay_tui::suggest::{Case, Suggestion, suggest};

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance")
}

fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn fixture(status: &str) -> RelayState {
    let orchestration = conformance_dir().join(format!("status-{status}/workspace/.orchestration"));
    derive_state(&RelayFiles {
        backlog: read_or_empty(&orchestration.join("BACKLOG.md")),
        todo: read_or_empty(&orchestration.join("TODO.md")),
        handoff: read_or_empty(&orchestration.join("HANDOFF.md")),
        changelog: read_or_empty(&orchestration.join("CHANGELOG.md")),
        specs: BTreeMap::new(),
    })
}

fn synthetic(backlog: &str, todo: &str) -> RelayState {
    synthetic_with_changelog(backlog, todo, "")
}

fn synthetic_with_changelog(backlog: &str, todo: &str, changelog: &str) -> RelayState {
    derive_state(&RelayFiles {
        backlog: backlog.to_string(),
        todo: todo.to_string(),
        changelog: changelog.to_string(),
        ..RelayFiles::default()
    })
}

fn shape(s: &Suggestion) -> (Case, &'static str, Option<&str>, Option<&str>, Option<&str>) {
    (s.case, s.skill, s.todo_id.as_deref(), s.backlog_id.as_deref(), s.title.as_deref())
}

#[test]
fn a_directory_without_a_workspace_suggests_the_installer() {
    let s = suggest(None);
    assert_eq!(shape(&s), (Case::NotAWorkspace, "relay-setup", None, None, None));
}

#[test]
fn inconsistent_records_point_to_status_without_naming_ids() {
    let state = fixture("inconsistent");
    assert!(matches!(state, RelayState::Inconsistent { .. }));
    assert_eq!(shape(&suggest(Some(&state))), (Case::Inconsistent, "relay-status", None, None, None));
}

#[test]
fn in_progress_resumes_the_handoff_subtask() {
    let state = fixture("in_progress");
    let RelayState::Ok(ok) = &state else { panic!("in_progress fixture is not ok") };
    let h = ok.handoff.as_ref().unwrap();
    let s = suggest(Some(&state));
    assert_eq!(s.case, Case::InProgress);
    assert_eq!(s.skill, "relay-session");
    assert_eq!(s.todo_id.as_deref(), Some(h.todo_id.as_str()));
    assert_eq!(s.backlog_id.as_deref(), Some(h.backlog_id.as_str()));
    assert_eq!(s.title, None);
}

#[test]
fn blocked_with_a_handoff_resolves_the_blocker_then_resumes() {
    let state = fixture("blocked");
    let RelayState::Ok(ok) = &state else { panic!("blocked fixture is not ok") };
    let h = ok.handoff.as_ref().expect("the blocked fixture carries a handoff");
    let s = suggest(Some(&state));
    assert_eq!(s.case, Case::BlockedWithHandoff);
    assert_eq!(s.skill, "relay-session");
    assert_eq!(s.todo_id.as_deref(), Some(h.todo_id.as_str()));
    assert_eq!(s.backlog_id.as_deref(), Some(h.backlog_id.as_str()));
}

#[test]
fn blocked_without_a_handoff_sends_to_continue_with_the_backlog_item() {
    let state = synthetic(
        "# Backlog\n\n- [•] B-001 - Item\n",
        "# Active task: B-001\n\n- [!] T-001 - Primeiro\n- [ ] T-002 - Segundo (needs: T-001)\n",
    );
    let RelayState::Ok(ok) = &state else { panic!("synthetic blocked state is inconsistent") };
    assert!(ok.handoff.is_none());
    assert!(ok.todo.iter().all(|e| !e.available));
    let s = suggest(Some(&state));
    assert_eq!(shape(&s), (Case::BlockedWithoutHandoff, "relay-continue", None, Some("B-001"), None));
}

#[test]
fn ready_names_the_first_available_todo_item_and_its_title() {
    let state = fixture("ready");
    let RelayState::Ok(ok) = &state else { panic!("ready fixture is not ok") };
    let first = ok.todo.iter().find(|e| e.available).unwrap();
    let s = suggest(Some(&state));
    assert_eq!(s.case, Case::Ready);
    assert_eq!(s.skill, "relay-session");
    assert_eq!(s.todo_id.as_deref(), Some(first.id.as_str()));
    assert_eq!(s.title.as_deref(), Some(first.text.as_str()));
    assert_eq!(s.backlog_id, ok.active_backlog_id);
    // The title is the entry text, without `needs:` and `spec:` annotations.
    assert!(!s.title.unwrap().contains("(needs"));
}

#[test]
fn a_finished_todo_with_an_open_backlog_item_asks_to_close_it() {
    let state = synthetic_with_changelog(
        "# Backlog\n\n- [•] B-001 - Item\n",
        "# Active task: B-001\n\n- [x] T-001 - Feito\n",
        "# Change log\n\n## 2026-10-03 - T-001 - Feito\n- Backlog: B-001\n- Spec: .specs/x.md\n- Result: r\n- Evidence: e\n- Criteria: none\n- Decisions: none\n",
    );
    let s = suggest(Some(&state));
    assert_eq!(shape(&s), (Case::DoneWithTodo, "relay-session", None, Some("B-001"), None));
}

#[test]
fn backlog_names_the_first_available_item_in_textual_order() {
    let state = fixture("backlog");
    let RelayState::Ok(ok) = &state else { panic!("backlog fixture is not ok") };
    let first = ok.backlog.iter().find(|e| e.available).unwrap();
    let s = suggest(Some(&state));
    assert_eq!(s.case, Case::BacklogWithAvailable);
    assert_eq!(s.skill, "relay-session");
    assert_eq!(s.backlog_id.as_deref(), Some(first.id.as_str()));
    assert_eq!(s.title.as_deref(), Some(first.text.as_str()));
    assert_eq!(s.todo_id, None);
}

#[test]
fn backlog_without_an_available_item_sends_to_continue() {
    let state = synthetic(
        "# Backlog\n\n- [ ] B-001 - Primeiro (needs: B-002)\n- [!] B-002 - Bloqueado\n",
        "# Active task\n\nNo active task.\n",
    );
    let RelayState::Ok(_) = &state else { panic!("synthetic backlog state is inconsistent") };
    assert_eq!(shape(&suggest(Some(&state))), (Case::BacklogWithoutAvailable, "relay-continue", None, None, None));
}

#[test]
fn done_suggests_a_new_idea_with_spec() {
    let state = fixture("done");
    assert_eq!(shape(&suggest(Some(&state))), (Case::Done, "relay-spec", None, None, None));
}

#[test]
fn idle_suggests_starting_with_spec() {
    let state = fixture("idle");
    assert_eq!(shape(&suggest(Some(&state))), (Case::Idle, "relay-spec", None, None, None));
}

#[test]
fn the_same_input_gives_the_same_suggestion() {
    for status in ["idle", "backlog", "ready", "in_progress", "blocked", "done", "inconsistent"] {
        let state = fixture(status);
        assert_eq!(suggest(Some(&state)), suggest(Some(&state)), "{status}");
        assert_eq!(suggest(Some(&state)), suggest(Some(&fixture(status))), "{status} rederived");
    }
}
