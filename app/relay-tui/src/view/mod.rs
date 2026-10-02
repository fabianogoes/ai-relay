//! The screen: a pure function of the derived state, the freshness and the
//! clock, drawn into a ratatui buffer. No disk, no clock of its own, no
//! protocol logic: `available`, status and counts arrive from the core.
//!
//! The layout rules (cards, heights, widths) are in
//! `docs/design-system/README.md` section 10, "View".

mod cards;
pub mod text;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::core::RelayState;
use crate::theme;

/// What the screen is showing.
#[derive(Debug, Clone, Copy)]
pub enum Screen<'a> {
    State(&'a RelayState),
    /// The observed directory has no `.orchestration/`: not a Relay workspace.
    NotARelayWorkspace,
}

/// Whether the shown snapshot is the settled one or a change is in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Updating,
}

#[derive(Debug, Clone, Copy)]
pub struct View<'a> {
    pub screen: Screen<'a>,
    /// The observed path, as it should be shown.
    pub workspace: &'a str,
    pub freshness: Freshness,
    /// The clock, in Unix seconds, for "há 4 min".
    pub now_unix: i64,
}

/// Below this many columns only the header and the status line are drawn.
const MIN_CARD_WIDTH: u16 = 40;
/// Below this many rows only the header and the footer are drawn.
const MIN_HEIGHT: u16 = 6;
/// From this many rows on, a blank row separates the header from the cards.
const AIRY_HEIGHT: u16 = 10;

impl Widget for &View<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        header(self, Rect { height: 1, ..area }, buf);
        if area.height == 1 {
            return;
        }
        let footer_y = area.bottom() - 1;
        footer(Rect { y: footer_y, height: 1, ..area }, buf);
        if area.height < MIN_HEIGHT {
            return;
        }
        let top = area.y + if area.height >= AIRY_HEIGHT { 2 } else { 1 };
        let body = Rect { y: top, height: footer_y.saturating_sub(top), ..area };
        if area.width < MIN_CARD_WIDTH {
            cards::status_line(self, body, buf);
        } else {
            cards::body(self, body, buf);
        }
    }
}

fn header(view: &View, area: Rect, buf: &mut Buffer) {
    let width = area.width as usize;
    let badge = " relay ";
    let (dot, word) = match view.freshness {
        Freshness::Fresh => (theme::GREEN, "atualizado"),
        Freshness::Updating => (theme::YELLOW, "atualizando"),
    };
    let right = vec![
        Span::styled("●", theme::color(dot)),
        Span::styled(format!(" {word}"), theme::color(theme::META)),
    ];
    let right_width = 2 + word.len();
    let mut left = vec![Span::styled(
        badge,
        Style::new().fg(theme::ON_BADGE).bg(theme::GREEN).add_modifier(Modifier::BOLD),
    )];
    let room = width.saturating_sub(badge.len() + 1 + right_width + 1);
    if room >= 4 {
        left.push(Span::raw(" "));
        left.push(Span::styled(text::truncate(view.workspace, room), theme::color(theme::META)));
    }
    buf.set_line(area.x, area.y, &Line::from(left), area.width);
    if width > badge.len() + right_width {
        let x = area.x + (width - right_width) as u16;
        buf.set_line(x, area.y, &Line::from(right), right_width as u16);
    }
}

fn footer(area: Rect, buf: &mut Buffer) {
    let line = Line::from(vec![
        Span::styled(" q", theme::bold(theme::FG)),
        Span::styled(" sair", theme::color(theme::META)),
    ]);
    buf.set_line(area.x, area.y, &line, area.width);
}
