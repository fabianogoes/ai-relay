//! `derive_state`: the five records in, a `RelayState` out.

use std::collections::HashSet;

use super::history::spec_id;
use super::integrity::{ClosedEntry, IntegrityInput, Waiver, run_integrity_checks};
use super::parse::{
    ChangelogRecord, HandoffStatus, RawEntry, RawHandoff, parse_backlog, parse_changelog,
    parse_handoff, parse_todo,
};
use super::types::{ChecklistEntry, Handoff, OkState, RelayFiles, RelayState, WorkStatus};

/// `archived_done` are the `[x]` entries of closing sections: they left the
/// backlog but still satisfy a `needs`.
fn with_availability(entries: &[RawEntry], archived_done: &HashSet<&str>) -> Vec<ChecklistEntry> {
    let done: HashSet<&str> = entries
        .iter()
        .filter(|e| e.marker == 'x')
        .map(|e| e.id.as_str())
        .chain(archived_done.iter().copied())
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
            dropped: e.dropped.clone(),
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
            HandoffStatus::InProgress | HandoffStatus::Invalid(_) => WorkStatus::InProgress,
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
        return if backlog.iter().all(|e| e.marker == 'x' || e.marker == '-') {
            WorkStatus::Done
        } else {
            WorkStatus::Backlog
        };
    }
    WorkStatus::Idle
}

/// The path of the spec a per-spec changelog (`changelog/<id>.md`) belongs to:
/// the `.specs/` file with that id, or `.specs/<id>` when there is none.
fn spec_of_changelog(key: &str, files: &RelayFiles) -> String {
    let name = key.rsplit('/').next().unwrap_or(key);
    let id = name.strip_suffix(".md").unwrap_or(name);
    files
        .specs
        .keys()
        .find(|path| spec_id(path) == id)
        .cloned()
        .unwrap_or_else(|| format!(".specs/{id}"))
}

pub fn derive_state(files: &RelayFiles) -> RelayState {
    let todo_parsed = parse_todo(&files.todo);
    let backlog_all = parse_backlog(&files.backlog);
    let handoff_parsed = parse_handoff(&files.handoff);

    let mut changelog: Vec<ChangelogRecord> = parse_changelog(&files.changelog).records;
    let mut closed: Vec<ClosedEntry> = Vec::new();
    let mut waivers: Vec<Waiver> = Vec::new();
    for (key, text) in &files.changelogs {
        let spec = spec_of_changelog(key, files);
        let parsed = parse_changelog(text);
        let has_drop = parsed.closed.iter().any(|e| e.marker == '-');
        for mut record in parsed.records {
            record.spec = spec.clone();
            record.file_spec = Some(spec.clone());
            changelog.push(record);
        }
        closed.extend(parsed.closed.into_iter().map(|entry| ClosedEntry {
            spec: spec.clone(),
            entry,
        }));
        waivers.extend(parsed.waived.into_iter().map(|criterion| Waiver {
            spec: spec.clone(),
            criterion,
            has_drop,
        }));
    }

    // A backlog entry that already appears in the closing section of its own
    // spec is archived: the closing section is the authority, and the entry
    // left in `BACKLOG.md` is cleanup.
    let backlog_raw: Vec<RawEntry> = backlog_all
        .iter()
        .filter(|e| {
            !closed
                .iter()
                .any(|c| c.entry.id == e.id && e.spec.as_deref() == Some(c.spec.as_str()))
        })
        .cloned()
        .collect();
    let archived_done: HashSet<&str> = closed
        .iter()
        .filter(|c| c.entry.marker == 'x')
        .map(|c| c.entry.id.as_str())
        .collect();

    let todo_entries = with_availability(&todo_parsed.entries, &HashSet::new());
    let backlog_entries = with_availability(&backlog_raw, &archived_done);

    let violations = run_integrity_checks(&IntegrityInput {
        handoff: handoff_parsed.handoff.as_ref(),
        handoff_count: handoff_parsed.count,
        active_backlog_id: todo_parsed.active_backlog_id.as_deref(),
        todo: &todo_parsed.entries,
        backlog: &backlog_raw,
        backlog_all: &backlog_all,
        changelog: &changelog,
        closed: &closed,
        waivers: &waivers,
        specs: &files.specs,
    });
    if !violations.is_empty() {
        return RelayState::Inconsistent { violations };
    }

    RelayState::Ok(OkState {
        status: derive_status(
            handoff_parsed.handoff.as_ref(),
            &todo_entries,
            &backlog_entries,
        ),
        handoff: handoff_parsed.handoff.as_ref().map(to_public_handoff),
        active_backlog_id: todo_parsed.active_backlog_id,
        completed: todo_entries.iter().filter(|e| e.marker == 'x').count(),
        total: todo_entries.len(),
        todo: todo_entries,
        backlog: backlog_entries,
    })
}
