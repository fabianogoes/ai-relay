//! The screen: a pure function of the derived state, the freshness and the
//! clock, drawn into a ratatui buffer. No disk, no clock of its own, no
//! protocol logic: `available`, status and counts arrive from the core.
//!
//! The layout rules (cards, heights, widths) are in
//! `docs/design-system/terminal.md`, "View".

mod cards;
mod hint;
mod history;
mod specs;
pub mod text;

pub use history::{HistoryScreen, detail_extent, list_geometry, row_at};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use std::sync::LazyLock;

use crate::core::{History, RelayState};
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
    /// The extracted spec titles and files, which Agora needs to group the
    /// backlog by spec. `no_history()` when there are none.
    pub history: &'a History,
    /// The observed path, as it should be shown.
    pub workspace: &'a str,
    pub freshness: Freshness,
    /// The clock, in Unix seconds, for "há 4 min".
    pub now_unix: i64,
}

/// No spec files at all: for a screen that has nothing to group by.
pub fn no_history() -> &'static History {
    static EMPTY: LazyLock<History> = LazyLock::new(History::default);
    &EMPTY
}

/// Below this many columns only the header and the status line are drawn.
const MIN_CARD_WIDTH: u16 = 40;
/// Below this many rows only the header and the footer are drawn.
const MIN_HEIGHT: u16 = 6;
/// From this many rows on, a blank row separates the header from the cards.
const AIRY_HEIGHT: u16 = 10;

/// Where the parts of a screen go: the header row, the body, and the footer, a
/// framed card of three rows when there is room (a plain line below that). With
/// room, a blank row also sits between the header and the body and between the
/// body and the footer. Shared by Agora and Histórico, and by the click mapping.
#[derive(Debug, Clone, Copy)]
pub(super) struct Layout {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
    /// The footer is a framed card (three rows).
    pub boxed: bool,
}

pub(super) fn layout(area: Rect) -> Layout {
    let boxed = area.height >= AIRY_HEIGHT;
    let footer_height = if boxed { 3 } else { 1 };
    let footer = Rect { y: area.bottom().saturating_sub(footer_height), height: footer_height.min(area.height), ..area };
    let top = area.y + if boxed { 2 } else { 1 };
    // The blank row between the body and the footer card.
    let bottom = footer.y.saturating_sub(u16::from(boxed));
    Layout {
        header: Rect { height: 1.min(area.height), ..area },
        body: Rect { y: top, height: bottom.saturating_sub(top), ..area },
        footer,
        boxed,
    }
}

impl Widget for &View<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let places = layout(area);
        header(self.workspace, self.freshness, places.header, buf);
        if area.height == 1 {
            return;
        }
        footer(&FOOTER_NOW, places, buf);
        if area.height < MIN_HEIGHT {
            return;
        }
        let body = places.body;
        if area.width < MIN_CARD_WIDTH {
            cards::status_line(self, body, buf);
        } else {
            cards::body(self, body, buf);
        }
    }
}

fn header(workspace: &str, freshness: Freshness, area: Rect, buf: &mut Buffer) {
    let width = area.width as usize;
    let badge = " relay ";
    let (dot, word) = match freshness {
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
        left.push(Span::styled(text::truncate(workspace, room), theme::color(theme::META)));
    }
    buf.set_line(area.x, area.y, &Line::from(left), area.width);
    if width > badge.len() + right_width {
        let x = area.x + (width - right_width) as u16;
        buf.set_line(x, area.y, &Line::from(right), right_width as u16);
    }
}

/// One key of the footer: what is pressed and what it does. When the footer
/// does not fit, the hint with the lowest `rank` goes first; `q sair` never goes.
#[derive(Debug, Clone, Copy)]
pub(super) struct Hint {
    pub key: &'static str,
    pub label: &'static str,
    pub rank: u8,
}

const QUIT_RANK: u8 = u8::MAX;
const QUIT: Hint = Hint { key: "q", label: "sair", rank: QUIT_RANK };
const RELOAD: Hint = Hint { key: "r", label: "recarregar", rank: 0 };
const MOVE: Hint = Hint { key: "↑↓", label: "mover", rank: 1 };
const SCROLL: Hint = Hint { key: "↑↓", label: "rolar", rank: 1 };
const OPEN: Hint = Hint { key: "Enter", label: "abrir", rank: 2 };
const BACK: Hint = Hint { key: "Esc", label: "voltar", rank: 3 };
const TO_HISTORY: Hint = Hint { key: "Tab", label: "histórico", rank: 4 };
const TO_NOW: Hint = Hint { key: "Tab", label: "agora", rank: 4 };

/// The footers of `docs/design-system/terminal.md`, in the order they are shown.
pub(super) const FOOTER_NOW: [Hint; 3] = [TO_HISTORY, RELOAD, QUIT];
pub(super) const FOOTER_LIST: [Hint; 6] = [MOVE, OPEN, BACK, TO_NOW, RELOAD, QUIT];
pub(super) const FOOTER_DETAIL: [Hint; 5] = [SCROLL, BACK, TO_NOW, RELOAD, QUIT];

fn hints_width(hints: &[Hint]) -> usize {
    let words: usize = hints.iter().map(|h| text::width(h.key) + 1 + text::width(h.label)).sum();
    words + 3 * hints.len().saturating_sub(1)
}

/// The hints that fit in `budget` columns: whole ones only, the least important
/// dropped first.
fn fitting(hints: &[Hint], budget: usize) -> Vec<Hint> {
    let mut kept = hints.to_vec();
    while hints_width(&kept) > budget {
        let Some(lowest) = kept.iter().filter(|h| h.rank != QUIT_RANK).map(|h| h.rank).min() else {
            break;
        };
        let at = kept.iter().position(|h| h.rank == lowest).unwrap();
        kept.remove(at);
    }
    kept
}

/// The footer: the keys in a framed card, or on one plain line when the screen
/// is short. Either way they stay inside the width of the cards' content (the
/// width minus four), so a hint is never cut in half.
fn footer(hints: &[Hint], places: Layout, buf: &mut Buffer) {
    let area = places.footer;
    let kept = fitting(hints, (area.width as usize).saturating_sub(4));
    let mut spans = Vec::new();
    for (i, hint) in kept.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", theme::color(theme::META)));
        }
        spans.push(Span::styled(hint.key, theme::bold(theme::FG)));
        spans.push(Span::styled(format!(" {}", hint.label), theme::color(theme::META)));
    }
    footer_line(places, spans, buf);
}

/// Draws `spans` as the footer's one row: inside the card when it is boxed, after
/// a space on the plain line otherwise.
fn footer_line(places: Layout, mut spans: Vec<Span<'static>>, buf: &mut Buffer) {
    let area = places.footer;
    if places.boxed {
        let inner = cards::card(buf, area, &[], &[], theme::DIM);
        buf.set_line(inner.x, inner.y, &Line::from(spans), inner.width);
    } else {
        spans.insert(0, Span::raw(" "));
        buf.set_line(area.x, area.y, &Line::from(spans), area.width);
    }
}

/// The question that `Esc` in Agora asks, in place of the footer's keys: leaving
/// is confirmed by `Esc`, `Enter` or `y`, and any other key cancels. The longest
/// wording that fits the width of the cards' content (the width minus four) is
/// used; the answer keys are never cut in half.
pub fn quit_prompt(area: Rect, buf: &mut Buffer) {
    if area.width == 0 || area.height < 2 {
        return;
    }
    let places = layout(area);
    let budget = (area.width as usize).saturating_sub(4);
    let wordings = [
        "Esc, Enter ou y confirmam · outra tecla cancela",
        "Esc, Enter ou y confirmam",
        "Esc ou y confirmam",
        "y confirma",
        "y",
    ];
    let rest = wordings
        .iter()
        .find(|w| text::width("Sair? ") + text::width(w) <= budget)
        .copied()
        .unwrap_or("");
    // The frame stays; only its row is rewritten.
    let row = if places.boxed {
        Rect { x: places.footer.x + 2, y: places.footer.y + 1, width: places.footer.width.saturating_sub(4), height: 1 }
    } else {
        Rect { y: places.footer.y, ..places.footer }
    };
    let blank = " ".repeat(row.width as usize);
    buf.set_line(row.x, row.y, &Line::from(blank), row.width);
    let mut spans = vec![
        Span::styled("Sair?", theme::bold(theme::YELLOW)),
        Span::styled(format!(" {rest}"), theme::color(theme::META)),
    ];
    if !places.boxed {
        spans.insert(0, Span::raw(" "));
    }
    buf.set_line(row.x, row.y, &Line::from(spans), row.width);
}
