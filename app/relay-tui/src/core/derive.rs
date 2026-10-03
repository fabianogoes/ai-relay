//! `derive_state`: the five records in, a `RelayState` out.

use std::collections::HashSet;

use super::integrity::{IntegrityInput, run_integrity_checks};
use super::parse::{
    HandoffStatus, RawEntry, RawHandoff, parse_backlog, parse_changelog, parse_handoff, parse_todo,
};
use super::types::{
    ChecklistEntry, Handoff, OkState, RelayFiles, RelayState, WorkStatus,
};

fn with_availability(entries: &[RawEntry]) -> Vec<ChecklistEntry> {
    let done: HashSet<&str> = entries
        .iter()
        .filter(|e| e.marker == 'x')
        .map(|e| e.id.as_str())
        .collect();
    entries
        .iter()
        .map(|e| ChecklistEntry {
            id: e.id.clone(),
            text: e.text.clone(),
            marker: e.marker,
            needs: e.needs.clone(),
            available: e.marker == ' ' && e.needs.iter().all(|n| done.contains(n.as_str())),
            spec: e.spec.clone(),
        })
        .collect()
}

fn to_public_handoff(handoff: &RawHandoff) -> Handoff {
    Handoff {
        backlog_id: handoff.backlog_id.clone(),
        todo_id: handoff.todo_id.clone(),
        spec: handoff.spec.clone(),
        harness: handoff.harness.clone(),
        updated: handoff.updated.clone(),
        objective: handoff.objective.clone(),
        next_step: handoff.next_step.clone(),
        context: handoff.context.clone(),
    }
}

fn derive_status(
    handoff: Option<&RawHandoff>,
    todo: &[ChecklistEntry],
    backlog: &[ChecklistEntry],
) -> WorkStatus {
    if let Some(handoff) = handoff {
        return match handoff.status {
            HandoffStatus::Blocked => WorkStatus::Blocked,
            HandoffStatus::InProgress => WorkStatus::InProgress,
        };
    }
    if !todo.is_empty() {
        if todo.iter().all(|e| e.marker == 'x') {
            return WorkStatus::Done;
        }
        if todo.iter().any(|e| e.available) {
            return WorkStatus::Ready;
        }
        return WorkStatus::Blocked;
    }
    if !backlog.is_empty() {
        return if backlog.iter().all(|e| e.marker == 'x') {
            WorkStatus::Done
        } else {
            WorkStatus::Backlog
        };
    }
    WorkStatus::Idle
}

pub fn derive_state(files: &RelayFiles) -> RelayState {
    let todo_parsed = parse_todo(&files.todo);
    let backlog_raw = parse_backlog(&files.backlog);
    let handoff_parsed = parse_handoff(&files.handoff);
    let changelog = parse_changelog(&files.changelog);

    let todo_entries = with_availability(&todo_parsed.entries);
    let backlog_entries = with_availability(&backlog_raw);

    let violations = run_integrity_checks(&IntegrityInput {
        handoff: handoff_parsed.handoff.as_ref(),
        handoff_count: handoff_parsed.count,
        active_backlog_id: todo_parsed.active_backlog_id.as_deref(),
        todo: &todo_parsed.entries,
        backlog: &backlog_raw,
        changelog: &changelog,
        specs: &files.specs,
    });
    if !violations.is_empty() {
        return RelayState::Inconsistent { violations };
    }

    RelayState::Ok(OkState {
        status: derive_status(handoff_parsed.handoff.as_ref(), &todo_entries, &backlog_entries),
        handoff: handoff_parsed.handoff.as_ref().map(to_public_handoff),
        active_backlog_id: todo_parsed.active_backlog_id,
        completed: todo_entries.iter().filter(|e| e.marker == 'x').count(),
        total: todo_entries.len(),
        todo: todo_entries,
        backlog: backlog_entries,
    })
}
