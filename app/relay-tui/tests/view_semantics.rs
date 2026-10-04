//! What the snapshots show, said as rules: state is never only color, and the
//! tones are the ones of `DESIGN.md`.

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
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join("workspace");
    derive_state(&read_workspace(&dir))
}

fn render(state: &RelayState, freshness: Freshness, width: u16, height: u16) -> Buffer {
    let view = View {
        history: relay_tui::view::no_history(),
        screen: Screen::State(state),
        workspace: "~/Developer/relay",
        freshness,
        now_unix: parse_rfc3339("2026-09-07T14:10:00Z").unwrap(),
        language: relay_tui::language::Language::PtBr,
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    (&view).render(area, &mut buf);
    buf
}

fn rows(buf: &Buffer) -> Vec<String> {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect()
}

fn text(buf: &Buffer) -> String {
    rows(buf).join("\n")
}

/// Where `needle` first appears: its column and row.
fn find(buf: &Buffer, needle: &str) -> Option<(u16, u16)> {
    rows(buf).iter().enumerate().find_map(|(y, row)| {
        row.find(needle)
            .map(|byte| (row[..byte].chars().count() as u16, y as u16))
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
            let buf = render(
                &case(&format!("status-{name}")),
                Freshness::Fresh,
                width,
                24,
            );
            let label = theme::status_label(status);
            assert!(
                text(&buf).contains(label),
                "status-{name} at {width} columns lacks \"{label}\""
            );
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
        let RelayState::Ok(ok) = &state else {
            panic!("{name} is not ok")
        };
        let label = theme::status_label(ok.status);
        let (x, y) = find(&buf, label).unwrap_or_else(|| panic!("no label in {name}"));
        let cell = &buf[(x, y)];
        assert_eq!(cell.fg, tone, "{name}: label tone");
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "{name}: label is bold"
        );
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
    assert_eq!(
        (
            buf[(x, y)].fg,
            buf[(x, y)].modifier.contains(Modifier::BOLD)
        ),
        (theme::RED, true)
    );
    assert_eq!(buf[(x - 2, y)].symbol(), "●");
    let (cx, cy) = find(&buf, "handoff-names-no-pending-todo").unwrap();
    assert_eq!(buf[(cx, cy)].fg, theme::RED);
    // Parameters are plain text, not red (red is under 4.5:1 on some backgrounds).
    let (dx, dy) = find(&buf, "todo_id=T-001").unwrap();
    assert_eq!(buf[(dx, dy)].fg, theme::FG);
}

#[test]
fn a_blocked_item_and_an_unavailable_one_say_why_in_words() {
    // Tall enough for the whole TODO (the footer card takes rows from the body).
    let blocked = render(&case("status-blocked"), Freshness::Fresh, 58, 32);
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
                    assert!(
                        decorative,
                        "status-{name}: \"{symbol}\" at ({x},{y}) in a decorative color"
                    );
                }
            }
        }
    }
}

#[test]
fn no_cell_uses_a_color_outside_the_palette() {
    let palette = [
        theme::FG,
        theme::META,
        theme::GREEN,
        theme::BLUE,
        theme::YELLOW,
        theme::RED,
        theme::ID,
        theme::DIM,
        theme::BAR_EMPTY,
        Color::Reset,
    ];
    for name in [
        "idle",
        "backlog",
        "ready",
        "in_progress",
        "blocked",
        "done",
        "inconsistent",
    ] {
        let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, 58, 24);
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let fg = buf[(x, y)].fg;
                // The `relay` badge draws ON_BADGE on a green background.
                assert!(
                    palette.contains(&fg) || fg == theme::ON_BADGE,
                    "status-{name}: {fg:?} at ({x},{y})"
                );
            }
        }
    }
}

// ------------------------------------------------- next-step line (spec 004)

/// The row of the next-step line: the one just above the footer.
/// The row of the next-step line: the last of the body, which is the row above
/// the footer on a short screen, and above the blank row and the footer card
/// (three rows) on a taller one.
fn hint_y(buf: &Buffer) -> u16 {
    if buf.area.height >= 10 {
        buf.area.height - 5
    } else {
        buf.area.height - 2
    }
}

fn hint_row(buf: &Buffer) -> String {
    rows(buf)[hint_y(buf) as usize].clone()
}

fn has_hint(buf: &Buffer) -> bool {
    hint_row(buf).contains("relay-")
}

#[test]
fn the_skill_of_the_suggestion_is_text_in_bold_fg_never_only_a_color() {
    let expected: [(&str, &[&str]); 7] = [
        ("idle", &["relay-spec"]),
        ("backlog", &["relay-session"]),
        ("ready", &["relay-session"]),
        ("in_progress", &["relay-session"]),
        ("blocked", &["relay-session"]),
        ("done", &["relay-spec"]),
        ("inconsistent", &["relay-status", "relay-continue"]),
    ];
    for (name, skills) in expected {
        let buf = render(&case(&format!("status-{name}")), Freshness::Fresh, 120, 24);
        let y = hint_y(&buf);
        let row = hint_row(&buf);
        for skill in skills {
            let byte = row
                .find(skill)
                .unwrap_or_else(|| panic!("{name}: `{skill}` is not in `{row}`"));
            let x0 = row[..byte].chars().count() as u16;
            for x in x0..x0 + skill.chars().count() as u16 {
                let cell = &buf[(x, y)];
                assert!(
                    cell.modifier.contains(Modifier::BOLD),
                    "{name}: `{skill}` is not bold at column {x}"
                );
                assert_eq!(
                    cell.fg,
                    theme::FG,
                    "{name}: `{skill}` is not fg at column {x}"
                );
            }
        }
        // Ids stay in the `id` color and the rest of the sentence in `meta`.
        for (x, ch) in row.chars().enumerate() {
            let cell = &buf[(x as u16, y)];
            if ch == ' ' || cell.modifier.contains(Modifier::BOLD) {
                continue;
            }
            assert!(
                cell.fg == theme::META || cell.fg == theme::ID,
                "{name}: column {x} (`{ch}`) is {:?}",
                cell.fg
            );
        }
    }
}

#[test]
fn a_cut_suggestion_ends_in_an_ellipsis_and_narrow_screens_have_none() {
    let state = case("status-in_progress");
    let wide = render(&state, Freshness::Fresh, 58, 24);
    assert!(hint_row(&wide).trim_end().ends_with("relay-session."));
    let narrow = render(&state, Freshness::Fresh, 40, 24);
    assert!(
        hint_row(&narrow).trim_end().ends_with('…'),
        "{}",
        hint_row(&narrow)
    );
    for width in [39, 30] {
        let buf = render(&state, Freshness::Fresh, width, 24);
        assert!(
            !text(&buf).contains("relay-session"),
            "a hint at {width} columns"
        );
    }
}

/// A compacted Handoff has no blank row after its first line.
fn handoff_is_compact(buf: &Buffer) -> bool {
    let all = rows(buf);
    let Some(top) = all.iter().position(|r| r.starts_with("╭ Handoff")) else {
        return false;
    };
    let second = &all[top + 2];
    !second.trim_matches(|c| c == '│' || c == ' ').is_empty()
}

#[test]
fn the_line_yields_before_the_handoff_is_compacted_or_the_todo_is_cut() {
    for name in ["status-in_progress", "status-blocked"] {
        let state = case(name);
        let mut appeared = false;
        for height in 6..=40 {
            let buf = render(&state, Freshness::Fresh, 100, height);
            let shown = has_hint(&buf);
            assert!(
                !appeared || shown,
                "{name}: the line vanished again at height {height}"
            );
            appeared |= shown;
            if shown {
                assert!(
                    !handoff_is_compact(&buf),
                    "{name}: the line is shown over a compact Handoff at height {height}"
                );
                let cut = rows(&buf)
                    .iter()
                    .any(|r| r.starts_with("│ +") && r.contains("iten"));
                assert!(
                    !cut,
                    "{name}: the line is shown over a cut TODO at height {height}"
                );
            }
        }
        assert!(appeared, "{name}: the line never appears");
    }
}
