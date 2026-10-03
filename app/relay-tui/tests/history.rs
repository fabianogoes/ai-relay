//! `extract_history` against the real records of this repository (spec
//! 20261002-002, A-002): its specs, backlog entries and dozens of changelog
//! records, with repeated `T-NNN`.

use std::collections::HashSet;
use std::path::Path;

use relay_tui::core::{History, extract_history};
use relay_tui::workspace::read_workspace;

fn real_history() -> History {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    extract_history(&read_workspace(&root))
}

#[test]
fn every_spec_file_is_listed_newest_first_with_a_title() {
    let h = real_history();
    assert!(h.specs.len() >= 6, "expected the repository's specs, got {}", h.specs.len());
    let paths: Vec<&str> = h.specs.iter().map(|s| s.path.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(paths, sorted, "specs must be in descending file-name order");
    for spec in &h.specs {
        assert!(!spec.title.is_empty() && !spec.id.is_empty(), "{}", spec.path);
        assert!(!spec.title.starts_with('#'), "{}", spec.path);
    }
    let first = h.specs.iter().find(|s| s.id == "20260907-002").expect("spec 20260907-002");
    assert!(!first.title.contains("20260907-002 -"), "the id is split from the title");
}

#[test]
fn backlog_items_are_grouped_under_the_spec_they_point_at() {
    let h = real_history();
    let navigation = h
        .specs
        .iter()
        .find(|s| s.id == "20261002-002")
        .expect("spec 20261002-002 of this very change");
    let ids: Vec<&str> = navigation.items.iter().map(|i| i.id.as_str()).collect();
    assert!(ids.contains(&"B-040") && ids.contains(&"B-041"), "{ids:?}");
    let b040 = navigation.items.iter().find(|i| i.id == "B-040").unwrap();
    assert!(b040.text.contains("ADR-0010"), "annotations are stripped: {}", b040.text);
    assert!(!b040.text.contains("spec:") && !b040.text.contains("needs:"));
    // The oldest spec's items are all done and counted, never a percentage.
    let old = h.specs.iter().find(|s| s.id == "20260907-002").unwrap();
    assert!(!old.items.is_empty());
    assert_eq!(old.done(), old.items.len());

    // Every item sits in exactly one group.
    let mut seen = HashSet::new();
    for item in h.specs.iter().flat_map(|s| &s.items).chain(&h.no_spec) {
        assert!(seen.insert(item.id.clone()), "{} listed twice", item.id);
    }
    assert!(seen.contains("B-005"));
}

#[test]
fn tasks_of_a_finished_item_carry_every_field() {
    let h = real_history();
    let tasks = h.tasks_of("B-040");
    let ids: Vec<&str> = tasks.iter().map(|t| t.todo_id.as_str()).collect();
    assert_eq!(ids, ["T-001", "T-002", "T-003"]);
    let last = tasks[2];
    assert!(last.title.contains("Histórico"), "{}", last.title);
    assert_eq!(last.date, "2026-10-03");
    assert_eq!(last.spec.as_deref(), Some(".specs/20261002-002-relay-tui-navegacao-pelo-historico.md"));
    assert!(last.result.as_deref().is_some_and(|r| r.contains("AGENTS.md")));
    assert!(last.evidence.is_some() && last.decisions.is_some());
    assert_eq!(last.criteria.as_deref(), Some("A-001"));
}

#[test]
fn repeated_ids_are_kept() {
    let h = real_history();
    assert!(h.records.len() >= 70, "got {}", h.records.len());
    // A record without `Criteria` is covered by the unit tests: the real
    // changelog has none.
    // `T-NNN` restarts at every backlog item, so the same id recurs across
    // items and each occurrence is a record of its own. (A repeat inside one
    // item, an append-only correction, is covered by the unit tests: the real
    // changelog has none yet.)
    let t001 = h.records.iter().filter(|r| r.todo_id == "T-001").count();
    assert!(t001 >= 20, "got {t001}");
    for r in &h.records {
        assert!(!r.date.is_empty() && !r.title.is_empty(), "{r:?}");
    }
}
