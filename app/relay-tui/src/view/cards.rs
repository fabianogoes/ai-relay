//! The cards of the screen. Each builds its lines for a given width and the
//! layout decides how many fit; nothing here scrolls.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Widget};

use crate::core::{ChecklistEntry, Handoff, OkState, RelayState, Violation, WorkStatus};
use crate::suggest::suggest;
use crate::theme;

use super::hint;
use super::specs;
use super::text::{relative_time_in, truncate, width, wrap_capped};
use super::{Screen, View};

/// The least a TODO card can be: frame, bar and one line.
const TODO_MIN: u16 = 4;

/// A run of text and its style.
pub(super) type Seg = (String, Style);

pub(super) fn seg(text: impl Into<String>, style: Style) -> Seg {
    (text.into(), style)
}

pub(super) fn seg_width(segs: &[Seg]) -> usize {
    segs.iter().map(|(t, _)| width(t)).sum()
}

pub(super) fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// `segs` as a line of at most `max` columns; the segment that overflows ends
/// in `…` and the rest is dropped.
fn fit(segs: &[Seg], max: usize) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used = 0;
    for (text, style) in segs {
        let w = width(text);
        if used + w <= max {
            spans.push(Span::styled(text.clone(), *style));
            used += w;
            continue;
        }
        let rest = max - used;
        if rest > 0 {
            spans.push(Span::styled(truncate(text, rest), *style));
        } else if let Some(last) = spans.pop() {
            // The cut falls between two runs: the previous one takes the `…`,
            // so a cut line never ends as if it were whole.
            let kept = last.content.chars().count().saturating_sub(1);
            let cut: String = last.content.chars().take(kept).collect();
            spans.push(Span::styled(format!("{cut}…"), last.style));
        }
        break;
    }
    Line::from(spans)
}

pub(super) fn put(buf: &mut Buffer, area: Rect, row: usize, segs: &[Seg]) {
    if row < area.height as usize {
        buf.set_line(
            area.x,
            area.y + row as u16,
            &fit(segs, area.width as usize),
            area.width,
        );
    }
}

/// Draws the frame of a card and returns the area for its content: inside the
/// border, with one column of padding on each side.
pub(super) fn card(
    buf: &mut Buffer,
    area: Rect,
    title: &[Seg],
    right: &[Seg],
    border: Color,
) -> Rect {
    let to_line = |segs: &[Seg]| {
        Line::from(
            segs.iter()
                .map(|(t, s)| Span::styled(t.clone(), *s))
                .collect::<Vec<_>>(),
        )
    };
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::color(border))
        .title_top(to_line(title));
    if !right.is_empty() {
        block = block.title_top(to_line(right).right_aligned());
    }
    let inner = block.inner(area);
    block.render(area, buf);
    Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(2),
        ..inner
    }
}

pub(super) fn plain_title(text: &str) -> Vec<Seg> {
    vec![seg(format!(" {text} "), theme::bold(theme::FG))]
}

fn count_title(done: usize, total: usize) -> Vec<Seg> {
    vec![seg(format!(" {done}/{total} "), theme::color(theme::META))]
}

fn status_title(label: &str, tone: Color) -> Vec<Seg> {
    vec![
        seg(" ● ", theme::bold(tone)),
        seg(format!("{label} "), theme::bold(tone)),
    ]
}

// ---------------------------------------------------------------- narrow

/// Below the card width: the status in one line, with the active task if any.
pub(super) fn status_line(view: &View, body: Rect, buf: &mut Buffer) {
    if body.height == 0 {
        return;
    }
    let segs = match view.screen {
        Screen::NotARelayWorkspace => {
            vec![seg(
                if view.language == crate::language::Language::En {
                    "Not a Relay workspace"
                } else {
                    "Não é um workspace Relay"
                },
                theme::color(theme::META),
            )]
        }
        Screen::State(RelayState::Inconsistent { violations }) => vec![
            seg("● ", theme::bold(theme::RED)),
            seg(theme::INCONSISTENT_LABEL, theme::bold(theme::RED)),
            seg(
                format!("  {}", plural(violations.len(), "violação", "violações")),
                theme::color(theme::META),
            ),
        ],
        Screen::State(RelayState::Ok(ok)) => {
            let tone = theme::status_tone(ok.status);
            let mut segs = vec![
                seg("● ", theme::bold(tone)),
                seg(
                    theme::status_label_in(ok.status, view.language),
                    theme::bold(tone),
                ),
            ];
            if let Some(id) = &ok.active_backlog_id {
                segs.push(seg(format!("  {id}"), theme::color(theme::ID)));
            }
            segs
        }
    };
    put(
        buf,
        Rect {
            x: body.x + 1,
            width: body.width.saturating_sub(1),
            ..body
        },
        0,
        &segs,
    );
}

// ----------------------------------------------------------------- cards

pub(super) fn body(view: &View, area: Rect, buf: &mut Buffer) {
    if area.height < 3 {
        return;
    }
    // The next-step line sits on the last row of the area, just above the
    // footer; the cards above it get one row less.
    let hinted = hint_fits(view, area);
    let cards = if hinted {
        Rect {
            height: area.height - 1,
            ..area
        }
    } else {
        area
    };
    match view.screen {
        Screen::NotARelayWorkspace => not_relay_card(
            view,
            Rect {
                height: cards.height.min(6),
                ..cards
            },
            buf,
        ),
        Screen::State(RelayState::Inconsistent { violations }) => {
            let rows = violation_rows(violations, cards.width.saturating_sub(4) as usize);
            violations_card(
                violations,
                view.language,
                Rect {
                    height: cards.height.min(rows as u16 + 2),
                    ..cards
                },
                buf,
            );
        }
        Screen::State(RelayState::Ok(ok)) => ok_body(view, ok, cards, buf),
    }
    if hinted {
        // Aligned with the content of the cards (inside their border and
        // padding), and inside the same width.
        let line = Rect {
            x: area.x + 2,
            y: area.bottom() - 1,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        let segs = hint::segments(
            &suggest(match view.screen {
                Screen::NotARelayWorkspace => None,
                Screen::State(state) => Some(state),
            }),
            line.width as usize,
            view.language,
        );
        put(buf, line, 0, &segs);
    }
}

/// Whether the next-step line is drawn. As the height shrinks the Backlog
/// gives up its frame first, then this line, then the Handoff is compacted,
/// and last the TODO is cut (`DESIGN.md`, "Altura").
fn hint_fits(view: &View, area: Rect) -> bool {
    let Some(spare) = area.height.checked_sub(1).filter(|h| *h >= 3) else {
        return false;
    };
    match view.screen {
        Screen::NotARelayWorkspace => spare >= 6,
        Screen::State(RelayState::Inconsistent { violations }) => {
            let rows = violation_rows(violations, area.width.saturating_sub(4) as usize);
            spare >= rows as u16 + 2
        }
        Screen::State(RelayState::Ok(ok)) => match ok.status {
            WorkStatus::Idle | WorkStatus::Done => spare >= 3,
            WorkStatus::Backlog => spare > specs::full_height(&specs::groups(ok, view.history)),
            WorkStatus::Ready | WorkStatus::InProgress | WorkStatus::Blocked => {
                // The line is the first to give way: it stays only while, without
                // its row, the Handoff is not compacted (one line per field)
                // and the TODO is not cut.
                let reduced = plan(
                    view,
                    ok,
                    Rect {
                        height: spare,
                        ..area
                    },
                );
                let compact = reduced.handoff.is_some_and(|(d, _)| d == Density::Compact);
                let todo_whole = ok.todo.is_empty() || reduced.todo == ok.todo.len() as u16 + 4;
                !compact && todo_whole
            }
        },
    }
}

fn not_relay_card(view: &View, area: Rect, buf: &mut Buffer) {
    let inner = card(
        buf,
        area,
        &plain_title(if view.language == crate::language::Language::En {
            "Not a Relay workspace"
        } else {
            "Não é um workspace Relay"
        }),
        &[],
        theme::DIM,
    );
    put(
        buf,
        inner,
        0,
        &[seg(view.workspace, theme::color(theme::META))],
    );
    put(
        buf,
        inner,
        2,
        &[seg(
            if view.language == crate::language::Language::En {
                "No .orchestration/ directory here."
            } else {
                "Não há .orchestration/ neste diretório."
            },
            theme::fg(),
        )],
    );
    put(
        buf,
        inner,
        3,
        &[seg(
            if view.language == crate::language::Language::En {
                "Still watching: it will appear when created."
            } else {
                "Continuo vigiando: ele aparece quando for criado."
            },
            theme::color(theme::META),
        )],
    );
}

/// Rows one violation takes: its check, its detail (at most 3 lines) and its
/// records.
fn violation_height(v: &Violation, width: usize) -> usize {
    2 + wrap_capped(
        &violation_params(v, crate::language::Language::PtBr),
        width,
        3,
    )
    .len()
}

fn violation_params(v: &Violation, language: crate::language::Language) -> String {
    if language == crate::language::Language::En {
        let check = match v.check.as_str() {
            "handoff-names-no-pending-todo" => "The handoff does not name a pending TODO item",
            "backlog-id-mismatch" => "The handoff backlog ID does not match the active item",
            "handoff-status-invalid" => "The handoff status is invalid",
            "duplicate-id" => "An ID appears more than once",
            "needs-unknown-id" => "A dependency names an unknown backlog item",
            "needs-cycle" => "Backlog dependencies contain a cycle",
            "needs-incomplete-on-done" => "A completed item has an incomplete dependency",
            "needs-dropped-entry" => "An item depends on a dropped entry",
            "criteria-without-evidence" => "Completed criteria have no evidence",
            "dropped-without-reason" => "A dropped entry has no reason",
            "waived-without-drop" => "A waiver has no dropped criterion",
            "changelog-spec-mismatch" => "The changelog names a different spec",
            "spec-path-mismatch" => "The spec path does not match its ID",
            "todo-cleared-before-changelog" => "The TODO was cleared before its changelog entry",
            "handoff-updated-invalid" => "The handoff timestamp is invalid",
            "unknown-marker" => "The entry has an unknown marker",
            _ => "Relay integrity check failed",
        };
        return format!(
            "{check}: {}",
            v.params
                .iter()
                .map(|(k, value)| format!("{k}={value}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    v.params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Rows the whole list takes, with a blank row between violations.
fn violation_rows(violations: &[Violation], width: usize) -> usize {
    let heights: usize = violations.iter().map(|v| violation_height(v, width)).sum();
    heights + violations.len().saturating_sub(1)
}

fn violations_card(
    violations: &[Violation],
    language: crate::language::Language,
    area: Rect,
    buf: &mut Buffer,
) {
    let right = vec![seg(
        format!(" {} ", violations.len()),
        theme::color(theme::META),
    )];
    let title = vec![
        seg(" ● ", theme::bold(theme::RED)),
        seg(
            format!(
                "{} ",
                if language == crate::language::Language::En {
                    "Inconsistent"
                } else {
                    theme::INCONSISTENT_LABEL
                }
            ),
            theme::bold(theme::RED),
        ),
    ];
    let inner = card(buf, area, &title, &right, theme::RED);
    let capacity = inner.height as usize;
    let mut row = 0;
    for (index, v) in violations.iter().enumerate() {
        let detail = wrap_capped(&violation_params(v, language), inner.width as usize, 3);
        let height = violation_height(v, inner.width as usize);
        let is_last = index + 1 == violations.len();
        // One row stays free for "+N violações", unless this is the last one
        // and it fits as it is.
        let fits = if is_last {
            row + height <= capacity
        } else {
            row + height < capacity
        };
        if !fits {
            let hidden = violations.len() - index;
            put(
                buf,
                inner,
                row,
                &[seg(
                    format!(
                        "+{}",
                        if language == crate::language::Language::En {
                            plural(hidden, "violation", "violations")
                        } else {
                            plural(hidden, "violação", "violações")
                        }
                    ),
                    theme::color(theme::META),
                )],
            );
            return;
        }
        put(buf, inner, row, &[seg(&v.check, theme::bold(theme::RED))]);
        for (i, line) in detail.iter().enumerate() {
            put(buf, inner, row + 1 + i, &[seg(line, theme::fg())]);
        }
        put(
            buf,
            inner,
            row + 1 + detail.len(),
            &[seg(v.records.join(" · "), theme::color(theme::META))],
        );
        row += height + 1;
    }
}

// ------------------------------------------------------------------- ok

fn ok_body(view: &View, ok: &OkState, area: Rect, buf: &mut Buffer) {
    match ok.status {
        WorkStatus::Idle => {
            let inner = card(
                buf,
                Rect {
                    height: area.height.min(3),
                    ..area
                },
                &plain_title(if view.language == crate::language::Language::En {
                    "Idle"
                } else {
                    "Sem trabalho"
                }),
                &[],
                theme::DIM,
            );
            put(
                buf,
                inner,
                0,
                &[seg(
                    if view.language == crate::language::Language::En {
                        "No backlog, TODO, or handoff in this workspace."
                    } else {
                        "Nenhum backlog, TODO ou handoff neste workspace."
                    },
                    theme::fg(),
                )],
            );
        }
        WorkStatus::Done => done_card(ok, area, buf, view.language),
        WorkStatus::Backlog => backlog_body(view, ok, area, buf),
        WorkStatus::Ready | WorkStatus::InProgress | WorkStatus::Blocked => {
            work_body(view, ok, area, buf);
        }
    }
}

fn done_card(ok: &OkState, area: Rect, buf: &mut Buffer, language: crate::language::Language) {
    let (done, total) = if ok.backlog.is_empty() {
        (ok.completed, ok.total)
    } else {
        (
            ok.backlog.iter().filter(|e| e.marker == 'x').count(),
            work_total(ok),
        )
    };
    let en = language == crate::language::Language::En;
    let title = status_title(
        theme::status_label_in(WorkStatus::Done, language),
        theme::GREEN,
    );
    let inner = card(
        buf,
        Rect {
            height: area.height.min(3),
            ..area
        },
        &title,
        &[],
        theme::GREEN,
    );
    let source = if ok.backlog.is_empty() {
        if en { "in TODO" } else { "do TODO" }
    } else {
        if en { "in backlog" } else { "do backlog" }
    };
    put(
        buf,
        inner,
        0,
        &[
            seg("✓ ", theme::bold(theme::GREEN)),
            seg(
                if en {
                    format!("{done} of {total} {source} items completed.")
                } else {
                    format!("{done} de {total} itens {source} concluídos.")
                },
                theme::fg(),
            ),
        ],
    );
}

/// How the rows of a work body are shared: the Handoff card (its density and
/// height), the one-line status that stands in for it, the TODO card and the
/// Backlog (3 rows as a card, 1 as a line, 0 when absent).
#[derive(Clone, Copy, PartialEq, Eq)]
struct Plan {
    handoff: Option<(Density, u16)>,
    lead_line: u16,
    todo: u16,
    backlog: u16,
}

fn plan(view: &View, ok: &OkState, area: Rect) -> Plan {
    let mut remaining = area.height;
    let mut plan = Plan {
        handoff: None,
        lead_line: 0,
        todo: 0,
        backlog: 0,
    };
    // The TODO and the Backlog, whole; and the least they need to still say
    // something.
    let todo_full = if ok.todo.is_empty() {
        0
    } else {
        ok.todo.len() as u16 + 4
    };
    // The backlog area, whole: the current spec and the pending ones.
    let groups = specs::groups(ok, view.history);
    let backlog_full = specs::full_height(&groups);

    if let Some(handoff) = &ok.handoff {
        let inner_width = area.width.saturating_sub(4) as usize;
        let min_rest = if ok.todo.is_empty() { 0 } else { TODO_MIN } + u16::from(backlog_full > 0);
        // The handoff takes the most text that still leaves everything else
        // whole; failing that, the standard caps; failing that, one line per
        // field.
        let fits = |lines: &[Vec<Seg>], rest: u16| lines.len() as u16 + 2 + rest <= remaining;
        let mut density = Density::Full;
        let mut lines = handoff_lines(
            handoff,
            ok.status,
            inner_width,
            view.now_unix,
            density,
            view.language,
        );
        if !fits(&lines, todo_full + backlog_full) {
            density = Density::Standard;
            lines = handoff_lines(
                handoff,
                ok.status,
                inner_width,
                view.now_unix,
                density,
                view.language,
            );
            if !fits(&lines, min_rest) {
                density = Density::Compact;
                lines = handoff_lines(
                    handoff,
                    ok.status,
                    inner_width,
                    view.now_unix,
                    density,
                    view.language,
                );
            }
        }
        let height = (lines.len() as u16 + 2)
            .min(remaining.saturating_sub(min_rest).max(3))
            .min(remaining);
        plan.handoff = Some((density, height));
        remaining -= height;
    } else if ok.status == WorkStatus::Ready && remaining > 0 {
        // No Handoff card to carry the status, so the line carries it.
        plan.lead_line = 1;
        remaining -= 1;
    }

    if remaining == 0 {
        return plan;
    }

    // The backlog area gives way (pending specs to a line, the current spec cut,
    // both to the count line) before the TODO is cut.
    plan.backlog = if backlog_full == 0 {
        0
    } else {
        match remaining.saturating_sub(todo_full) {
            0 => 1.min(remaining),
            avail => specs::shape(&groups, avail).height().min(remaining),
        }
    };
    let todo_height = remaining.saturating_sub(plan.backlog).min(todo_full);
    if !ok.todo.is_empty() && todo_height >= TODO_MIN {
        plan.todo = todo_height;
    }
    plan
}

fn work_body(view: &View, ok: &OkState, area: Rect, buf: &mut Buffer) {
    let plan = plan(view, ok, area);
    let mut y = area.y;
    let at = |y: u16, height: u16| Rect { y, height, ..area };

    if let (Some(handoff), Some((density, height))) = (&ok.handoff, plan.handoff) {
        let tone = theme::status_tone(ok.status);
        let inner_width = area.width.saturating_sub(4) as usize;
        let mut lines = handoff_lines(
            handoff,
            ok.status,
            inner_width,
            view.now_unix,
            density,
            view.language,
        );
        lines.truncate(height.saturating_sub(2) as usize);
        let inner = card(
            buf,
            at(y, height),
            &plain_title("Handoff"),
            &status_title(theme::status_label_in(ok.status, view.language), tone),
            tone,
        );
        for (row, line) in lines.iter().enumerate() {
            put(buf, inner, row, line);
        }
        y += height;
    } else if plan.lead_line == 1 {
        let tone = theme::status_tone(ok.status);
        put(
            buf,
            at(y, 1),
            0,
            &[
                seg(" ● ", theme::bold(tone)),
                seg(
                    theme::status_label_in(ok.status, view.language),
                    theme::bold(tone),
                ),
                seg(
                    if view.language == crate::language::Language::En {
                        "  No active handoff"
                    } else {
                        "  Sem handoff ativo"
                    },
                    theme::color(theme::META),
                ),
            ],
        );
        y += 1;
    }

    if plan.todo > 0 {
        todo_card(ok, at(y, plan.todo), buf, view.language);
        y += plan.todo;
    }
    if plan.backlog > 0 {
        let groups = specs::groups(ok, view.history);
        let shape = specs::shape(&groups, plan.backlog);
        specs::draw(
            &groups,
            ok,
            shape,
            at(y, plan.backlog),
            buf,
            |a, b| backlog_line(ok, a, b, view.language),
            view.language,
        );
    }
}

/// How much of the handoff text is shown: as much as the height allows, down
/// to one line per field.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Density {
    /// Room for everything else too: the fields nearly whole.
    Full,
    Standard,
    Compact,
}

impl Density {
    /// Lines for the objective, the next step and the blocker.
    fn caps(self) -> (usize, usize, usize) {
        match self {
            Density::Full => (10, 10, 12),
            Density::Standard => (3, 3, 4),
            Density::Compact => (1, 1, 2),
        }
    }
}

fn handoff_lines(
    h: &Handoff,
    status: WorkStatus,
    width: usize,
    now: i64,
    density: Density,
    language: crate::language::Language,
) -> Vec<Vec<Seg>> {
    const LABEL: usize = 10;
    let dim = theme::color(theme::META);
    let mut lines = vec![vec![
        seg(&h.backlog_id, theme::bold(theme::ID)),
        seg(" · ", dim),
        seg(&h.todo_id, theme::color(theme::ID)),
        seg(
            format!(
                " · {} · {}",
                h.harness,
                relative_time_in(&h.updated, now, language)
            ),
            dim,
        ),
    ]];
    if density != Density::Compact {
        lines.push(vec![]);
    }
    let (objective, next, blocker) = density.caps();
    let mut field = |label: &str, text: &str, cap: usize| {
        let wrapped = wrap_capped(text, width.saturating_sub(LABEL), cap);
        let wrapped = if wrapped.is_empty() {
            vec!["—".to_string()]
        } else {
            wrapped
        };
        for (i, line) in wrapped.iter().enumerate() {
            let lead = if i == 0 {
                format!("{label:<LABEL$}")
            } else {
                " ".repeat(LABEL)
            };
            lines.push(vec![seg(lead, dim), seg(line, theme::fg())]);
        }
    };
    field(
        if language == crate::language::Language::En {
            "Objective"
        } else {
            "Objetivo"
        },
        &h.objective,
        objective,
    );
    field(
        if language == crate::language::Language::En {
            "Next"
        } else {
            "Próximo"
        },
        &h.next_step,
        next,
    );
    if status == WorkStatus::Blocked {
        field(
            if language == crate::language::Language::En {
                "Block"
            } else {
                "Bloqueio"
            },
            &h.context,
            blocker,
        );
    }
    lines
}

fn entry_tone(e: &ChecklistEntry) -> Color {
    match e.marker {
        'x' | '•' => theme::GREEN,
        '!' => theme::YELLOW,
        _ if e.available => theme::BLUE,
        _ => theme::BAR_EMPTY,
    }
}

fn todo_card(ok: &OkState, area: Rect, buf: &mut Buffer, language: crate::language::Language) {
    let inner = card(
        buf,
        area,
        &plain_title("TODO"),
        &count_title(ok.completed, ok.total),
        theme::DIM,
    );
    let w = inner.width as usize;

    // One segment per item, in the tone of its state: a count, never a percentage.
    let n = ok.todo.len().min(w.max(1));
    let gap = usize::from(n * 2 - 1 <= w);
    let seg_w = ((w + gap) / n).saturating_sub(gap).clamp(1, 8);
    let mut bar: Vec<Seg> = Vec::new();
    for (i, entry) in ok.todo.iter().take(n).enumerate() {
        if i > 0 && gap == 1 {
            bar.push(seg(" ", Style::new()));
        }
        bar.push(seg("█".repeat(seg_w), theme::color(entry_tone(entry))));
    }
    put(buf, inner, 0, &bar);

    let first = if inner.height >= 4 { 2 } else { 1 };
    let rows = (inner.height as usize).saturating_sub(first);
    let shown = if ok.todo.len() <= rows {
        ok.todo.len()
    } else {
        rows.saturating_sub(1)
    };
    for (i, entry) in ok.todo.iter().take(shown).enumerate() {
        put(
            buf,
            inner,
            first + i,
            &todo_item(entry, &ok.todo, w, language),
        );
    }
    if shown < ok.todo.len() && rows > 0 {
        let hidden = &ok.todo[shown..];
        let mut segs = vec![seg(
            format!(
                "+{}",
                if language == crate::language::Language::En {
                    plural(hidden.len(), "item", "items")
                } else {
                    plural(hidden.len(), "item", "itens")
                }
            ),
            theme::color(theme::META),
        )];
        if let Some(active) = hidden.iter().find(|e| e.marker == '•') {
            segs.push(seg(
                format!(" · ● {} em andamento", active.id),
                theme::color(theme::META),
            ));
        }
        put(buf, inner, first + shown, &segs);
    }
}

fn todo_item(
    e: &ChecklistEntry,
    all: &[ChecklistEntry],
    width: usize,
    language: crate::language::Language,
) -> Vec<Seg> {
    let (glyph, glyph_style, text_style) = match e.marker {
        'x' => ("✓", theme::bold(theme::GREEN), theme::color(theme::META)),
        '•' => ("●", theme::bold(theme::GREEN), theme::bold(theme::FG)),
        '!' => ("!", theme::bold(theme::YELLOW), theme::fg()),
        _ if e.available => ("○", theme::color(theme::BLUE), theme::fg()),
        _ => ("◌", theme::color(theme::META), theme::color(theme::META)),
    };
    // Which needs are still open is a lookup in the same list; whether the
    // item is available at all was decided by the core.
    let suffix = match e.marker {
        '!' => if language == crate::language::Language::En {
            "  blocked"
        } else {
            "  bloqueado"
        }
        .to_string(),
        ' ' if !e.available => {
            let open: Vec<&str> = e
                .needs
                .iter()
                .filter(|n| all.iter().any(|o| &o.id == *n && o.marker != 'x'))
                .map(String::as_str)
                .collect();
            if open.is_empty() {
                String::new()
            } else {
                format!(
                    "  {} {}",
                    if language == crate::language::Language::En {
                        "after"
                    } else {
                        "após"
                    },
                    open.join(", ")
                )
            }
        }
        _ => String::new(),
    };
    let head = [
        seg(format!("{glyph} "), glyph_style),
        seg(format!("{} ", e.id), theme::color(theme::ID)),
    ];
    let used = seg_width(&head) + width_of(&suffix);
    let text = truncate(&e.text, width.saturating_sub(used));
    let mut segs = head.to_vec();
    segs.push(seg(text, text_style));
    if !suffix.is_empty() {
        segs.push(seg(suffix, theme::color(theme::YELLOW)));
    }
    segs
}

fn width_of(s: &str) -> usize {
    width(s)
}

// --------------------------------------------------------------- backlog

/// The backlog entries that are work to do or done: a dropped one is neither.
fn work_total(ok: &OkState) -> usize {
    ok.backlog.iter().filter(|e| e.marker != '-').count()
}

struct Counts {
    done: usize,
    active: usize,
    blocked: usize,
    available: usize,
    waiting: usize,
}

fn counts(ok: &OkState) -> Counts {
    let mut c = Counts {
        done: 0,
        active: 0,
        blocked: 0,
        available: 0,
        waiting: 0,
    };
    for e in &ok.backlog {
        match e.marker {
            'x' => c.done += 1,
            '•' => c.active += 1,
            '!' => c.blocked += 1,
            '-' => {}
            _ if e.available => c.available += 1,
            _ => c.waiting += 1,
        }
    }
    c
}

fn backlog_line(ok: &OkState, area: Rect, buf: &mut Buffer, language: crate::language::Language) {
    let c = counts(ok);
    let mut segs = vec![
        seg(" Backlog", theme::bold(theme::FG)),
        seg(
            format!(" {}/{}", c.done, work_total(ok)),
            theme::color(theme::META),
        ),
    ];
    for (n, one, many) in [
        (
            c.active,
            if language == crate::language::Language::En {
                "in progress"
            } else {
                "em curso"
            },
            if language == crate::language::Language::En {
                "in progress"
            } else {
                "em curso"
            },
        ),
        (
            c.blocked,
            if language == crate::language::Language::En {
                "blocked"
            } else {
                "bloqueado"
            },
            if language == crate::language::Language::En {
                "blocked"
            } else {
                "bloqueados"
            },
        ),
        (
            c.available,
            if language == crate::language::Language::En {
                "available"
            } else {
                "disponível"
            },
            if language == crate::language::Language::En {
                "available"
            } else {
                "disponíveis"
            },
        ),
        (
            c.waiting,
            if language == crate::language::Language::En {
                "waiting"
            } else {
                "aguardando"
            },
            if language == crate::language::Language::En {
                "waiting"
            } else {
                "aguardando"
            },
        ),
    ] {
        let part = seg(
            format!(" · {}", plural(n, one, many)),
            theme::color(theme::META),
        );
        // A count that does not fit whole is left out, not cut in half.
        if n > 0 && seg_width(&segs) + width(&part.0) <= area.width as usize {
            segs.push(part);
        }
    }
    put(buf, area, 0, &segs);
}

/// The `backlog` status ("A escolher"): no TODO and no handoff. A line carries
/// the status, as for `ready`, and the current spec and the pending ones show what
/// can be picked.
fn backlog_body(view: &View, ok: &OkState, area: Rect, buf: &mut Buffer) {
    if area.height == 0 {
        return;
    }
    let tone = theme::status_tone(WorkStatus::Backlog);
    put(
        buf,
        Rect { height: 1, ..area },
        0,
        &[
            seg(" ● ", theme::bold(tone)),
            seg(
                theme::status_label_in(WorkStatus::Backlog, view.language),
                theme::bold(tone),
            ),
            seg(
                if view.language == crate::language::Language::En {
                    "  No active handoff"
                } else {
                    "  Sem handoff ativo"
                },
                theme::color(theme::META),
            ),
        ],
    );
    let rest = Rect {
        y: area.y + 1,
        height: area.height - 1,
        ..area
    };
    if rest.height == 0 {
        return;
    }
    let groups = specs::groups(ok, view.history);
    let shape = specs::shape(&groups, rest.height);
    specs::draw(
        &groups,
        ok,
        shape,
        Rect {
            height: shape.height(),
            ..rest
        },
        buf,
        |a, b| backlog_line(ok, a, b, view.language),
        view.language,
    );
}
