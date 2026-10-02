//! The palette of `docs/design-system/README.md` section 10 (the terminal
//! medium): One Dark with the Charm pink for identifiers. The background is
//! never painted; it comes from the terminal.

use ratatui::style::{Color, Modifier, Style};

use crate::core::WorkStatus;

pub const FG: Color = Color::Rgb(0xab, 0xb2, 0xbf);
pub const META: Color = Color::Rgb(0x9b, 0xa6, 0xb4);
pub const GREEN: Color = Color::Rgb(0x98, 0xc3, 0x79);
pub const BLUE: Color = Color::Rgb(0x61, 0xaf, 0xef);
pub const YELLOW: Color = Color::Rgb(0xe5, 0xc0, 0x7b);
pub const RED: Color = Color::Rgb(0xe0, 0x6c, 0x75);
/// Identifiers: `B-NNN`, `T-NNN`.
pub const ID: Color = Color::Rgb(0xff, 0x75, 0xbf);
/// Borders and decorative strokes only; never text (it fails AA).
pub const DIM: Color = Color::Rgb(0x5c, 0x63, 0x70);
/// The empty segment of the TODO bar only; never text.
pub const BAR_EMPTY: Color = Color::Rgb(0x3e, 0x44, 0x51);
/// Text on a solid `GREEN` badge: the One Dark background.
pub const ON_BADGE: Color = Color::Rgb(0x28, 0x2c, 0x34);

pub fn fg() -> Style {
    Style::new().fg(FG)
}

pub fn bold(color: Color) -> Style {
    Style::new().fg(color).add_modifier(Modifier::BOLD)
}

pub fn color(color: Color) -> Style {
    Style::new().fg(color)
}

/// The tone of a derived status (design system section 10, "Status").
pub fn status_tone(status: WorkStatus) -> Color {
    match status {
        WorkStatus::InProgress | WorkStatus::Done => GREEN,
        WorkStatus::Blocked => YELLOW,
        WorkStatus::Ready | WorkStatus::Backlog => BLUE,
        WorkStatus::Idle => META,
    }
}

/// The label of a derived status, the same words as the web interface.
pub fn status_label(status: WorkStatus) -> &'static str {
    match status {
        WorkStatus::Backlog => "A escolher",
        WorkStatus::Ready => "Pronto",
        WorkStatus::InProgress => "Em andamento",
        WorkStatus::Blocked => "Bloqueado",
        WorkStatus::Done => "Concluído",
        WorkStatus::Idle => "Sem trabalho",
    }
}

pub const INCONSISTENT_LABEL: &str = "Inconsistente";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_is_the_one_in_the_design_system() {
        let hex = |c: Color| match c {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            other => panic!("not an rgb color: {other:?}"),
        };
        let table = [
            (FG, "#abb2bf"),
            (META, "#9ba6b4"),
            (GREEN, "#98c379"),
            (BLUE, "#61afef"),
            (YELLOW, "#e5c07b"),
            (RED, "#e06c75"),
            (ID, "#ff75bf"),
            (DIM, "#5c6370"),
            (BAR_EMPTY, "#3e4451"),
            (ON_BADGE, "#282c34"),
        ];
        for (color, expected) in table {
            assert_eq!(hex(color), expected);
        }
    }

    #[test]
    fn statuses_map_to_the_tones_of_the_design_system() {
        assert_eq!(status_tone(WorkStatus::InProgress), GREEN);
        assert_eq!(status_tone(WorkStatus::Done), GREEN);
        assert_eq!(status_tone(WorkStatus::Blocked), YELLOW);
        assert_eq!(status_tone(WorkStatus::Ready), BLUE);
        assert_eq!(status_tone(WorkStatus::Backlog), BLUE);
        assert_eq!(status_tone(WorkStatus::Idle), META);
    }

    #[test]
    fn every_status_has_its_word() {
        let labels: Vec<&str> = [
            WorkStatus::InProgress,
            WorkStatus::Blocked,
            WorkStatus::Ready,
            WorkStatus::Backlog,
            WorkStatus::Done,
            WorkStatus::Idle,
        ]
        .into_iter()
        .map(status_label)
        .collect();
        assert_eq!(
            labels,
            ["Em andamento", "Bloqueado", "Pronto", "A escolher", "Concluído", "Sem trabalho"]
        );
    }
}
