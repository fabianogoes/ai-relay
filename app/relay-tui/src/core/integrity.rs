//! The 13 integrity checks of `docs/PROTOCOL.md`: in protocol order, at most
//! one violation per check, and the exact detail text, which
//! `tests/fixtures/` compares.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use super::parse::{ChangelogRecord, RawEntry, RawHandoff, parse_spec_criteria};
use super::types::Violation;

pub(crate) struct IntegrityInput<'a> {
    pub handoff: Option<&'a RawHandoff>,
    pub handoff_count: usize,
    pub active_backlog_id: Option<&'a str>,
    pub todo: &'a [RawEntry],
    pub backlog: &'a [RawEntry],
    pub changelog: &'a [ChangelogRecord],
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

fn violation(check: &str, detail: String, records: &[&str]) -> Violation {
    Violation {
        check: check.to_string(),
        detail,
        records: records.iter().map(|r| r.to_string()).collect(),
    }
}

/// Whether the first (TypeScript) reader's `Date.parse` accepted a string that already
/// has the RFC 3339 shape. V8 is lenient about the day (1–31 for every month,
/// so `02-31` parses) and accepts `24:00:00` but no other hour 24; it rejects
/// an out-of-range month, minute, second or offset.
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
                format!(
                    "O handoff aponta {}, que está [x]; {} está [•].",
                    handoff.todo_id,
                    in_progress.map_or("?", |e| e.id.as_str())
                ),
                &["handoff", "todo"],
            ))
        }
        None => Some(violation(
            "handoff-names-no-pending-todo",
            format!("O handoff aponta {}, que não existe no TODO.", handoff.todo_id),
            &["handoff", "todo"],
        )),
    }
}

fn check_backlog_id_mismatch(i: &IntegrityInput) -> Option<Violation> {
    if i.handoff.is_none() && i.active_backlog_id.is_none() {
        return None;
    }
    if let Some(active) = i.active_backlog_id {
        if !i.backlog.iter().any(|e| e.id == active) {
            return Some(violation(
                "backlog-id-mismatch",
                format!("O backlog ativo {active} não existe no backlog."),
                &["handoff", "todo", "backlog"],
            ));
        }
    }
    let handoff = i.handoff?;
    if Some(handoff.backlog_id.as_str()) != i.active_backlog_id {
        return Some(violation(
            "backlog-id-mismatch",
            format!(
                "O handoff diz {}, o TODO diz {}.",
                handoff.backlog_id,
                i.active_backlog_id.unwrap_or("nenhum")
            ),
            &["handoff", "todo", "backlog"],
        ));
    }
    None
}

fn check_spec_path_mismatch(i: &IntegrityInput) -> Option<Violation> {
    let handoff = i.handoff?;
    let Some(entry) = i.backlog.iter().find(|e| Some(e.id.as_str()) == i.active_backlog_id) else {
        return Some(violation(
            "spec-path-mismatch",
            format!(
                "A tarefa ativa {} não tem entrada de backlog confrontável.",
                i.active_backlog_id.unwrap_or("?")
            ),
            &["handoff", "backlog"],
        ));
    };
    if handoff.spec.is_empty() {
        return Some(violation("spec-path-mismatch", "O handoff não tem Spec.".into(), &["handoff"]));
    }
    let entry_spec = entry.spec.as_deref().unwrap_or("");
    if entry_spec.is_empty() {
        return Some(violation(
            "spec-path-mismatch",
            format!("A entrada {} não tem spec.", entry.id),
            &["backlog"],
        ));
    }
    if entry_spec != handoff.spec {
        return Some(violation(
            "spec-path-mismatch",
            format!("O handoff diz {}, a tarefa diz {}.", handoff.spec, entry_spec),
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
            format!("Harness inválido: \"{}\".", handoff.harness),
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
            format!("Updated inválido: \"{}\".", handoff.updated),
            &["handoff"],
        ));
    }
    None
}

fn check_multiple_handoffs(i: &IntegrityInput) -> Option<Violation> {
    (i.handoff_count > 1).then(|| {
        violation(
            "multiple-handoffs",
            format!("Existem {} registros de handoff.", i.handoff_count),
            &["handoff"],
        )
    })
}

fn check_todo_cleared_before_changelog(i: &IntegrityInput) -> Option<Violation> {
    for entry in i.todo.iter().filter(|e| e.marker == 'x') {
        let has = i.changelog.iter().any(|r| {
            Some(r.backlog_id.as_str()) == i.active_backlog_id && r.todo_id == entry.id
        });
        if !has {
            return Some(violation(
                "todo-cleared-before-changelog",
                format!("A subtarefa {} está [x] sem registro no changelog.", entry.id),
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
            format!("Backlog {active} está done com TODO pendente."),
            &["backlog", "todo"],
        ));
    }
    None
}

fn check_unknown_marker(i: &IntegrityInput) -> Option<Violation> {
    i.todo
        .iter()
        .chain(i.backlog)
        .find(|e| !KNOWN_MARKERS.contains(&e.marker))
        .map(|e| {
            violation(
                "unknown-marker",
                format!("Marcador desconhecido \"[{}]\" em {}.", e.marker, e.id),
                &["todo", "backlog"],
            )
        })
}

fn records<'a>(i: &'a IntegrityInput) -> [(&'static str, &'a [RawEntry]); 2] {
    [("todo", i.todo), ("backlog", i.backlog)]
}

fn check_needs_unknown_id(i: &IntegrityInput) -> Option<Violation> {
    for (name, entries) in records(i) {
        let ids: HashSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        for entry in entries {
            if let Some(need) = entry.needs.iter().find(|n| !ids.contains(n.as_str())) {
                return Some(violation(
                    "needs-unknown-id",
                    format!("{} referencia {}, ausente em {}.", entry.id, need, name),
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
            if color.get(id).copied().unwrap_or(0) == 0 {
                if let Some(cycle) = visit(id, &[], &by_id, &mut color) {
                    return Some(violation(
                        "needs-cycle",
                        format!("Ciclo em {}: {}.", name, cycle.join(" -> ")),
                        &[name],
                    ));
                }
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
                if by_id.get(need.as_str()).is_none_or(|dep| dep.marker != 'x') {
                    return Some(violation(
                        "needs-incomplete-on-done",
                        format!("{} está [x] mas precisa de {}, não concluído.", entry.id, need),
                        &[name],
                    ));
                }
            }
        }
    }
    None
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
        if !entries.iter().all(|e| e.marker == 'x') {
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
                !i.changelog.iter().any(|r| {
                    (r.spec == spec_path && r.criteria.contains(c))
                        || (!prefix.is_empty() && r.criteria.contains(&format!("{prefix}/{c}")))
                })
            })
            .collect();
        if !missing.is_empty() {
            let list: Vec<&str> = missing.iter().map(|c| c.as_str()).collect();
            return Some(violation(
                "criteria-without-evidence",
                format!("Spec {}: critérios sem evidência: {}.", spec_path, list.join(", ")),
                &["backlog", "changelog", "spec"],
            ));
        }
    }
    None
}

type Check = fn(&IntegrityInput) -> Option<Violation>;

const CHECKS: [Check; 13] = [
    check_handoff_names_no_pending_todo,
    check_backlog_id_mismatch,
    check_spec_path_mismatch,
    check_handoff_harness,
    check_handoff_updated,
    check_multiple_handoffs,
    check_todo_cleared_before_changelog,
    check_backlog_done_with_pending_todo,
    check_unknown_marker,
    check_needs_unknown_id,
    check_needs_cycle,
    check_needs_incomplete_on_done,
    check_criteria_without_evidence,
];

pub(crate) fn run_integrity_checks(input: &IntegrityInput) -> Vec<Violation> {
    CHECKS.iter().filter_map(|check| check(input)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each row is the answer of the first (TypeScript) reader's `Date.parse` (NaN or not)
    // for a string that matches the RFC 3339 shape.
    #[test]
    fn timestamps_are_accepted_exactly_as_the_reference_accepts_them() {
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
        assert_eq!(spec_prefix(".specs/20260907-001-ui.md"), "20260907-001");
        assert_eq!(spec_prefix(".specs/sem-prefixo.md"), "");
    }
}
