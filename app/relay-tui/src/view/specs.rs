//! The grouping of the backlog by spec that Agora shows (spec 20261002-002,
//! "Visão Agora agrupada por spec"): the current spec with its open items, and
//! the other specs that still have work. Pure data; the cards are drawn in
//! `cards.rs`.
//!
//! Nothing here decides what is available or done: that comes from the core's
//! `OkState` (`available` and the markers). This only groups, in the order of
//! the backlog.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::{ChecklistEntry, History, OkState, spec_id};
use crate::language::Language;
use crate::theme;

use super::cards::{Seg, card, plain_title, plural, put, seg, seg_width};
use super::text::{self, truncate};

/// Whether the current spec is the one being worked on or the one that comes
/// next. Without an active item (state `backlog`) the choice is the protocol's
/// default recommendation, not a priority, so the card must not say "em curso".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Tag {
    InProgress,
    Next,
}

impl Tag {
    pub fn word(self, language: Language) -> &'static str {
        match (self, language) {
            (Tag::InProgress, Language::PtBr) => "em curso",
            (Tag::Next, Language::PtBr) => "a seguir",
            (Tag::InProgress, Language::En) => "in progress",
            (Tag::Next, Language::En) => "up next",
        }
    }
}

/// One spec, or the group of items with no valid spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Group<'a> {
    /// `AAAAMMDD-NNN`; `None` for the items with no spec (the Sem spec line).
    pub id: Option<String>,
    pub title: String,
    pub done: usize,
    pub total: usize,
    /// The items not done, in the order of the backlog.
    pub open: Vec<&'a ChecklistEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct Groups<'a> {
    /// The current spec and why it is the current one; absent when there is no
    /// item to anchor it (state `backlog` with nothing available, or an active
    /// item with no valid spec).
    pub current: Option<(Group<'a>, Tag)>,
    /// The other specs with at least one item not done, in the order they first
    /// appear in `BACKLOG.md`, with the Sem spec group last.
    pub others: Vec<Group<'a>>,
}

/// The path of the spec an entry points at, if it is a file of `.specs/`.
fn valid_spec<'a>(entry: &'a ChecklistEntry, history: &History) -> Option<&'a str> {
    let path = entry.spec.as_deref()?;
    history.specs.iter().any(|s| s.path == path).then_some(path)
}

pub(super) fn groups<'a>(ok: &'a OkState, history: &History) -> Groups<'a> {
    // The groups in the order they first appear, keyed by path (`None` = no spec).
    let mut keys: Vec<Option<&str>> = Vec::new();
    // A dropped entry is not work to do: Agora never shows or counts it.
    for entry in ok.backlog.iter().filter(|e| e.marker != '-') {
        let key = valid_spec(entry, history);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    // The items with no spec come last, as the Sem spec level does in Histórico.
    keys.sort_by_key(|k| k.is_none());
    let build = |key: Option<&str>| {
        let members: Vec<&ChecklistEntry> = ok
            .backlog
            .iter()
            .filter(|e| e.marker != '-' && valid_spec(e, history) == key)
            .collect();
        let (id, title) = match key {
            Some(path) => {
                let title = history
                    .specs
                    .iter()
                    .find(|s| s.path == path)
                    .map(|s| s.title.clone());
                (Some(spec_id(path)), title.unwrap_or_default())
            }
            None => (None, "Sem spec".to_string()),
        };
        Group {
            id,
            title,
            done: members.iter().filter(|e| e.marker == 'x').count(),
            total: members.len(),
            open: members
                .iter()
                .filter(|e| e.marker != 'x')
                .copied()
                .collect(),
        }
    };

    // The anchor: the active item, or else the first available one (the
    // protocol's default, which already comes from the core).
    let (anchor, tag) = match ok.active_backlog_id.as_deref() {
        Some(id) => (ok.backlog.iter().find(|e| e.id == id), Tag::InProgress),
        None => (ok.backlog.iter().find(|e| e.available), Tag::Next),
    };
    let current_key = anchor.and_then(|e| valid_spec(e, history));
    let current = current_key.map(|key| (build(Some(key)), tag));

    let others = keys
        .into_iter()
        .filter(|key| *key != current_key || current_key.is_none())
        .map(build)
        .filter(|g| !g.open.is_empty())
        .collect();
    Groups { current, others }
}

// ---------------------------------------------------------------- sizes

/// How the rows given to the backlog area are shared. Heights, not rows of
/// content: `pending` is 0 (none), 1 (the line `N specs pendentes`) or a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Shape {
    pub current: u16,
    pub pending: u16,
    /// Both cards give way to the one-line count of the whole backlog.
    pub count_line: bool,
}

impl Shape {
    pub fn height(self) -> u16 {
        if self.count_line {
            1
        } else {
            self.current + self.pending
        }
    }
}

fn current_full(g: &Groups) -> u16 {
    g.current
        .as_ref()
        .map_or(0, |(c, _)| 2 + c.open.len() as u16)
}

fn pending_full(g: &Groups) -> u16 {
    if g.others.is_empty() {
        0
    } else {
        2 + g.others.len() as u16
    }
}

/// Rows for everything whole: both cards.
pub(super) fn full_height(g: &Groups) -> u16 {
    current_full(g) + pending_full(g)
}

/// The shape for `avail` rows, in the order `DESIGN.md` gives: first Specs
/// pendentes becomes a line, then Spec atual is cut (`+N itens`), and last both
/// become the count line of the backlog. `avail` is at least 1.
pub(super) fn shape(g: &Groups, avail: u16) -> Shape {
    let (cur_full, pend_full) = (current_full(g), pending_full(g));
    let pend_line = u16::from(!g.others.is_empty());
    if avail >= cur_full + pend_full {
        return Shape {
            current: cur_full,
            pending: pend_full,
            count_line: false,
        };
    }
    if avail >= cur_full + pend_line {
        return Shape {
            current: cur_full,
            pending: pend_line,
            count_line: false,
        };
    }
    if cur_full == 0 {
        // Only the pending specs: their line is the least that says something.
        return Shape {
            current: 0,
            pending: pend_line.max(1).min(avail),
            count_line: false,
        };
    }
    // Frame, one item and the `+N itens` row make the smallest cut card: one
    // that shows no item at all would only say how many there are.
    if avail >= 4 + pend_line {
        return Shape {
            current: avail - pend_line,
            pending: pend_line,
            count_line: false,
        };
    }
    Shape {
        current: 0,
        pending: 0,
        count_line: true,
    }
}

// ---------------------------------------------------------------- drawing

fn item_glyph(e: &ChecklistEntry) -> (&'static str, Color) {
    match e.marker {
        '•' => ("●", theme::GREEN),
        '!' => ("!", theme::YELLOW),
        _ if e.available => ("○", theme::BLUE),
        _ => ("◌", theme::META),
    }
}

/// One line: `lead`, the text cut to what is left and, flush right, `right`
/// (given long and short; the short one is used when the long one would leave
/// the text almost nothing).
fn row(
    lead: Vec<Seg>,
    text: &str,
    text_style: Style,
    right: [Vec<Seg>; 2],
    width: usize,
) -> Vec<Seg> {
    let lead_w = seg_width(&lead);
    let room_for = |r: &[Seg]| width.saturating_sub(lead_w + seg_width(r) + 1);
    let right = if room_for(&right[0]) >= 12 || right[1].is_empty() {
        &right[0]
    } else {
        &right[1]
    };
    let right_w = seg_width(right);
    let (right, right_w) = if room_for(right) >= 4 {
        (right.clone(), right_w)
    } else {
        (Vec::new(), 0)
    };
    let room = width.saturating_sub(lead_w + if right_w > 0 { right_w + 1 } else { 0 });
    let text = truncate(text, room);
    let pad = width.saturating_sub(lead_w + text::width(&text) + right_w);
    let mut out = lead;
    out.push(seg(text, text_style));
    if right_w > 0 {
        out.push(seg(" ".repeat(pad), Style::new()));
        out.extend(right);
    }
    out
}

fn item_row(
    e: &ChecklistEntry,
    all: &[ChecklistEntry],
    width: usize,
    language: Language,
) -> Vec<Seg> {
    let en = language == Language::En;
    let (glyph, tone) = item_glyph(e);
    let lead = vec![
        seg(format!("{glyph} "), theme::bold(tone)),
        seg(format!("{} ", e.id), theme::color(theme::ID)),
    ];
    let (text_style, right) = match e.marker {
        '•' => (
            theme::bold(theme::FG),
            [
                vec![seg(
                    if en { "in progress" } else { "em curso" },
                    theme::fg(),
                )],
                vec![],
            ],
        ),
        '!' => (
            theme::fg(),
            [
                vec![seg(
                    if en { "blocked" } else { "bloqueado" },
                    theme::color(theme::YELLOW),
                )],
                vec![],
            ],
        ),
        _ if e.available => (
            theme::fg(),
            [
                vec![seg(
                    if en { "available" } else { "disponível" },
                    theme::fg(),
                )],
                vec![],
            ],
        ),
        _ => {
            // Which needs are still open is a lookup in the same list; whether
            // the item is available at all was decided by the core.
            let open: Vec<&str> = e
                .needs
                .iter()
                .filter(|n| all.iter().any(|o| &o.id == *n && o.marker != 'x'))
                .map(String::as_str)
                .collect();
            let short = vec![seg(
                if en { "waiting" } else { "aguardando" },
                theme::color(theme::META),
            )];
            let mut long = short.clone();
            if !open.is_empty() {
                long.push(seg(
                    format!(
                        " · {} {}",
                        if en { "after" } else { "após" },
                        open.join(", ")
                    ),
                    theme::color(theme::YELLOW),
                ));
            }
            (theme::color(theme::META), [long, short])
        }
    };
    row(lead, &e.text, text_style, right, width)
}

fn count(g: &Group) -> Seg {
    seg(format!("{}/{}", g.done, g.total), theme::color(theme::META))
}

fn current_card(
    c: &Group,
    tag: Tag,
    all: &[ChecklistEntry],
    area: Rect,
    buf: &mut Buffer,
    language: Language,
) {
    let tone = match tag {
        Tag::InProgress => theme::GREEN,
        Tag::Next => theme::BLUE,
    };
    let right = vec![
        seg(format!(" {}", tag.word(language)), theme::bold(tone)),
        seg(
            format!(" · {}/{} ", c.done, c.total),
            theme::color(theme::META),
        ),
    ];
    // The id is always whole; the title gives way to what the right side needs.
    let id = c.id.clone().unwrap_or_default();
    let room =
        (area.width as usize).saturating_sub(2 + seg_width(&right) + 1 + 3 + text::width(&id));
    let title = vec![
        seg(" ", Style::new()),
        seg(id, theme::bold(theme::ID)),
        seg(
            format!(" {} ", truncate(&c.title, room)),
            theme::bold(theme::FG),
        ),
    ];
    let inner = card(buf, area, &title, &right, theme::DIM);
    let rows = inner.height as usize;
    // The last row says what is hidden, unless everything fits.
    let shown = if c.open.len() <= rows {
        c.open.len()
    } else {
        rows.saturating_sub(1)
    };
    for (i, e) in c.open.iter().take(shown).enumerate() {
        put(
            buf,
            inner,
            i,
            &item_row(e, all, inner.width as usize, language),
        );
    }
    if shown < c.open.len() && rows > 0 {
        put(
            buf,
            inner,
            shown,
            &[seg(
                format!("+{}", plural(c.open.len() - shown, "item", "itens")),
                theme::color(theme::META),
            )],
        );
    }
}

fn pending_row(g: &Group, width: usize, language: Language) -> Vec<Seg> {
    let lead = match &g.id {
        Some(id) => vec![seg(format!("{id} "), theme::color(theme::ID))],
        None => vec![],
    };
    let title = if g.id.is_none() && language == Language::En {
        "No spec"
    } else {
        &g.title
    };
    row(lead, title, theme::fg(), [vec![count(g)], vec![]], width)
}

fn pending_card(others: &[Group], area: Rect, buf: &mut Buffer, language: Language) {
    let right = vec![seg(
        format!(" {} ", others.len()),
        theme::color(theme::META),
    )];
    let inner = card(
        buf,
        area,
        &plain_title(if language == Language::En {
            "Pending specs"
        } else {
            "Specs pendentes"
        }),
        &right,
        theme::DIM,
    );
    let rows = inner.height as usize;
    let shown = if others.len() <= rows {
        others.len()
    } else {
        rows.saturating_sub(1)
    };
    for (i, g) in others.iter().take(shown).enumerate() {
        put(
            buf,
            inner,
            i,
            &pending_row(g, inner.width as usize, language),
        );
    }
    if shown < others.len() && rows > 0 {
        put(
            buf,
            inner,
            shown,
            &[seg(
                format!("+{}", plural(others.len() - shown, "spec", "specs")),
                theme::color(theme::META),
            )],
        );
    }
}

/// Draws the backlog area in the `shape` computed for its height: Spec atual,
/// then Specs pendentes (a card or a line), or the one-line count of the whole
/// backlog.
pub(super) fn draw(
    g: &Groups,
    ok: &OkState,
    shape: Shape,
    area: Rect,
    buf: &mut Buffer,
    count_line: impl Fn(Rect, &mut Buffer),
    language: Language,
) {
    if shape.count_line {
        count_line(Rect { height: 1, ..area }, buf);
        return;
    }
    let mut y = area.y;
    if let (Some((current, tag)), true) = (&g.current, shape.current > 0) {
        current_card(
            current,
            *tag,
            &ok.backlog,
            Rect {
                y,
                height: shape.current,
                ..area
            },
            buf,
            language,
        );
        y += shape.current;
    }
    match shape.pending {
        0 => {}
        1 => {
            let line = Rect {
                y,
                height: 1,
                ..area
            };
            let n = g.others.len();
            put(
                buf,
                line,
                0,
                &[seg(
                    format!(
                        " {}",
                        if language == Language::En {
                            plural(n, "pending spec", "pending specs")
                        } else {
                            plural(n, "spec pendente", "specs pendentes")
                        }
                    ),
                    theme::color(theme::META),
                )],
            );
        }
        h => pending_card(
            &g.others,
            Rect {
                y,
                height: h,
                ..area
            },
            buf,
            language,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{RelayFiles, RelayState, derive_state, extract_history};

    const A: &str = ".specs/20260101-001-a.md";
    const B: &str = ".specs/20260202-001-b.md";
    const C: &str = ".specs/20260303-001-c.md";

    fn world(backlog: &str, todo: &str) -> (OkState, History) {
        let files = RelayFiles {
            changelogs: Default::default(),
            backlog: backlog.to_string(),
            todo: todo.to_string(),
            specs: [
                (A.to_string(), "# 20260101-001 - Primeira\n".to_string()),
                (B.to_string(), "# 20260202-001 - Segunda\n".to_string()),
                (C.to_string(), "# 20260303-001 - Terceira\n".to_string()),
            ]
            .into(),
            ..Default::default()
        };
        match derive_state(&files) {
            RelayState::Ok(ok) => (ok, extract_history(&files)),
            other => panic!("not ok: {other:?}"),
        }
    }

    fn ids(items: &[&ChecklistEntry]) -> Vec<String> {
        items.iter().map(|e| e.id.clone()).collect()
    }

    #[test]
    fn the_current_spec_is_the_one_of_the_active_item() {
        let (ok, history) = world(
            &format!(
                "- [x] B-001 - a (spec: `{A}`)\n- [•] B-002 - b (spec: `{B}`)\n- [ ] B-003 - c (spec: `{B}`)\n\
                 - [x] B-004 - d (spec: `{B}`)\n- [ ] B-005 - e (spec: `{C}`)\n"
            ),
            "# Active task: B-002\n\n- [•] T-001 - x\n",
        );
        let g = groups(&ok, &history);
        let (current, tag) = g.current.expect("a current spec");
        assert_eq!(tag, Tag::InProgress);
        assert_eq!(
            (current.id.as_deref(), current.title.as_str()),
            (Some("20260202-001"), "Segunda")
        );
        // The done item counts but is not listed; the others follow the backlog.
        assert_eq!((current.done, current.total), (1, 3));
        assert_eq!(ids(&current.open), ["B-002", "B-003"]);
        // Spec A is all done, so only C is left, and not the current one.
        assert_eq!(g.others.len(), 1);
        assert_eq!(
            (
                g.others[0].id.as_deref(),
                g.others[0].done,
                g.others[0].total
            ),
            (Some("20260303-001"), 0, 1)
        );
    }

    #[test]
    fn without_an_active_item_it_is_the_spec_of_the_first_available_one() {
        // B-001 waits for B-002 (spec A); B-002 is the first available, in spec B.
        let (ok, history) = world(
            &format!(
                "- [ ] B-001 - a (spec: `{A}`) (needs: B-002)\n- [ ] B-002 - b (spec: `{B}`)\n- [ ] B-003 - c (spec: `{C}`)\n"
            ),
            "",
        );
        let g = groups(&ok, &history);
        let (current, tag) = g.current.expect("a current spec");
        assert_eq!(tag, Tag::Next);
        assert_eq!(current.id.as_deref(), Some("20260202-001"));
        // The others keep the order of the backlog: A first, then C.
        let others: Vec<_> = g.others.iter().map(|o| o.id.clone().unwrap()).collect();
        assert_eq!(others, ["20260101-001", "20260303-001"]);
    }

    #[test]
    fn no_available_entry_means_no_current_spec_and_every_pending_spec_is_listed() {
        let (ok, history) = world(
            &format!(
                "- [!] B-001 - a (spec: `{A}`)\n- [ ] B-002 - b (spec: `{B}`) (needs: B-001)\n"
            ),
            "",
        );
        let g = groups(&ok, &history);
        assert!(g.current.is_none());
        assert_eq!(g.others.len(), 2);
    }

    #[test]
    fn items_with_no_valid_spec_form_the_sem_spec_line() {
        let (ok, history) = world(
            "- [ ] B-001 - sem anotacao\n- [ ] B-002 - arquivo que nao existe (spec: `.specs/nao-existe.md`)\n\
             - [x] B-003 - feito, sem spec\n",
            "",
        );
        let g = groups(&ok, &history);
        // The first available entry has no valid spec, so there is no current card.
        assert!(g.current.is_none());
        assert_eq!(g.others.len(), 1);
        let sem = &g.others[0];
        assert_eq!((sem.id.clone(), sem.title.as_str()), (None, "Sem spec"));
        assert_eq!((sem.done, sem.total), (1, 3));
        assert_eq!(ids(&sem.open), ["B-001", "B-002"]);
    }

    #[test]
    fn an_active_item_with_no_valid_spec_has_no_current_card() {
        let (ok, history) = world(
            "- [•] B-001 - a\n- [ ] B-002 - b (spec: `.specs/20260101-001-a.md`)\n",
            "# Active task: B-001\n\n- [•] T-001 - x\n",
        );
        let g = groups(&ok, &history);
        assert!(g.current.is_none());
        assert_eq!(g.others.len(), 2);
    }

    #[test]
    fn a_spec_with_everything_done_is_not_pending() {
        let (ok, history) = world(
            &format!("- [x] B-001 - a (spec: `{A}`)\n- [ ] B-002 - b (spec: `{B}`)\n"),
            "",
        );
        let g = groups(&ok, &history);
        assert_eq!(g.current.unwrap().0.id.as_deref(), Some("20260202-001"));
        assert!(g.others.is_empty());
    }
}
