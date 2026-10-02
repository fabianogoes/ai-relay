//! Port of `app/relay-core`: parse the five records and derive a `RelayState`.
//!
//! Pure by construction: nothing here touches the disk or the environment.

mod derive;
mod integrity;
mod parse;
mod types;

pub use derive::derive_state;
pub use types::{
    ChecklistEntry, Handoff, OkState, RelayFiles, RelayState, Violation, WorkStatus,
};
