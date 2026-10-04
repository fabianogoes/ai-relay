//! Checks the live repository records independently of history snapshots.

use std::path::Path;

use relay_tui::core::{RelayState, derive_state};
use relay_tui::workspace::read_workspace;

#[test]
fn live_repository_records_are_consistent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    match derive_state(&read_workspace(&root)) {
        RelayState::Ok(_) => {}
        RelayState::Inconsistent { violations } => {
            panic!("live repository records are inconsistent: {violations:#?}");
        }
    }
}
