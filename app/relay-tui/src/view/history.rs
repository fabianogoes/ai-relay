//! The Histórico view (ADR-0004): one card with the open level of the
//! navigation, a list of one-line rows or the detail of a task.
//!
//! A pure function of the `Nav`, the extracted `History` and the derived state;
//! the layout rules (levels, rows, scroll, widths) are in
//! `DESIGN.md`, "Visão Histórico".

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::core::{HistoryItem, TaskRecord};
use crate::language::Language;
use crate::nav::{Ctx, Level, Nav, Row, RowKey, RowRef};
use crate::theme;

use super::cards::{Seg, card, plain_title, put, seg, seg_width};
use super::text::{truncate, width, wrap};
use super::{Freshness, MIN_CARD_WIDTH, MIN_HEIGHT};

/// What the Histórico view draws.
pub struct HistoryScreen<'a> {
    pub workspace: &'a str,
    pub freshness: Freshness,
    pub language: Language,
    pub nav: &'a Nav,
    pub ctx: Ctx<'a>,
}

/// The width and the number of rows inside the card for a terminal of this
/// size, or `None` when only the header and the footer (or the width notice)
/// fit. The navigation needs it for the page keys and the detail's limit.
pub fn list_geometry(width: u16, height: u16) -> Option<(usize, usize)> {
    if width < MIN_CARD_WIDTH || height < MIN_HEIGHT {
        return None;
    }
    let body = super::layout(Rect::new(0, 0, width, height)).body.height;
    // Frame (2) plus at least one row.
    (body >= 3).then(|| ((width - 4) as usize, (body - 2) as usize))
}

/// The list row under a click at `(column, row)` on a terminal of this size, if
/// any: inside the card's borders, on a row that is drawn, at a list level. The
/// header, the footer, the frame and the empty area below the last row are not
/// rows; nor is anything in the detail.
pub fn row_at(
    nav: &Nav,
    ctx: &Ctx,
    width: u16,
    height: u16,
    column: u16,
    row: u16,
) -> Option<usize> {
    if nav.level() == Level::Detail {
        return None;
    }
    let (_, rows) = list_geometry(width, height)?;
    let top = super::layout(Rect::new(0, 0, width, height)).body.y;
    // Inside the left and right border, and below the top border.
    if column == 0 || column + 1 >= width || row <= top {
        return None;
    }
    let line = (row - top - 1) as usize;
    if line >= rows {
        return None;
    }
    let len = nav.rows(ctx, nav.level()).len();
    let index = window_start(len, rows, nav.selected_index(nav.level())) + line;
    (index < len).then_some(index)
}

/// How far the detail of the selected task can scroll in a terminal of this
/// size: its wrapped lines minus the rows that are visible.
pub fn detail_extent(nav: &Nav, ctx: &Ctx, width: u16, height: u16) -> usize {
    let Some((inner_width, rows)) = list_geometry(width, height) else {
        return 0;
    };
    detail_lines(nav, ctx, inner_width, Language::PtBr)
        .len()
        .saturating_sub(rows)
}

impl Widget for &HistoryScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let places = super::layout(area);
        super::header(
            self.workspace,
            self.freshness,
            self.language,
            places.header,
            buf,
        );
        if area.height == 1 {
            return;
        }
        let hints: &[super::Hint] = if self.nav.level() == Level::Detail {
            &super::FOOTER_DETAIL
        } else {
            &super::FOOTER_LIST
        };
        super::footer(hints, self.language, places, buf);
        if area.height < MIN_HEIGHT {
            return;
        }
        let body = places.body;
        if area.width < MIN_CARD_WIDTH {
            // The sentence is 31 columns: on a narrower terminal it wraps by
            // words rather than ending in `…` halfway through its meaning.
            let room = Rect {
                x: body.x + 1,
                width: body.width.saturating_sub(1),
                ..body
            };
            for (row, text) in wrap(
                if self.language == Language::En {
                    "History needs 40 columns"
                } else {
                    "Histórico precisa de 40 colunas"
                },
                room.width as usize,
            )
            .iter()
            .enumerate()
            {
                put(
                    buf,
                    room,
                    row,
                    &[seg(text.clone(), theme::color(theme::META))],
                );
            }
            return;
        }
        if body.height < 3 {
            return;
        }
        self.card(body, buf);
    }
}

impl HistoryScreen<'_> {
    fn card(&self, area: Rect, buf: &mut Buffer) {
        let level = self.nav.level();
        let title = self.title(level);
        let right_room = (area.width as usize).saturating_sub(2 + seg_width(&title) + 1);
        // The indicator is decided once the rows that fit are known.
        let rows = (area.height as usize).saturating_sub(2);
        let inner_width = (area.width as usize).saturating_sub(4);

        let (lines, scroll, empty) = if level == Level::Detail {
            let lines = detail_lines(self.nav, &self.ctx, inner_width, self.language);
            let scroll = self.nav.scroll().min(lines.len().saturating_sub(rows));
            (lines, scroll, None)
        } else {
            let selected = self.nav.selected_index(level);
            let all = self.list_lines(level, inner_width);
            let start = window_start(all.len(), rows, selected);
            let empty = all
                .is_empty()
                .then(|| self.empty_notice(level, inner_width));
            (all, start, empty)
        };
        let end = (scroll + rows).min(lines.len());
        let above = scroll;
        let below = lines.len() - end;

        let indicator = scroll_indicator(above, below, right_room, self.language);
        let inner = card(buf, area, &title, &indicator, theme::DIM);
        if let Some(notice) = empty {
            for (i, text) in notice.into_iter().take(rows).enumerate() {
                put(buf, inner, i, &[seg(text, theme::color(theme::META))]);
            }
            return;
        }
        for (i, line) in lines[scroll..end].iter().enumerate() {
            put(buf, inner, i, line);
        }
    }

    fn title(&self, level: Level) -> Vec<Seg> {
        let bold = theme::bold(theme::FG);
        let id = |text: String| seg(text, theme::bold(theme::ID));
        let en = self.language == Language::En;
        match level {
            Level::Specs => plain_title("Specs"),
            Level::Items => {
                let parent = match self.nav.selected(Level::Specs) {
                    Some(RowKey::Spec(path)) => id(crate::core::spec_id(path)),
                    Some(RowKey::NoSpec) => seg(if en { "No spec" } else { "Sem spec" }, bold),
                    _ => seg("", bold),
                };
                vec![
                    seg(if en { " Items · " } else { " Itens · " }, bold),
                    parent,
                    seg(" ", bold),
                ]
            }
            Level::Tasks | Level::Detail => {
                let (label, key) = if level == Level::Tasks {
                    (
                        if en { " Tasks · " } else { " Tarefas · " },
                        self.nav.selected(Level::Items),
                    )
                } else {
                    (
                        if en { " Task · " } else { " Tarefa · " },
                        self.nav.selected(Level::Tasks),
                    )
                };
                let name = match key {
                    Some(RowKey::Item(id)) | Some(RowKey::Pending(id)) => id.clone(),
                    Some(RowKey::Record { todo_id, .. }) => todo_id.clone(),
                    _ => String::new(),
                };
                vec![seg(label, bold), id(name), seg(" ", bold)]
            }
        }
    }

    fn list_lines(&self, level: Level, width: usize) -> Vec<Vec<Seg>> {
        let selected = self.nav.selected_index(level);
        let items = self.parent_items();
        self.nav
            .rows(&self.ctx, level)
            .iter()
            .enumerate()
            .map(|(i, row)| match level {
                Level::Specs => self.spec_row(row, i == selected, width),
                Level::Items => self.item_row(row, items, i == selected, width),
                _ => self.task_row(row, i == selected, width),
            })
            .collect()
    }

    /// The backlog items of the spec open at the items level.
    fn parent_items(&self) -> &[HistoryItem] {
        match self.nav.selected(Level::Specs) {
            Some(RowKey::Spec(path)) => self
                .ctx
                .history
                .specs
                .iter()
                .find(|s| &s.path == path)
                .map_or(&[][..], |s| &s.items[..]),
            Some(RowKey::NoSpec) => &self.ctx.history.no_spec,
            _ => &[],
        }
    }

    /// What a level with no rows says. A dropped item says why, wrapped by the
    /// width and never cut.
    fn empty_notice(&self, level: Level, width: usize) -> Vec<String> {
        if self.language == Language::En {
            return match level {
                Level::Specs => vec!["No specs in .specs/".into()],
                Level::Items => vec!["No backlog items in this spec".into()],
                _ => vec!["No task record for this item".into()],
            };
        }
        match level {
            Level::Specs => vec!["Nenhuma spec em .specs/".into()],
            Level::Items => vec!["Nenhum item de backlog nesta spec".into()],
            _ => {
                let dropped = match self.nav.selected(Level::Items) {
                    Some(RowKey::Item(id)) => self
                        .parent_items()
                        .iter()
                        .find(|i| &i.id == id && i.is_dropped()),
                    _ => None,
                };
                match dropped {
                    Some(item) => {
                        let reason = item.dropped.as_deref().unwrap_or("").trim();
                        wrap(
                            &format!(
                                "{}: {reason}",
                                if self.language == Language::En {
                                    "Dropped"
                                } else {
                                    "Descartado"
                                }
                            ),
                            width,
                        )
                    }
                    None => vec![if self.language == Language::En {
                        "No task record for this item".into()
                    } else {
                        "Nenhuma tarefa registrada para este item".into()
                    }],
                }
            }
        }
    }

    fn spec_row(&self, row: &Row, selected: bool, width: usize) -> Vec<Seg> {
        let (id, text, done, total) = match row.at {
            RowRef::Spec(i) => {
                let s = &self.ctx.history.specs[i];
                (Some(s.id.clone()), s.title.clone(), s.done(), s.total())
            }
            _ => {
                let items = &self.ctx.history.no_spec;
                let total = items.iter().filter(|i| !i.is_dropped()).count();
                (
                    None,
                    if self.language == Language::En {
                        "No spec".to_string()
                    } else {
                        "Sem spec".to_string()
                    },
                    items.iter().filter(|i| i.is_done()).count(),
                    total,
                )
            }
        };
        let closed = matches!(row.at, RowRef::Spec(i) if self.ctx.history.specs[i].closed);
        let count = format!("{done}/{total}");
        let right = vec![seg(
            if closed {
                if self.language == Language::En {
                    format!("closed · {count}")
                } else {
                    format!("fechada · {count}")
                }
            } else {
                count
            },
            theme::color(theme::META),
        )];
        line(
            selected,
            id.as_deref(),
            &text,
            text_style(selected, false),
            right,
            width,
        )
    }

    fn item_row(&self, row: &Row, items: &[HistoryItem], selected: bool, width: usize) -> Vec<Seg> {
        let RowRef::Item(i) = row.at else {
            return Vec::new();
        };
        let item = &items[i];
        let (glyph, tone, word) = self.item_state(item);
        // A dropped item is all `meta`: it is no longer work.
        let word_style = if item.is_dropped() {
            theme::color(theme::META)
        } else {
            theme::fg()
        };
        let right = vec![
            seg(format!("{glyph} "), theme::bold(tone)),
            seg(word, word_style),
        ];
        let finished = item.is_done() || item.is_dropped();
        line(
            selected,
            Some(&item.id),
            &item.text,
            text_style(selected, finished),
            right,
            width,
        )
    }

    /// The marker and word of a backlog item. Availability comes from the core's
    /// derived state; with no derived state (`inconsistent`) an open item can
    /// only be called pending.
    fn item_state(
        &self,
        item: &HistoryItem,
    ) -> (&'static str, ratatui::style::Color, &'static str) {
        let en = self.language == Language::En;
        match item.marker {
            'x' => ("✓", theme::GREEN, if en { "done" } else { "feito" }),
            '-' => ("×", theme::META, if en { "dropped" } else { "descartado" }),
            '•' => (
                "●",
                theme::GREEN,
                if en { "in progress" } else { "em curso" },
            ),
            '!' => ("!", theme::YELLOW, if en { "blocked" } else { "bloqueado" }),
            _ => match self
                .ctx
                .ok
                .and_then(|ok| ok.backlog.iter().find(|e| e.id == item.id))
            {
                Some(e) if e.available => (
                    "○",
                    theme::BLUE,
                    if en { "available" } else { "disponível" },
                ),
                Some(_) => ("◌", theme::META, if en { "waiting" } else { "aguardando" }),
                None => ("○", theme::META, if en { "pending" } else { "pendente" }),
            },
        }
    }

    fn task_row(&self, row: &Row, selected: bool, width: usize) -> Vec<Seg> {
        match row.at {
            RowRef::Record(i) => {
                let r = &self.ctx.history.records[i];
                let right = vec![seg(r.date.clone(), theme::color(theme::META))];
                line(
                    selected,
                    Some(&r.todo_id),
                    &r.title,
                    text_style(selected, false),
                    right,
                    width,
                )
            }
            RowRef::Pending(i) => {
                let Some(entry) = self.ctx.ok.and_then(|ok| ok.todo.get(i)) else {
                    return Vec::new();
                };
                let (glyph, tone) = todo_marker(entry.marker, entry.available);
                let right = vec![
                    seg(format!("{glyph} "), theme::bold(tone)),
                    seg(
                        if self.language == Language::En {
                            "no record"
                        } else {
                            "sem registro"
                        },
                        theme::color(theme::META),
                    ),
                ];
                line(
                    selected,
                    Some(&entry.id),
                    &entry.text,
                    text_style(selected, false),
                    right,
                    width,
                )
            }
            _ => Vec::new(),
        }
    }
}

/// The glyph and tone of a TODO item, the same as the TODO card of Agora.
fn todo_marker(marker: char, available: bool) -> (&'static str, ratatui::style::Color) {
    match marker {
        'x' => ("✓", theme::GREEN),
        '•' => ("●", theme::GREEN),
        '!' => ("!", theme::YELLOW),
        _ if available => ("○", theme::BLUE),
        _ => ("◌", theme::META),
    }
}

fn text_style(selected: bool, done: bool) -> Style {
    match (selected, done) {
        (true, _) => theme::bold(theme::FG),
        (false, true) => theme::color(theme::META),
        (false, false) => theme::fg(),
    }
}

/// One row of a list: the selection mark, the id, the text cut to what is left
/// and, flush right, the state. Always one line of `width` columns at most.
fn line(
    selected: bool,
    id: Option<&str>,
    text: &str,
    text_style: Style,
    right: Vec<Seg>,
    width: usize,
) -> Vec<Seg> {
    let mark = if selected {
        seg("▸ ", theme::bold(theme::BLUE))
    } else {
        seg("  ", Style::new())
    };
    let mut lead = vec![mark];
    if let Some(id) = id {
        lead.push(seg(format!("{id} "), theme::color(theme::ID)));
    }
    let lead_w = seg_width(&lead);
    let right_w = seg_width(&right);
    // Keep a gap before the state; when even a few characters of text do not
    // fit beside it, the state gives way before the text does.
    let (right, right_w) = if width >= lead_w + right_w + 1 + 4 {
        (right, right_w)
    } else {
        (Vec::new(), 0)
    };
    let room = width.saturating_sub(lead_w + if right_w > 0 { right_w + 1 } else { 0 });
    let text = truncate(text, room);
    let pad = width.saturating_sub(lead_w + super::text::width(&text) + right_w);
    let mut out = lead;
    out.push(seg(text, text_style));
    if right_w > 0 {
        out.push(seg(" ".repeat(pad), Style::new()));
        out.extend(right);
    }
    out
}

/// The first row shown so that `selected` stays visible, centred when there is
/// room both ways. Stateless: the same selection always gives the same window.
fn window_start(len: usize, rows: usize, selected: usize) -> usize {
    if rows == 0 || len <= rows {
        return 0;
    }
    selected.saturating_sub(rows / 2).min(len - rows)
}

/// `↑ 3 acima · ↓ 12 abaixo`, as much of it as fits in `room` columns: the
/// words, then the numbers alone, then nothing.
fn scroll_indicator(above: usize, below: usize, room: usize, language: Language) -> Vec<Seg> {
    let build = |words: bool| {
        let mut parts = Vec::new();
        if above > 0 {
            parts.push(if words {
                if language == Language::En {
                    format!("↑ {above} above")
                } else {
                    format!("↑ {above} acima")
                }
            } else {
                format!("↑{above}")
            });
        }
        if below > 0 {
            parts.push(if words {
                if language == Language::En {
                    format!("↓ {below} below")
                } else {
                    format!("↓ {below} abaixo")
                }
            } else {
                format!("↓{below}")
            });
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" {} ", parts.join(if words { " · " } else { " " }))
        }
    };
    for words in [true, false] {
        let text = build(words);
        if width(&text) <= room {
            return if text.is_empty() {
                Vec::new()
            } else {
                vec![seg(text, theme::color(theme::META))]
            };
        }
    }
    Vec::new()
}

// ---------------------------------------------------------------- detail

/// The lines of the selected task's detail, wrapped to `width`; never cut. A
/// task with a record shows its fields (an absent one is left out); one without
/// says so.
pub(super) fn detail_lines(
    nav: &Nav,
    ctx: &Ctx,
    width: usize,
    language: Language,
) -> Vec<Vec<Seg>> {
    let rows = nav.rows(ctx, Level::Tasks);
    let Some(row) = rows.get(nav.selected_index(Level::Tasks)) else {
        return Vec::new();
    };
    let meta = theme::color(theme::META);
    let mut lines: Vec<Vec<Seg>> = Vec::new();
    // `wrap` gives no line for empty text, so a blank row is pushed by hand.
    fn paragraph(lines: &mut Vec<Vec<Seg>>, text: &str, width: usize, style: Style) {
        for l in wrap(text, width) {
            lines.push(vec![seg(l, style)]);
        }
    }
    match row.at {
        RowRef::Record(i) => {
            let r: &TaskRecord = &ctx.history.records[i];
            paragraph(
                &mut lines,
                &format!("{} · {}", r.todo_id, r.title),
                width,
                theme::bold(theme::FG),
            );
            paragraph(&mut lines, &r.date, width, meta);
            let fields = [
                ("Backlog", &r.backlog),
                ("Spec", &r.spec),
                ("Result", &r.result),
                ("Evidence", &r.evidence),
                ("Criteria", &r.criteria),
                ("Decisions", &r.decisions),
            ];
            for (label, value) in fields {
                let Some(value) = value else { continue };
                lines.push(Vec::new());
                lines.push(vec![seg(label, theme::bold(theme::META))]);
                let style = if label == "Backlog" {
                    theme::color(theme::ID)
                } else {
                    theme::fg()
                };
                paragraph(&mut lines, value, width, style);
            }
        }
        RowRef::Pending(i) => {
            if let Some(entry) = ctx.ok.and_then(|ok| ok.todo.get(i)) {
                let (glyph, tone) = todo_marker(entry.marker, entry.available);
                lines.push(vec![
                    seg(format!("{glyph} "), theme::bold(tone)),
                    seg(entry.id.clone(), theme::bold(theme::ID)),
                ]);
                paragraph(&mut lines, &entry.text, width, theme::bold(theme::FG));
                lines.push(Vec::new());
                paragraph(
                    &mut lines,
                    if language == Language::En {
                        "No changelog record yet."
                    } else {
                        "Sem registro no changelog ainda."
                    },
                    width,
                    meta,
                );
            }
        }
        _ => {}
    }
    lines
}
