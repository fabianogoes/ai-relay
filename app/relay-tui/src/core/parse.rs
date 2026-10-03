//! Parsers for the five records.
//!
//! The regular expressions keep the behavior of the first reader, written in
//! JavaScript, quirks included, because the cases of `tests/fixtures/` were
//! recorded from it. Where the Rust engine differs from JavaScript's, the
//! pattern is adjusted rather than the behavior:
//!
//! * `\d` is `[0-9]` and `\b` is the ASCII boundary (JavaScript's are ASCII).
//! * `.` is `[^\n\r\x{2028}\x{2029}]` (JavaScript's `.` does not match `\r`, so a
//!   CRLF line never matches a pattern ending in `(.*)$`).
//! * A checklist marker is one UTF-16 code unit in JavaScript, so a character
//!   outside the BMP never forms a marker.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawEntry {
    pub id: String,
    pub text: String,
    pub marker: char,
    pub needs: Vec<String>,
    pub spec: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandoffStatus {
    InProgress,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHandoff {
    pub status: HandoffStatus,
    pub backlog_id: String,
    pub todo_id: String,
    pub spec: String,
    pub harness: String,
    pub updated: String,
    pub objective: String,
    pub next_step: String,
    pub context: String,
}

/// Only the fields the integrity checks read; the date, title and evidence of
/// a record are not needed by the derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChangelogRecord {
    pub todo_id: String,
    pub backlog_id: String,
    pub spec: String,
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedTodo {
    pub active_backlog_id: Option<String>,
    pub entries: Vec<RawEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedHandoff {
    pub count: usize,
    pub handoff: Option<RawHandoff>,
}

static CHECKLIST_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*[-*]\s*\[([^\]\x{10000}-\x{10FFFF}])\]\s+(\S+)\s*-\s*([^\n\r\x{2028}\x{2029}]*)$")
        .unwrap()
});
static SPEC_ANNOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(spec:\s*([^)]*)\)").unwrap());
static NEEDS_ANNOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(needs:\s*([^)]*)\)").unwrap());
static TODO_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^#\s*Active task(?:\s*:\s*(\S+))?\s*$").unwrap());
static KEY_VALUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-\s*([A-Za-z]+):\s*([^\n\r\x{2028}\x{2029}]*)$").unwrap());
static SECTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^##\s+([^\n\r\x{2028}\x{2029}]+)$").unwrap());
static CHANGELOG_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^##\s+(\S+)\s+-\s+(\S+)\s+-\s+([^\n\r\x{2028}\x{2029}]*)$").unwrap()
});
static CRITERIA_SECTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^##\s+Acceptance criteria\s*$").unwrap());
static ANY_SECTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^##\s+").unwrap());
static CRITERION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*-\s*(A-[0-9]+)(?-u:\b)").unwrap());

/// Takes the first `(...)` annotation out of `text`: its trimmed value and the
/// text left behind (the annotation replaced by a space, then trimmed).
fn take_annot(text: &str, re: &Regex) -> (String, String) {
    let Some(caps) = re.captures(text) else {
        return (String::new(), text.to_string());
    };
    let whole = caps.get(0).unwrap();
    let value = caps[1].trim().to_string();
    let rest = format!("{} {}", &text[..whole.start()], &text[whole.end()..]);
    (value, rest.trim().to_string())
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_checklist_lines(text: &str) -> Vec<RawEntry> {
    let mut entries = Vec::new();
    for line in text.split('\n') {
        let Some(m) = CHECKLIST_LINE.captures(line) else {
            continue;
        };
        let (spec_value, rest) = take_annot(&m[3], &SPEC_ANNOT);
        let (needs_value, rest) = take_annot(&rest, &NEEDS_ANNOT);
        entries.push(RawEntry {
            id: m[2].to_string(),
            text: rest.trim().to_string(),
            marker: m[1].chars().next().unwrap(),
            needs: if needs_value.is_empty() {
                Vec::new()
            } else {
                split_list(&needs_value)
            },
            spec: if spec_value.is_empty() {
                None
            } else {
                Some(spec_value.replace('`', ""))
            },
        });
    }
    entries
}

pub(crate) fn parse_backlog(text: &str) -> Vec<RawEntry> {
    parse_checklist_lines(text)
}

pub(crate) fn parse_todo(text: &str) -> ParsedTodo {
    let active_backlog_id = TODO_HEADER
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|g| g.as_str().to_string());
    ParsedTodo { active_backlog_id, entries: parse_checklist_lines(text) }
}

pub(crate) fn parse_handoff(text: &str) -> ParsedHandoff {
    if text.trim().is_empty() || text.contains("No active handoff") {
        return ParsedHandoff { count: 0, handoff: None };
    }

    let mut kv: HashMap<String, String> = HashMap::new();
    let mut sections: HashMap<String, String> = HashMap::new();
    let mut count = 0;
    let mut current: Option<String> = None;

    for line in text.split('\n') {
        if let Some(m) = KEY_VALUE.captures(line) {
            let key = m[1].to_ascii_lowercase();
            if key == "status" {
                count += 1;
            }
            kv.insert(key, m[2].trim().to_string());
            current = None;
            continue;
        }
        if let Some(m) = SECTION.captures(line) {
            let name = m[1].trim().to_lowercase();
            sections.insert(name.clone(), String::new());
            current = Some(name);
            continue;
        }
        if let Some(name) = &current {
            let body = sections.get_mut(name).unwrap();
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
        }
    }

    let field = |map: &HashMap<String, String>, key: &str| map.get(key).cloned().unwrap_or_default();
    let section = |key: &str| sections.get(key).map(|s| s.trim().to_string()).unwrap_or_default();

    let handoff = RawHandoff {
        status: if kv.get("status").map(String::as_str) == Some("blocked") {
            HandoffStatus::Blocked
        } else {
            HandoffStatus::InProgress
        },
        backlog_id: field(&kv, "backlog"),
        todo_id: field(&kv, "todo"),
        spec: field(&kv, "spec"),
        harness: field(&kv, "harness"),
        updated: field(&kv, "updated"),
        objective: section("objective"),
        next_step: section("next step"),
        context: section("context"),
    };
    ParsedHandoff { count, handoff: Some(handoff) }
}

pub(crate) fn parse_changelog(text: &str) -> Vec<ChangelogRecord> {
    let mut records = Vec::new();
    let mut current: Option<ChangelogRecord> = None;
    for line in text.split('\n') {
        if let Some(m) = CHANGELOG_HEADER.captures(line) {
            records.extend(current.take());
            current = Some(ChangelogRecord {
                todo_id: m[2].to_string(),
                backlog_id: String::new(),
                spec: String::new(),
                criteria: Vec::new(),
            });
            continue;
        }
        let Some(record) = current.as_mut() else {
            continue;
        };
        if let Some(m) = KEY_VALUE.captures(line) {
            let value = m[2].trim();
            match m[1].to_ascii_lowercase().as_str() {
                "backlog" => record.backlog_id = value.to_string(),
                "spec" => record.spec = value.to_string(),
                "criteria" => record.criteria = split_list(value),
                _ => {}
            }
        }
    }
    records.extend(current);
    records
}

pub(crate) fn parse_spec_criteria(text: &str) -> Vec<String> {
    let mut criteria = Vec::new();
    let mut in_section = false;
    for line in text.split('\n') {
        if CRITERIA_SECTION.is_match(line) {
            in_section = true;
            continue;
        }
        if in_section && ANY_SECTION.is_match(line) {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(m) = CRITERION.captures(line) {
            criteria.push(m[1].to_string());
        }
    }
    criteria
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|i| i.to_string()).collect()
    }

    // Expected values below are the outputs of the first (TypeScript) parser on the
    // same inputs, including the inputs it silently skips.
    #[test]
    fn checklist_lines_follow_the_reference_parser() {
        let text = "- [ ] B-001 - Um (spec: `.specs/a.md`) (needs: B-002, B-003)\n\
                    * [x] B-002 -   Dois  \n\
                    - [?] B-003 - Tres\n\
                    - [😀] B-004 - emoji\n\
                    - [ ] B-005 - crlf\r\n\
                    \x20 - [•] B-006 - indent (needs: )\n\
                    -[!] B-007-semespaco\n\
                    - [ ] B-008 - (spec: ) vazio\n\
                    - [ ] B-009 - (spec: `) crase";
        let entries = parse_backlog(text);
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        // B-004 (marker outside the BMP) and B-005 (CRLF) never match.
        assert_eq!(ids, ["B-001", "B-002", "B-003", "B-006", "B-007", "B-008", "B-009"]);
        assert_eq!(entries[0].text, "Um");
        assert_eq!(entries[0].needs, s(&["B-002", "B-003"]));
        assert_eq!(entries[0].spec.as_deref(), Some(".specs/a.md"));
        assert_eq!((entries[1].marker, entries[1].text.as_str()), ('x', "Dois"));
        assert_eq!(entries[2].marker, '?');
        assert_eq!((entries[3].marker, entries[3].needs.len()), ('•', 0));
        assert_eq!((entries[4].marker, entries[4].text.as_str()), ('!', "semespaco"));
        assert_eq!(entries[5].spec, None);
        assert_eq!(entries[6].spec.as_deref(), Some(""));
    }

    #[test]
    fn todo_header_gives_the_active_backlog_id() {
        assert_eq!(
            parse_todo("# Active task: B-004\n\n- [ ] T-001 - x").active_backlog_id.as_deref(),
            Some("B-004")
        );
        let empty = parse_todo("# Active task\n\nNo active task.\n");
        assert_eq!((empty.active_backlog_id, empty.entries.len()), (None, 0));
        assert_eq!(
            parse_todo("#Active task :   B-9   \n").active_backlog_id.as_deref(),
            Some("B-9")
        );
    }

    #[test]
    fn handoff_follows_the_reference_parser() {
        let parsed = parse_handoff(
            "# Handoff\n\n- Status: blocked\n- Backlog: B-1\n- TODO: T-2\n- Spec: .specs/x.md\n\
             - Harness: codex\n- Updated: 2026-01-01T00:00:00Z\n\n## Objective\n\nlinha1\n\nlinha2\n\
             ## Next step\nvai\n## Context\n- Status: dentro\nfim\n",
        );
        let h = parsed.handoff.unwrap();
        // The second `Status` line counts and overrides the first; it also ends
        // the Context section, so `fim` is not collected.
        assert_eq!(parsed.count, 2);
        assert_eq!(h.status, HandoffStatus::InProgress);
        assert_eq!((h.backlog_id.as_str(), h.todo_id.as_str(), h.harness.as_str()), ("B-1", "T-2", "codex"));
        assert_eq!(h.objective, "linha1\n\nlinha2");
        assert_eq!((h.next_step.as_str(), h.context.as_str()), ("vai", ""));
    }

    #[test]
    fn an_empty_or_cleared_handoff_is_no_handoff() {
        for text in ["# Handoff\n\nNo active handoff.\n", "   \n", ""] {
            let parsed = parse_handoff(text);
            assert_eq!((parsed.count, parsed.handoff), (0, None));
        }
    }

    #[test]
    fn changelog_keeps_the_fields_the_checks_read() {
        let records = parse_changelog(
            "# Change log\n\n## 2026-09-07 - T-001 - Titulo aqui\n- Backlog: B-001\n- Spec: .specs/a.md\n\
             - Evidence: a\n  continua\n- Criteria: A-001, 20260907-001/A-002,,\n- Decisions: none\n\n\
             ## 2026-09-08 - T-002 - Outro\n- Backlog: B-002\n- Criteria: none\n",
        );
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].todo_id, "T-001");
        assert_eq!(records[0].backlog_id, "B-001");
        assert_eq!(records[0].spec, ".specs/a.md");
        assert_eq!(records[0].criteria, s(&["A-001", "20260907-001/A-002"]));
        assert_eq!((records[1].spec.as_str(), records[1].criteria.clone()), ("", s(&["none"])));
        // A key line before any header, and a `##` line that is not a header,
        // never open a record.
        assert!(parse_changelog("- Backlog: B-9\n## solto\n- Backlog: B-8\n").is_empty());
    }

    #[test]
    fn spec_criteria_stop_at_the_next_section() {
        let text = "# T\n\n## Acceptance criteria\n- A-001 - um\n  - A-002 - dois\n- A-003x - nao\n\
                    - A-004\n- a-005 - minuscula\n## Backlog candidates\n- A-009 - fora\n";
        assert_eq!(parse_spec_criteria(text), s(&["A-001", "A-002", "A-004"]));
        assert_eq!(parse_spec_criteria("## ACCEPTANCE CRITERIA  \n- A-1 - x\n"), s(&["A-1"]));
    }
}
