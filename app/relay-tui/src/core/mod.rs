//! Port of `app/relay-core`: parse the five records and derive a `RelayState`.
//!
//! Pure by construction: nothing here touches the disk or the environment.

mod derive;
mod history;
mod integrity;
mod parse;
mod types;

pub use derive::derive_state;
pub use history::{
    History, HistoryItem, SpecEntry, TaskRecord, extract_history, extract_task_records, spec_id,
    spec_title,
};
pub use types::{
    ChecklistEntry, Handoff, OkState, RelayFiles, RelayState, Violation, WorkStatus,
};
