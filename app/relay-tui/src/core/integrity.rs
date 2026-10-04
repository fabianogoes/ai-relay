//! The 19 integrity checks of `docs/PROTOCOL.md`: in protocol order, at most
//! one violation per check, and the exact detail text, which
//! `tests/fixtures/` compares.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use super::parse::{ChangelogRecord, HandoffStatus, RawEntry, RawHandoff, parse_spec_criteria};
use super::types::Violation;

/// An entry of a `## Closed` section, with the spec of the file it is in.
pub(crate) struct ClosedEntry {
    pub spec: String,
    pub entry: RawEntry,
}

/// A `Waived` line of a closing section; `has_drop` is whether that section
/// closes at least one `[-]` entry.
pub(crate) struct Waiver {
    pub spec: String,
    pub criterion: String,
    pub has_drop: bool,
}

pub(crate) struct IntegrityInput<'a> {
    pub handoff: Option<&'a RawHandoff>,
    pub handoff_count: usize,
    pub active_backlog_id: Option<&'a str>,
    pub todo: &'a [RawEntry],
    /// The backlog without the entries already archived by a closing section.
    pub backlog: &'a [RawEntry],
    /// The whole `BACKLOG.md`, archived entries included.
    pub backlog_all: &'a [RawEntry],
    pub changelog: &'a [ChangelogRecord],
    pub closed: &'a [ClosedEntry],
    pub waivers: &'a [Waiver],
    pub specs: &'a BTreeMap<String, String>,
}

static HARNESS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9._-]*$").unwrap());
static RFC3339: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:Z|[+-]([0-9]{2}):([0-9]{2}))$",
    )
    .unwrap()
});
static SPEC_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]{8}-[0-9]{3}").unwrap());

const KNOWN_MARKERS: [char; 4] = [' ', '•', '!', 'x'];

fn violation(check: &str, params: &[(&str, &str)], records: &[&str]) -> Violation {
    Violation {
        check: check.to_string(),
        params: params
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
        records: records.iter().map(|r| r.to_string()).collect(),
    }
}

/// Whether a string that already has the RFC 3339 shape is a valid instant.
/// The day is lenient (1–31 for every month, so `02-31` passes) and `24:00:00`
/// is accepted but no other hour 24; an out-of-range month, minute, second or
/// offset is rejected.
fn rfc3339_is_parseable(value: &str) -> bool {
    let Some(c) = RFC3339.captures(value) else {
        return false;
    };
    let num = |i: usize| c.get(i).map(|g| g.as_str().parse::<u32>().unwrap());
    let (month, day) = (num(2).unwrap(), num(3).unwrap());
    let (hour, minute, second) = (num(4).unwrap(), num(5).unwrap(), num(6).unwrap());
    let hour_ok = hour <= 23 || (hour == 24 && minute == 0 && second == 0);
    (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour_ok
        && minute <= 59
        && second <= 59
        && num(7).is_none_or(|h| h <= 23)
        && num(8).is_none_or(|m| m <= 59)
}

fn check_handoff_names_no_pending_todo(i: &IntegrityInput) -> Option<Violation> {
    let handoff = i.handoff?;
    let named = i.todo.iter().find(|e| e.id == handoff.todo_id);
    match named {
        Some(entry) if entry.marker != 'x' => None,
        Some(_) => {
            let in_progress = i.todo.iter().find(|e| e.marker == '•');
            Some(violation(
                "handoff-names-no-pending-todo",
                &[
                    ("todo_id", &handoff.todo_id),
                    ("in_progress_id", in_progress.map_or("?", |e| e.id.as_str())),
                ],
                &["handoff", "todo"],
            ))
        }
        None => Some(violation(
            "handoff-names-no-pending-todo",
            &[("todo_id", &handoff.todo_id)],
            &["handoff", "todo"],
        )),
    }
}

fn check_backlog_id_mismatch(i: &IntegrityInput) -> Option<Violation> {
    if i.handoff.is_none() && i.active_backlog_id.is_none() {
        return None;
    }
    if let Some(active) = i.active_backlog_id
        && !i.backlog.iter().any(|e| e.id == active)
    {
        return Some(violation(
            "backlog-id-mismatch",
            &[("backlog_id", active)],
            &["handoff", "todo", "backlog"],
        ));
    }
    let handoff = i.handoff?;
    if Some(handoff.backlog_id.as_str()) != i.active_backlog_id {
        return Some(violation(
            "backlog-id-mismatch",
            &[
                ("handoff_backlog_id", &handoff.backlog_id),
                ("todo_backlog_id", i.active_backlog_id.unwrap_or("nenhum")),
            ],
            &["handoff", "todo", "backlog"],
        ));
    }
    None
}

fn check_spec_path_mismatch(i: &IntegrityInput) -> Option<Violation> {
    let handoff = i.handoff?;
    let Some(entry) = i
        .backlog
        .iter()
        .find(|e| Some(e.id.as_str()) == i.active_backlog_id)
    else {
        return Some(violation(
            "spec-path-mismatch",
            &[("backlog_id", i.active_backlog_id.unwrap_or("?"))],
            &["handoff", "backlog"],
        ));
    };
    if handoff.spec.is_empty() {
        return Some(violation("spec-path-mismatch", &[], &["handoff"]));
    }
    let entry_spec = entry.spec.as_deref().unwrap_or("");
    if entry_spec.is_empty() {
        return Some(violation(
            "spec-path-mismatch",
            &[("backlog_id", &entry.id)],
            &["backlog"],
        ));
    }
    if entry_spec != handoff.spec {
        return Some(violation(
            "spec-path-mismatch",
            &[
                ("handoff_spec", &handoff.spec),
                ("backlog_spec", entry_spec),
            ],
            &["handoff", "backlog"],
        ));
    }
    None
}

fn check_handoff_harness(i: &IntegrityInput) -> Option<Violation> {
    let handoff = i.handoff?;
    if handoff.harness.is_empty() || !HARNESS.is_match(&handoff.harness) {
        return Some(violation(
            "handoff-harness-invalid",
            &[("harness", &handoff.harness)],
            &["handoff"],
        ));
    }
    None
}

fn check_handoff_updated(i: &IntegrityInput) -> Option<Violation> {
    let handoff = i.handoff?;
    if handoff.updated.is_empty() || !rfc3339_is_parseable(&handoff.updated) {
        return Some(violation(
            "handoff-updated-invalid",
            &[("updated", &handoff.updated)],
            &["handoff"],
        ));
    }
    None
}

fn check_multiple_handoffs(i: &IntegrityInput) -> Option<Violation> {
    (i.handoff_count > 1).then(|| {
        violation(
            "multiple-handoffs",
            &[("count", &i.handoff_count.to_string())],
            &["handoff"],
        )
    })
}

fn check_handoff_status_invalid(i: &IntegrityInput) -> Option<Violation> {
    let HandoffStatus::Invalid(status) = &i.handoff?.status else {
        return None;
    };
    Some(violation(
        "handoff-status-invalid",
        &[("status", status)],
        &["handoff"],
    ))
}

fn check_duplicate_id(i: &IntegrityInput) -> Option<Violation> {
    // A `B-NNN` is one entry of the backlog or of one closing section; the
    // same ID in `BACKLOG.md` and in the closing of its own spec is the
    // archiving window, not a repetition.
    let mut seen: Vec<(&str, Option<&str>, bool)> = Vec::new();
    for e in i.backlog_all {
        seen.push((e.id.as_str(), e.spec.as_deref(), false));
    }
    for c in i.closed {
        seen.push((c.entry.id.as_str(), Some(c.spec.as_str()), true));
    }
    for (n, (id, spec, closed)) in seen.iter().enumerate() {
        let repeated = seen[..n]
            .iter()
            .any(|(other_id, other_spec, other_closed)| {
                other_id == id && !(closed != other_closed && other_spec == spec)
            });
        if repeated {
            return Some(violation(
                "duplicate-id",
                &[("id", id)],
                &["backlog", "changelog"],
            ));
        }
    }
    for (n, entry) in i.todo.iter().enumerate() {
        if i.todo[..n].iter().any(|other| other.id == entry.id) {
            return Some(violation("duplicate-id", &[("id", &entry.id)], &["todo"]));
        }
    }
    None
}

fn check_changelog_spec_mismatch(i: &IntegrityInput) -> Option<Violation> {
    for record in i.changelog {
        let Some(file_spec) = record.file_spec.as_deref() else {
            continue;
        };
        let entry_spec = i
            .backlog_all
            .iter()
            .chain(i.closed.iter().map(|c| &c.entry))
            .find(|e| e.id == record.backlog_id)
            .and_then(|e| e.spec.as_deref());
        if let Some(entry_spec) = entry_spec
            && spec_prefix(entry_spec) != spec_prefix(file_spec)
        {
            return Some(violation(
                "changelog-spec-mismatch",
                &[
                    ("todo_id", &record.todo_id),
                    ("changelog_spec", file_spec),
                    ("backlog_id", &record.backlog_id),
                    ("backlog_spec", entry_spec),
                ],
                &["changelog", "backlog"],
            ));
        }
    }
    None
}

fn check_todo_cleared_before_changelog(i: &IntegrityInput) -> Option<Violation> {
    for entry in i.todo.iter().filter(|e| e.marker == 'x') {
        let has = i
            .changelog
            .iter()
            .any(|r| Some(r.backlog_id.as_str()) == i.active_backlog_id && r.todo_id == entry.id);
        if !has {
            return Some(violation(
                "todo-cleared-before-changelog",
                &[("todo_id", &entry.id)],
                &["handoff", "changelog"],
            ));
        }
    }
    None
}

fn check_backlog_done_with_pending_todo(i: &IntegrityInput) -> Option<Violation> {
    let active = i.active_backlog_id?;
    let entry = i.backlog.iter().find(|e| e.id == active);
    if entry.is_some_and(|e| e.marker == 'x') && i.todo.iter().any(|e| e.marker != 'x') {
        return Some(violation(
            "backlog-done-with-pending-todo",
            &[("backlog_id", active)],
            &["backlog", "todo"],
        ));
    }
    None
}

fn check_unknown_marker(i: &IntegrityInput) -> Option<Violation> {
    // `[-]` only exists in the backlog: a subtask is never dropped.
    i.todo
        .iter()
        .filter(|e| e.marker == '-' || !KNOWN_MARKERS.contains(&e.marker))
        .chain(
            i.backlog
                .iter()
                .filter(|e| e.marker != '-' && !KNOWN_MARKERS.contains(&e.marker)),
        )
        .next()
        .map(|e| {
            violation(
                "unknown-marker",
                &[("marker", &e.marker.to_string()), ("id", &e.id)],
                &["todo", "backlog"],
            )
        })
}

fn records<'a>(i: &'a IntegrityInput) -> [(&'static str, &'a [RawEntry]); 2] {
    [("todo", i.todo), ("backlog", i.backlog)]
}

fn check_needs_unknown_id(i: &IntegrityInput) -> Option<Violation> {
    for (name, entries) in records(i) {
        let mut ids: HashSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        if name == "backlog" {
            ids.extend(i.closed.iter().map(|c| c.entry.id.as_str()));
        }
        for entry in entries {
            if let Some(need) = entry.needs.iter().find(|n| !ids.contains(n.as_str())) {
                return Some(violation(
                    "needs-unknown-id",
                    &[("id", &entry.id), ("needed_id", need), ("record", name)],
                    &[name],
                ));
            }
        }
    }
    None
}

/// `new Map(entries.map(e => [e.id, e]))`: the last entry wins the value, the
/// first appearance fixes the key order.
fn index_by_id(entries: &[RawEntry]) -> (Vec<&str>, HashMap<&str, &RawEntry>) {
    let mut order = Vec::new();
    let mut by_id = HashMap::new();
    for entry in entries {
        if by_id.insert(entry.id.as_str(), entry).is_none() {
            order.push(entry.id.as_str());
        }
    }
    (order, by_id)
}

fn visit<'a>(
    id: &'a str,
    trail: &[&'a str],
    by_id: &HashMap<&'a str, &'a RawEntry>,
    color: &mut HashMap<&'a str, u8>,
) -> Option<Vec<&'a str>> {
    color.insert(id, 1);
    if let Some(entry) = by_id.get(id) {
        for need in &entry.needs {
            let need = need.as_str();
            match color.get(need).copied().unwrap_or(0) {
                1 => {
                    let mut cycle = trail.to_vec();
                    cycle.extend([id, need]);
                    return Some(cycle);
                }
                0 => {
                    let mut next = trail.to_vec();
                    next.push(id);
                    if let Some(found) = visit(need, &next, by_id, color) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
    }
    color.insert(id, 2);
    None
}

fn check_needs_cycle(i: &IntegrityInput) -> Option<Violation> {
    for (name, entries) in records(i) {
        let (order, by_id) = index_by_id(entries);
        let mut color: HashMap<&str, u8> = HashMap::new();
        for id in order {
            if color.get(id).copied().unwrap_or(0) == 0
                && let Some(cycle) = visit(id, &[], &by_id, &mut color)
            {
                return Some(violation(
                    "needs-cycle",
                    &[("record", name), ("cycle", &cycle.join(" -> "))],
                    &[name],
                ));
            }
        }
    }
    None
}

fn check_needs_incomplete_on_done(i: &IntegrityInput) -> Option<Violation> {
    for (name, entries) in records(i) {
        let (_, by_id) = index_by_id(entries);
        for entry in entries.iter().filter(|e| e.marker == 'x') {
            for need in &entry.needs {
                let archived_done = name == "backlog"
                    && i.closed
                        .iter()
                        .any(|c| c.entry.id == *need && c.entry.marker == 'x');
                if !archived_done && by_id.get(need.as_str()).is_none_or(|dep| dep.marker != 'x') {
                    return Some(violation(
                        "needs-incomplete-on-done",
                        &[("id", &entry.id), ("needed_id", need)],
                        &[name],
                    ));
                }
            }
        }
    }
    None
}

fn check_needs_dropped_entry(i: &IntegrityInput) -> Option<Violation> {
    for entry in i
        .backlog
        .iter()
        .filter(|e| e.marker != 'x' && e.marker != '-')
    {
        for need in &entry.needs {
            let dropped = i.backlog.iter().any(|e| e.id == *need && e.marker == '-')
                || i.closed
                    .iter()
                    .any(|c| c.entry.id == *need && c.entry.marker == '-');
            if dropped {
                return Some(violation(
                    "needs-dropped-entry",
                    &[("id", &entry.id), ("needed_id", need)],
                    &["backlog"],
                ));
            }
        }
    }
    None
}

fn check_dropped_without_reason(i: &IntegrityInput) -> Option<Violation> {
    i.backlog
        .iter()
        .chain(i.closed.iter().map(|c| &c.entry))
        .find(|e| e.marker == '-' && e.dropped.as_deref().is_none_or(|r| r.trim().is_empty()))
        .map(|e| {
            violation(
                "dropped-without-reason",
                &[("id", &e.id)],
                &["backlog", "changelog"],
            )
        })
}

fn check_waived_without_drop(i: &IntegrityInput) -> Option<Violation> {
    i.waivers.iter().find(|w| !w.has_drop).map(|w| {
        violation(
            "waived-without-drop",
            &[("criterion", &w.criterion), ("spec", &w.spec)],
            &["changelog"],
        )
    })
}

fn spec_prefix(path: &str) -> &str {
    SPEC_PREFIX.find(path).map_or("", |m| m.as_str())
}

fn check_criteria_without_evidence(i: &IntegrityInput) -> Option<Violation> {
    let mut by_spec: Vec<(&str, Vec<&RawEntry>)> = Vec::new();
    for entry in i.backlog {
        let Some(spec) = entry.spec.as_deref().filter(|s| !s.is_empty()) else {
            continue;
        };
        match by_spec.iter_mut().find(|(path, _)| *path == spec) {
            Some((_, list)) => list.push(entry),
            None => by_spec.push((spec, vec![entry])),
        }
    }
    for (spec_path, entries) in by_spec {
        if !entries.iter().all(|e| e.marker == 'x' || e.marker == '-') {
            continue;
        }
        let Some(content) = i.specs.get(spec_path).filter(|c| !c.is_empty()) else {
            continue;
        };
        let criteria = parse_spec_criteria(content);
        if criteria.is_empty() {
            continue;
        }
        let prefix = spec_prefix(spec_path);
        let missing: Vec<&String> = criteria
            .iter()
            .filter(|c| {
                let waived = i
                    .waivers
                    .iter()
                    .any(|w| w.has_drop && w.spec == spec_path && w.criterion == **c);
                !waived
                    && !i.changelog.iter().any(|r| {
                        (r.spec == spec_path && r.criteria.contains(c))
                            || (!prefix.is_empty() && r.criteria.contains(&format!("{prefix}/{c}")))
                    })
            })
            .collect();
        if !missing.is_empty() {
            let list: Vec<&str> = missing.iter().map(|c| c.as_str()).collect();
            return Some(violation(
                "criteria-without-evidence",
                &[("spec", spec_path), ("criteria", &list.join(", "))],
                &["backlog", "changelog", "spec"],
            ));
        }
    }
    None
}

type Check = fn(&IntegrityInput) -> Option<Violation>;

const CHECKS: [Check; 19] = [
    check_handoff_names_no_pending_todo,
    check_backlog_id_mismatch,
    check_spec_path_mismatch,
    check_handoff_harness,
    check_handoff_updated,
    check_multiple_handoffs,
    check_handoff_status_invalid,
    check_duplicate_id,
    check_changelog_spec_mismatch,
    check_todo_cleared_before_changelog,
    check_backlog_done_with_pending_todo,
    check_unknown_marker,
    check_needs_unknown_id,
    check_needs_cycle,
    check_needs_incomplete_on_done,
    check_needs_dropped_entry,
    check_dropped_without_reason,
    check_waived_without_drop,
    check_criteria_without_evidence,
];

pub(crate) fn run_integrity_checks(input: &IntegrityInput) -> Vec<Violation> {
    CHECKS.iter().filter_map(|check| check(input)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each row says whether a string that matches the RFC 3339 shape is
    // accepted.
    #[test]
    fn timestamps_are_accepted_exactly_as_the_table_says() {
        let accepted = [
            "2026-09-07T06:49:14Z",
            "2026-09-07T06:49:14-03:00",
            "2026-02-31T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-02-30T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-12-31T24:00:00Z",
            "2026-01-01T00:00:00+23:59",
            "2026-01-01T00:00:00-23:59",
            "2026-01-01T00:00:00+05:30",
            "0000-01-01T00:00:00Z",
            "9999-12-31T23:59:59Z",
        ];
        let rejected = [
            "2026-13-01T00:00:00Z",
            "2026-00-10T00:00:00Z",
            "2026-01-00T00:00:00Z",
            "2026-01-32T00:00:00Z",
            "2026-01-01T24:00:01Z",
            "2026-01-01T24:01:00Z",
            "2026-01-01T24:00:60Z",
            "2026-01-01T23:60:00Z",
            "2026-01-01T23:59:60Z",
            "2026-01-01T00:00:00+24:00",
            "2026-01-01T00:00:00-24:00",
            "2026-01-01T00:00:00+00:60",
            "2026-01-01T00:00:00+23:60",
            "2026-01-01T00:00:00-99:99",
            "2026-01-01t00:00:00z",
            "275760-09-13T00:00:00Z",
            "ontem",
            "",
        ];
        for s in accepted {
            assert!(rfc3339_is_parseable(s), "should accept {s}");
        }
        for s in rejected {
            assert!(!rfc3339_is_parseable(s), "should reject {s}");
        }
    }

    #[test]
    fn the_spec_prefix_is_the_first_date_and_sequence_in_the_path() {
        assert_eq!(
            spec_prefix(".specs/20260907-001-contrato.md"),
            "20260907-001"
        );
        assert_eq!(spec_prefix(".specs/sem-prefixo.md"), "");
    }
}
