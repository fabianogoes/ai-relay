//! What the snapshots show, said as rules: state is never only color, and the
//! tones are the ones of the design system.

use std::path::{Path, PathBuf};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::widgets::Widget;

use relay_tui::core::{RelayState, WorkStatus, derive_state};
use relay_tui::theme;
use relay_tui::view::text::parse_rfc3339;
use relay_tui::view::{Freshness, Screen, View};
use relay_tui::workspace::read_workspace;

fn case(name: &str) -> RelayState {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance").join(name).join("workspace");
    derive_state(&read_workspace(&dir))
}

fn render(state: &RelayState, freshness: Freshness, width: u16, height: u16) -> Buffer {
    let view = View {
        screen: Screen::State(state),
        workspace: "~/Developer/relay",
        freshness,
        now_unix: parse_rfc3339("2026-09-07T14:10:00Z").unwrap(),
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    (&view).render(area, &mut buf);
    buf
}

fn rows(buf: &Buffer) -> Vec<String> {
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect::<String>())
        .collect()
}

fn text(buf: &Buffer) -> String {
    rows(buf).join("\n")
}

/// Where `needle` first appears: its column and row.
fn find(buf: &Buffer, needle: &str) -> Option<(u16, u16)> {
    rows(buf).iter().enumerate().find_map(|(y, row)| {
        row.find(needle).map(|byte| (row[..byte].chars().count() as u16, y as u16))
    })
}

#[test]
fn every_status_names_itself_in_words() {
    let expected = [
        ("idle", WorkStatus::Idle),
        ("backlog", WorkStatus::Backlog),
        ("ready", WorkStatus::Ready),
        ("in_progress", WorkStatus::InProgress),
        ("blocked", WorkStatus::Blocked),
        ("done", WorkStatus::Done),
    ];
    for (name, status) in expected {
        for width in [58, 40] {
            let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, width, 24);
            let label = theme::status_label(status);
            assert!(text(&buf).contains(label), "status-{name} at {width} columns lacks \"{label}\"");
        }
    }
    let buf = render(&case("status-inconsistent"), Freshness::Fresh, 58, 24);
    assert!(text(&buf).contains(theme::INCONSISTENT_LABEL));
}

#[test]
fn the_label_and_the_marker_carry_the_tone_of_the_status() {
    let tones = [
        ("in_progress", theme::GREEN),
        ("blocked", theme::YELLOW),
        ("ready", theme::BLUE),
        ("backlog", theme::BLUE),
        ("done", theme::GREEN),
    ];
    for (name, tone) in tones {
        let state = case(&format!("status-{name}"));
        let buf = render(&state, Freshness::Fresh, 58, 24);
        let RelayState::Ok(ok) = &state else { panic!("{name} is not ok") };
        let label = theme::status_label(ok.status);
        let (x, y) = find(&buf, label).unwrap_or_else(|| panic!("no label in {name}"));
        let cell = &buf[(x, y)];
        assert_eq!(cell.fg, tone, "{name}: label tone");
        assert!(cell.modifier.contains(Modifier::BOLD), "{name}: label is bold");
        // The dot right before the label has the same tone.
        assert_eq!(buf[(x - 2, y)].symbol(), "●", "{name}: marker");
        assert_eq!(buf[(x - 2, y)].fg, tone, "{name}: marker tone");
    }
}

#[test]
fn the_handoff_border_takes_the_tone_of_the_status() {
    for (name, tone) in [("in_progress", theme::GREEN), ("blocked", theme::YELLOW)] {
        let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, 58, 24);
        let (x, y) = find(&buf, "╭ Handoff").expect("handoff card");
        assert_eq!(buf[(x, y)].fg, tone, "{name}: top-left corner");
        assert_eq!(buf[(x + 57, y)].fg, tone, "{name}: top-right corner");
    }
}

#[test]
fn inconsistent_is_red_but_its_body_is_not() {
    let buf = render(&case("status-inconsistent"), Freshness::Fresh, 58, 24);
    let (x, y) = find(&buf, "Inconsistente").unwrap();
    assert_eq!((buf[(x, y)].fg, buf[(x, y)].modifier.contains(Modifier::BOLD)), (theme::RED, true));
    assert_eq!(buf[(x - 2, y)].symbol(), "●");
    let (cx, cy) = find(&buf, "handoff-names-no-pending-todo").unwrap();
    assert_eq!(buf[(cx, cy)].fg, theme::RED);
    // The detail is plain text, not red (red is under 4.5:1 on some backgrounds).
    let (dx, dy) = find(&buf, "O handoff aponta").unwrap();
    assert_eq!(buf[(dx, dy)].fg, theme::FG);
}

#[test]
fn a_blocked_item_and_an_unavailable_one_say_why_in_words() {
    let blocked = render(&case("status-blocked"), Freshness::Fresh, 58, 24);
    let (x, y) = find(&blocked, "bloqueado").expect("the blocked item says it is blocked");
    assert_eq!(blocked[(x, y)].fg, theme::YELLOW);
}

#[test]
fn freshness_is_a_word_and_never_only_a_dot() {
    let state = case("status-in_progress");
    let fresh = render(&state, Freshness::Fresh, 58, 24);
    let updating = render(&state, Freshness::Updating, 58, 24);
    assert!(text(&fresh).contains("atualizado") && !text(&fresh).contains("atualizando"));
    assert!(text(&updating).contains("atualizando"));
    let (x, y) = find(&fresh, "atualizado").unwrap();
    assert_eq!(fresh[(x - 2, y)].fg, theme::GREEN);
    let (x, y) = find(&updating, "atualizando").unwrap();
    assert_eq!(updating[(x - 2, y)].fg, theme::YELLOW);
}

#[test]
fn borders_and_empty_bar_segments_never_carry_text() {
    // `dim` and `bar_empty` fail AA: the only glyphs allowed in them are the
    // frame and the bar.
    for name in ["in_progress", "blocked", "ready", "backlog", "inconsistent"] {
        let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, 58, 24);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                let symbol = cell.symbol();
                let decorative = symbol.chars().all(|c| "╭╮╰╯─│█ ".contains(c));
                if cell.fg == theme::DIM || cell.fg == theme::BAR_EMPTY {
                    assert!(decorative, "status-{name}: \"{symbol}\" at ({x},{y}) in a decorative color");
                }
            }
        }
    }
}

#[test]
fn no_cell_uses_a_color_outside_the_palette() {
    let palette = [
        theme::FG, theme::META, theme::GREEN, theme::BLUE, theme::YELLOW, theme::RED, theme::ID, theme::DIM,
        theme::BAR_EMPTY, Color::Reset,
    ];
    for name in ["idle", "backlog", "ready", "in_progress", "blocked", "done", "inconsistent"] {
        let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, 58, 24);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let fg = buf[(x, y)].fg;
                // The `relay` badge draws ON_BADGE on a green background.
                assert!(palette.contains(&fg) || fg == theme::ON_BADGE, "status-{name}: {fg:?} at ({x},{y})");
            }
        }
    }
}
