//! The view never panics and never draws outside its area, at any size.

use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use relay_tui::core::{RelayState, derive_state};
use relay_tui::view::{Freshness, Screen, View};
use relay_tui::workspace::read_workspace;

fn conformance_case(name: &str) -> RelayState {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance").join(name).join("workspace");
    derive_state(&read_workspace(&dir))
}

const SIZES: [(u16, u16); 14] = [
    (0, 0),
    (1, 1),
    (3, 2),
    (5, 3),
    (20, 8),
    (30, 12),
    (39, 20),
    (40, 5),
    (40, 6),
    (40, 20),
    (58, 7),
    (58, 24),
    (58, 3),
    (120, 50),
];

#[test]
fn every_screen_renders_at_every_size_without_panicking() {
    let mut cases = vec!["idle", "backlog", "ready", "in_progress", "blocked", "done", "inconsistent"]
        .into_iter()
        .map(|c| format!("status-{c}"))
        .collect::<Vec<_>>();
    cases.push("check-needs-cycle".to_string());
    for name in cases {
        let state = conformance_case(&name);
        for screen in [Screen::State(&state), Screen::NotARelayWorkspace] {
            for freshness in [Freshness::Fresh, Freshness::Updating] {
                let view = View { screen, workspace: "~/Developer/relay", freshness, now_unix: 1_790_000_000 };
                for (w, h) in SIZES {
                    let area = Rect::new(0, 0, w, h);
                    let mut buf = Buffer::empty(area);
                    (&view).render(area, &mut buf);
                }
            }
        }
    }
}

#[test]
fn an_area_that_does_not_start_at_the_origin_is_respected() {
    let state = conformance_case("status-in_progress");
    let view = View {
        screen: Screen::State(&state),
        workspace: "~/x",
        freshness: Freshness::Fresh,
        now_unix: 1_790_000_000,
    };
    let area = Rect::new(7, 3, 58, 24);
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 40));
    (&view).render(area, &mut buf);
    // Nothing outside the area was touched.
    for y in 0..40 {
        for x in 0..80 {
            let inside = x >= 7 && x < 65 && y >= 3 && y < 27;
            if !inside {
                assert_eq!(buf[(x, y)].symbol(), " ", "({x},{y}) was drawn outside the area");
            }
        }
    }
}
