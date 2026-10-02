//! The derived-state contract of ADR-0003, as Rust types.
//!
//! Mirrors `app/relay-core/src/types.ts`. The serialized form (camelCase keys,
//! an absent optional `spec`) is fixed by `app/conformance/`; this crate has no
//! serializer of its own because the view consumes these types directly.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
    Backlog,
    Ready,
    InProgress,
    Blocked,
    Done,
    Idle,
}

impl WorkStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            WorkStatus::Backlog => "backlog",
            WorkStatus::Ready => "ready",
            WorkStatus::InProgress => "in_progress",
            WorkStatus::Blocked => "blocked",
            WorkStatus::Done => "done",
            WorkStatus::Idle => "idle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub backlog_id: String,
    pub todo_id: String,
    pub spec: String,
    pub harness: String,
    pub updated: String,
    pub objective: String,
    pub next_step: String,
    pub context: String,
}

/// One checklist line. `marker` is one of `' '`, `'•'`, `'!'`, `'x'` in an
/// `Ok` state; any other marker is reported as an `unknown-marker` violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistEntry {
    pub id: String,
    pub text: String,
    pub marker: char,
    pub needs: Vec<String>,
    pub available: bool,
    pub spec: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkState {
    pub status: WorkStatus,
    pub handoff: Option<Handoff>,
    pub active_backlog_id: Option<String>,
    pub todo: Vec<ChecklistEntry>,
    pub backlog: Vec<ChecklistEntry>,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub check: String,
    pub detail: String,
    pub records: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayState {
    Ok(OkState),
    Inconsistent { violations: Vec<Violation> },
}

/// The content of the five records, already read. A missing record is `""`;
/// each spec is keyed `.specs/<file name>`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelayFiles {
    pub backlog: String,
    pub todo: String,
    pub handoff: String,
    pub changelog: String,
    pub specs: BTreeMap<String, String>,
}
