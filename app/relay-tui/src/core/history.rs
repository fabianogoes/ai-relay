//! Text extraction for the Histórico view (ADR-0010): the title of each spec
//! and the full changelog records.
//!
//! Only extraction. No protocol rule lives here: `RelayState`, the integrity
//! checks and the conformance suite do not read any of this, and the TypeScript
//! `relay-core` has no counterpart (spec 20261002-002, "O parse novo fica só no
//! Rust"). Pure, like the rest of the core.

use std::sync::LazyLock;

use regex::Regex;

use super::parse::parse_backlog;
use super::types::RelayFiles;

/// One backlog entry as the Histórico levels show it: the text without its
/// `spec:`/`needs:` annotations, and the raw marker. Availability is not
/// derived here (the view takes it from the core's `OkState`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub id: String,
    pub text: String,
    pub marker: char,
    pub needs: Vec<String>,
}

impl HistoryItem {
    pub fn is_done(&self) -> bool {
        self.marker == 'x'
    }
}

/// A spec file with the backlog items that point at it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecEntry {
    /// `.specs/<file name>`, the key in `RelayFiles::specs`.
    pub path: String,
    pub id: String,
    pub title: String,
    pub items: Vec<HistoryItem>,
}

impl SpecEntry {
    pub fn done(&self) -> usize {
        self.items.iter().filter(|i| i.is_done()).count()
    }
}

/// Everything the Histórico view lists, extracted from the records alone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    /// Newest first: descending file name.
    pub specs: Vec<SpecEntry>,
    /// Items whose `spec:` is missing or is not a file of `.specs/`.
    pub no_spec: Vec<HistoryItem>,
    /// Every changelog record, in textual order.
    pub records: Vec<TaskRecord>,
}

impl History {
    /// The records of a backlog item, in the changelog's textual order.
    pub fn tasks_of(&self, backlog_id: &str) -> Vec<&TaskRecord> {
        self.records.iter().filter(|r| r.backlog.as_deref() == Some(backlog_id)).collect()
    }
}

/// Extracts the Histórico lists from the content of the records. Independent
/// of `derive_state`, so it also works while the records are `inconsistent`.
pub fn extract_history(files: &RelayFiles) -> History {
    let mut specs: Vec<SpecEntry> = files
        .specs
        .iter()
        .map(|(path, text)| SpecEntry {
            path: path.clone(),
            id: spec_id(path),
            title: spec_title(path, text),
            items: Vec::new(),
        })
        .collect();
    // `BTreeMap` iterates ascending; the view wants the most recent first.
    specs.reverse();

    let mut no_spec = Vec::new();
    for entry in parse_backlog(&files.backlog) {
        let item = HistoryItem {
            id: entry.id,
            text: entry.text,
            marker: entry.marker,
            needs: entry.needs,
        };
        match entry.spec.as_deref().and_then(|s| specs.iter_mut().find(|e| e.path == s)) {
            Some(spec) => spec.items.push(item),
            None => no_spec.push(item),
        }
    }

    History { specs, no_spec, records: extract_task_records(&files.changelog) }
}

/// One changelog record with every field the Detalhe level shows. A field the
/// record does not carry is `None`, never invented: old records have no
/// `Criteria`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub date: String,
    pub todo_id: String,
    pub title: String,
    pub backlog: Option<String>,
    pub spec: Option<String>,
    pub result: Option<String>,
    pub evidence: Option<String>,
    pub criteria: Option<String>,
    pub decisions: Option<String>,
}

static RECORD_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^##\s+(\S+)\s+-\s+(\S+)\s+-\s+([^\n\r\x{2028}\x{2029}]*)$").unwrap()
});
static FIELD_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-\s*([A-Za-z]+):\s*([^\n\r\x{2028}\x{2029}]*)$").unwrap());
static TITLE_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s+(.*\S)\s*$").unwrap());
static SPEC_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]{8}-[0-9]{3})\s+-\s+(.*)$").unwrap());

/// Which field of the record is open, so that its continuation lines join it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Open {
    None,
    Backlog,
    Spec,
    Result,
    Evidence,
    Criteria,
    Decisions,
}

fn append(slot: &mut Option<String>, text: &str) {
    let text = text.trim();
    match slot {
        Some(existing) if !text.is_empty() => {
            if !existing.is_empty() {
                existing.push(' ');
            }
            existing.push_str(text);
        }
        Some(_) => {}
        None => *slot = Some(text.to_string()),
    }
}

fn slot_of(record: &mut TaskRecord, open: Open) -> Option<&mut Option<String>> {
    match open {
        Open::None => None,
        Open::Backlog => Some(&mut record.backlog),
        Open::Spec => Some(&mut record.spec),
        Open::Result => Some(&mut record.result),
        Open::Evidence => Some(&mut record.evidence),
        Open::Criteria => Some(&mut record.criteria),
        Open::Decisions => Some(&mut record.decisions),
    }
}

/// Every record of the changelog, in textual order. A repeated `T-NNN` (an
/// append-only correction) is kept as its own record. A field's continuation
/// lines (non-blank lines that are neither a field nor a header) are joined to
/// it with a space; a blank line, an unknown field or the next header closes it.
pub fn extract_task_records(changelog: &str) -> Vec<TaskRecord> {
    let mut records = Vec::new();
    let mut current: Option<TaskRecord> = None;
    let mut open = Open::None;

    for line in changelog.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(m) = RECORD_HEADER.captures(line) {
            records.extend(current.take());
            current = Some(TaskRecord {
                date: m[1].to_string(),
                todo_id: m[2].to_string(),
                title: m[3].trim().to_string(),
                backlog: None,
                spec: None,
                result: None,
                evidence: None,
                criteria: None,
                decisions: None,
            });
            open = Open::None;
            continue;
        }
        let Some(record) = current.as_mut() else {
            continue;
        };
        if let Some(m) = FIELD_LINE.captures(line) {
            open = match m[1].to_ascii_lowercase().as_str() {
                "backlog" => Open::Backlog,
                "spec" => Open::Spec,
                "result" => Open::Result,
                "evidence" => Open::Evidence,
                "criteria" => Open::Criteria,
                "decisions" => Open::Decisions,
                _ => Open::None,
            };
            if let Some(slot) = slot_of(record, open) {
                // A repeated field line replaces the earlier one.
                *slot = Some(m[2].trim().to_string());
            }
            continue;
        }
        if line.trim().is_empty() {
            open = Open::None;
            continue;
        }
        if let Some(slot) = slot_of(record, open) {
            append(slot, line);
        }
    }
    records.extend(current);
    records
}

/// The id `AAAAMMDD-NNN` of a spec path (`.specs/<id>-<slug>.md`), or the file
/// name without `.md` when it has no such prefix.
pub fn spec_id(path: &str) -> String {
    static ID: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([0-9]{8}-[0-9]{3})(?:-|$)").unwrap());
    let name = path.rsplit('/').next().unwrap_or(path);
    let stem = name.strip_suffix(".md").unwrap_or(name);
    match ID.captures(stem) {
        Some(m) => m[1].to_string(),
        None => stem.to_string(),
    }
}

/// The title of a spec: the text after `AAAAMMDD-NNN - ` in the first `# ` line;
/// without that format, the whole `# ` text; without a `# ` line, the file name.
pub fn spec_title(path: &str, text: &str) -> String {
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(m) = TITLE_LINE.captures(line) {
            return match SPEC_TITLE.captures(&m[1]) {
                Some(t) => t[2].trim().to_string(),
                None => m[1].to_string(),
            };
        }
    }
    path.rsplit('/').next().unwrap_or(path).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_carry_every_field() {
        let records = extract_task_records(
            "# Change log\n\n## 2026-09-07 - T-001 - Titulo - com hifen\n- Backlog: B-001\n\
             - Spec: .specs/a.md\n- Result: feito\n- Evidence: a\n- Criteria: A-001\n- Decisions: none\n",
        );
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!((r.date.as_str(), r.todo_id.as_str()), ("2026-09-07", "T-001"));
        assert_eq!(r.title, "Titulo - com hifen");
        assert_eq!(r.backlog.as_deref(), Some("B-001"));
        assert_eq!(r.spec.as_deref(), Some(".specs/a.md"));
        assert_eq!(r.result.as_deref(), Some("feito"));
        assert_eq!(r.evidence.as_deref(), Some("a"));
        assert_eq!(r.criteria.as_deref(), Some("A-001"));
        assert_eq!(r.decisions.as_deref(), Some("none"));
    }

    #[test]
    fn continuation_lines_join_their_field() {
        let records = extract_task_records(
            "## 2026-09-07 - T-001 - X\n- Result: primeira\n  segunda\n    terceira\n\
             - Evidence: e\n\n  solta depois de linha em branco\n- Criteria: none\n",
        );
        let r = &records[0];
        assert_eq!(r.result.as_deref(), Some("primeira segunda terceira"));
        // A blank line closes the field: what follows is not collected.
        assert_eq!(r.evidence.as_deref(), Some("e"));
        assert_eq!(r.criteria.as_deref(), Some("none"));
    }

    #[test]
    fn a_missing_field_is_none_not_invented() {
        let records = extract_task_records("## 2026-09-07 - T-001 - Antigo\n- Backlog: B-001\n- Result: r\n");
        let r = &records[0];
        assert_eq!(r.criteria, None);
        assert_eq!(r.decisions, None);
        assert_eq!(r.spec, None);
    }

    #[test]
    fn a_repeated_task_id_is_a_record_of_its_own() {
        let records = extract_task_records(
            "## 2026-09-07 - T-002 - Original\n- Backlog: B-001\n\n\
             ## 2026-09-08 - T-002 - Correcao\n- Backlog: B-001\n",
        );
        let titles: Vec<&str> = records.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["Original", "Correcao"]);
    }

    #[test]
    fn unknown_fields_and_stray_lines_are_ignored() {
        let records = extract_task_records(
            "solta\n- Backlog: B-9\n## nao e cabecalho\n## 2026-09-07 - T-001 - X\n\
             - Notes: ignorada\n  continua da ignorada\n- Result: r\r\n  c\r\n",
        );
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].backlog, None);
        assert_eq!(records[0].result.as_deref(), Some("r c"));
    }

    #[test]
    fn spec_ids_come_from_the_file_name() {
        assert_eq!(spec_id(".specs/20261002-002-relay-tui-x.md"), "20261002-002");
        assert_eq!(spec_id(".specs/20261002-002.md"), "20261002-002");
        assert_eq!(spec_id(".specs/notas.md"), "notas");
    }

    fn files(backlog: &str, changelog: &str, specs: &[(&str, &str)]) -> RelayFiles {
        RelayFiles {
            backlog: backlog.to_string(),
            changelog: changelog.to_string(),
            specs: specs.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn backlog_groups_by_spec_newest_first_with_a_no_spec_group() {
        let h = extract_history(&files(
            "- [x] B-001 - Um (spec: `.specs/20260101-001-a.md`)\n\
             - [ ] B-002 - Dois (spec: `.specs/20260202-001-b.md`) (needs: B-001)\n\
             - [x] B-003 - Tres (spec: `.specs/20260101-001-a.md`)\n\
             - [ ] B-004 - Orfao\n\
             - [ ] B-005 - Arquivo inexistente (spec: `.specs/nao-existe.md`)\n",
            "",
            &[
                (".specs/20260101-001-a.md", "# 20260101-001 - Primeira\n"),
                (".specs/20260202-001-b.md", "# 20260202-001 - Segunda\n"),
                (".specs/20260303-001-c.md", "sem titulo\n"),
            ],
        ));
        let ids: Vec<&str> = h.specs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["20260303-001", "20260202-001", "20260101-001"]);
        // A spec with no items is `0/0`.
        assert_eq!((h.specs[0].items.len(), h.specs[0].done()), (0, 0));
        assert_eq!(h.specs[0].title, "20260303-001-c.md");
        let b = &h.specs[1];
        assert_eq!((b.title.as_str(), b.items.len(), b.done()), ("Segunda", 1, 0));
        assert_eq!(b.items[0].needs, ["B-001"]);
        assert_eq!(b.items[0].text, "Dois");
        let a = &h.specs[2];
        let a_ids: Vec<&str> = a.items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!((a_ids, a.done()), (vec!["B-001", "B-003"], 2));
        let orphans: Vec<&str> = h.no_spec.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(orphans, ["B-004", "B-005"]);
    }

    #[test]
    fn tasks_of_an_item_keep_changelog_order_and_repeats() {
        let h = extract_history(&files(
            "",
            "## 2026-01-01 - T-001 - A\n- Backlog: B-1\n\n## 2026-01-02 - T-001 - Outro item\n- Backlog: B-2\n\n\
             ## 2026-01-03 - T-001 - Correcao\n- Backlog: B-1\n",
            &[],
        ));
        let titles: Vec<&str> = h.tasks_of("B-1").iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["A", "Correcao"]);
        assert!(h.tasks_of("B-9").is_empty());
    }

    #[test]
    fn empty_records_give_an_empty_history() {
        assert_eq!(extract_history(&RelayFiles::default()), History::default());
    }

    #[test]
    fn spec_titles_follow_the_three_shapes() {
        let p = ".specs/20261002-002-x.md";
        assert_eq!(spec_title(p, "# 20261002-002 - relay-tui: navegacao\n\n## Problem\n"), "relay-tui: navegacao");
        // No `AAAAMMDD-NNN - ` prefix: the whole text.
        assert_eq!(spec_title(p, "intro\n# Um titulo livre  \n"), "Um titulo livre");
        // A `##` line is not a title; with no `# ` line, the file name.
        assert_eq!(spec_title(p, "## Problem\ntexto\n"), "20261002-002-x.md");
        assert_eq!(spec_title(".specs/vazia.md", ""), "vazia.md");
    }
}
