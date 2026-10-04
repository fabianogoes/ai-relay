//! Derives every case of `tests/fixtures/` and compares it with its
//! `expected.json`.
//!
//! Every case must derive exactly its `expected.json`; the cases, not this
//! crate, are the reference.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use relay_tui::core::{ChecklistEntry, Handoff, RelayFiles, RelayState, Violation, derive_state};
use serde_json::{Value, json};

const STATUS_CASES: [&str; 7] = [
    "idle",
    "backlog",
    "ready",
    "in_progress",
    "blocked",
    "done",
    "inconsistent",
];

// Cases of the changelog structure that are not a status or a check: the
// per-spec changelog, the legacy one, closing sections, drops, waivers, CRLF
// and a strict handoff.
const STRUCTURE_CASES: [&str; 10] = [
    "all-done-with-drop",
    "archived-entry-cleanup",
    "closed-spec",
    "closed-with-waiver",
    "crlf",
    "done-active-task",
    "dropped-entry",
    "legacy-and-per-spec",
    "per-spec",
    "status-in-context",
];

// The 19 integrity checks of docs/PROTOCOL.md.
const CHECK_IDS: [&str; 19] = [
    "handoff-names-no-pending-todo",
    "backlog-id-mismatch",
    "spec-path-mismatch",
    "handoff-harness-invalid",
    "handoff-updated-invalid",
    "multiple-handoffs",
    "todo-cleared-before-changelog",
    "backlog-done-with-pending-todo",
    "unknown-marker",
    "needs-unknown-id",
    "needs-cycle",
    "needs-incomplete-on-done",
    "criteria-without-evidence",
    "handoff-status-invalid",
    "duplicate-id",
    "changelog-spec-mismatch",
    "needs-dropped-entry",
    "dropped-without-reason",
    "waived-without-drop",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn read_markdown_dir(dir: &Path, prefix: &str) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    if let Ok(dir) = fs::read_dir(dir) {
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".md") {
                files.insert(format!("{prefix}/{name}"), read_or_empty(&entry.path()));
            }
        }
    }
    files
}

// Same mapping as `workspace::read_workspace`: a missing record reads
// as empty text, each spec is keyed `.specs/<name>` and each per-spec
// changelog `changelog/<name>`.
fn load_workspace(workspace: &Path) -> RelayFiles {
    let orchestration = workspace.join(".orchestration");
    let specs = read_markdown_dir(&workspace.join(".specs"), ".specs");
    let changelogs = read_markdown_dir(&orchestration.join("changelog"), "changelog");
    RelayFiles {
        changelogs,
        backlog: read_or_empty(&orchestration.join("BACKLOG.md")),
        todo: read_or_empty(&orchestration.join("TODO.md")),
        handoff: read_or_empty(&orchestration.join("HANDOFF.md")),
        changelog: read_or_empty(&orchestration.join("CHANGELOG.md")),
        specs,
    }
}

fn handoff_json(h: &Handoff) -> Value {
    json!({
        "backlogId": h.backlog_id,
        "todoId": h.todo_id,
        "spec": h.spec,
        "harness": h.harness,
        "updated": h.updated,
        "objective": h.objective,
        "nextStep": h.next_step,
        "context": h.context,
    })
}

fn entry_json(e: &ChecklistEntry) -> Value {
    let mut value = json!({
        "id": e.id,
        "text": e.text,
        "marker": e.marker.to_string(),
        "needs": e.needs,
        "available": e.available,
    });
    // An absent optional key is absent in the JSON, never `null`.
    if let Some(spec) = &e.spec {
        value["spec"] = json!(spec);
    }
    if let Some(reason) = &e.dropped {
        value["dropped"] = json!(reason);
    }
    value
}

fn violation_json(v: &Violation) -> Value {
    json!({ "check": v.check, "params": v.params, "records": v.records })
}

// Written by hand on purpose: a second reading of the `RelayState` shape, with no
// serializer in the production crate.
fn state_json(state: &RelayState) -> Value {
    match state {
        RelayState::Ok(ok) => json!({
            "kind": "ok",
            "status": ok.status.as_str(),
            "handoff": ok.handoff.as_ref().map(handoff_json),
            "activeBacklogId": ok.active_backlog_id,
            "todo": ok.todo.iter().map(entry_json).collect::<Vec<_>>(),
            "backlog": ok.backlog.iter().map(entry_json).collect::<Vec<_>>(),
            "completed": ok.completed,
            "total": ok.total,
        }),
        RelayState::Inconsistent { violations } => json!({
            "kind": "inconsistent",
            "violations": violations.iter().map(violation_json).collect::<Vec<_>>(),
        }),
    }
}

fn case_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(fixtures_dir())
        .expect("tests/fixtures/ must exist")
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn the_suite_has_every_status_case_and_one_case_per_integrity_check() {
    let mut expected: Vec<String> = STATUS_CASES
        .iter()
        .map(|s| format!("status-{s}"))
        .chain(CHECK_IDS.iter().map(|c| format!("check-{c}")))
        .chain(STRUCTURE_CASES.iter().map(|c| format!("structure-{c}")))
        .collect();
    expected.sort();
    assert_eq!(case_names(), expected);
}

#[test]
fn every_case_derives_the_expected_state() {
    let mut failures = Vec::new();
    for name in case_names() {
        let dir = fixtures_dir().join(&name);
        let expected: Value = serde_json::from_str(&read_or_empty(&dir.join("expected.json")))
            .unwrap_or_else(|e| panic!("{name}: expected.json is not valid JSON: {e}"));
        let actual = state_json(&derive_state(&load_workspace(&dir.join("workspace"))));
        if actual != expected {
            failures.push(format!(
                "{name}\n  expected: {expected}\n  actual:   {actual}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) diverge:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
