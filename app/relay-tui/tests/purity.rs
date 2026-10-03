//! The core reads no disk and no environment, so the same input always derives the same state.

use std::fs;
use std::path::{Path, PathBuf};

// Anything that reaches outside the process: files, environment, processes,
// the network, the clock, threads.
const FORBIDDEN: [&str; 10] = [
    "std::fs",
    "std::env",
    "std::process",
    "std::net",
    "std::io",
    "std::time",
    "std::thread",
    "std::os",
    "tokio",
    "notify",
];

fn core_sources() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core");
    fs::read_dir(dir)
        .expect("src/core must exist")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect()
}

#[test]
fn the_core_touches_neither_disk_nor_environment() {
    let sources = core_sources();
    assert!(sources.len() >= 4, "src/core is missing sources");
    for path in sources {
        let text = fs::read_to_string(&path).unwrap();
        // Doc and line comments may mention these names; code may not.
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for needle in FORBIDDEN {
            assert!(
                !code.contains(needle),
                "{} uses {needle}; the core must stay pure",
                path.display()
            );
        }
    }
}
