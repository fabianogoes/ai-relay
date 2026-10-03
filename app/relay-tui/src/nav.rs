//! Navigation state of the Histórico view (ADR-0004): which view is open, which
//! level, which row is selected at each level, and how an input changes that.
//!
//! Local screen state and nothing else. No disk, no terminal: the lists it
//! walks come in as a `Ctx` (the extracted `History` and the derived `OkState`),
//! and an input comes back as an `Effect` the app carries out. A selection is
//! kept by id, so it survives the lists being rebuilt after a reload.

use crate::core::{History, OkState};

/// The two views. Agora is the one that opens; Histórico is the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Now,
    History,
}

/// The four levels of Histórico, each deepening the one before. `Detail` shows
/// the task selected at `Tasks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Specs,
    Items,
    Tasks,
    Detail,
}

/// What the terminal reports, with the app's key mapping already applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Up,
    Down,
    PageUp,
    PageDown,
    WheelUp,
    WheelDown,
    Enter,
    Esc,
    Backspace,
    /// `Tab` or `t`.
    Toggle,
    /// `r`.
    Reload,
    /// `q` or `Ctrl-C`.
    Quit,
    /// `y`: confirms leaving, when asked.
    Confirm,
    /// Any other key. It does nothing, except that it dismisses the question
    /// "leave?" that `Esc` in Agora asks.
    Other,
    /// A click on the list row with this index (the view turns the screen
    /// coordinate into a row).
    Click(usize),
}

/// What the app must do after an input. Navigation never does it itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    None,
    Quit,
    /// Read the workspace again as a whole; the selection survives by id.
    Reload,
}

/// The identity of a row, stable across a reload: a spec by its file, a backlog
/// item by its id, a changelog record by its task id and which occurrence of it
/// the item has (an append-only correction repeats the id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    Spec(String),
    NoSpec,
    Item(String),
    Record { todo_id: String, nth: usize },
    /// A TODO item that has no changelog record yet.
    Pending(String),
}

/// Where a row's content lives: an index into the data of `Ctx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowRef {
    /// `History::specs[i]`.
    Spec(usize),
    NoSpec,
    /// An item of the open spec (or of the Sem spec group).
    Item(usize),
    /// `History::records[i]`.
    Record(usize),
    /// `OkState::todo[i]`.
    Pending(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: RowKey,
    pub at: RowRef,
}

/// The data the lists are built from.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub history: &'a History,
    /// Absent while the records are `inconsistent` or not a workspace.
    pub ok: Option<&'a OkState>,
}

/// The selection at one list level: the key, and the row it was last at, which
/// is where it falls back to when the key is gone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Sel {
    key: Option<RowKey>,
    index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nav {
    pane: Pane,
    level: Level,
    /// Specs, items and tasks; `Detail` reads the tasks selection.
    sel: [Sel; 3],
    scroll: usize,
    detail_max: usize,
    page: usize,
    /// `Esc` in Agora asked "leave?" and the answer is pending.
    confirming: bool,
}

impl Default for Nav {
    fn default() -> Self {
        Nav::new()
    }
}

impl Nav {
    pub fn new() -> Self {
        Nav {
            pane: Pane::Now,
            level: Level::Specs,
            sel: Default::default(),
            scroll: 0,
            detail_max: 0,
            page: 10,
            confirming: false,
        }
    }

    /// Whether the app is asking the user to confirm leaving (after `Esc` in
    /// Agora). `Esc`, `Enter` and `y` confirm; any other key cancels; `q` and
    /// `Ctrl-C` leave at once, as they are explicit.
    pub fn confirming_quit(&self) -> bool {
        self.confirming
    }

    pub fn pane(&self) -> Pane {
        self.pane
    }

    pub fn level(&self) -> Level {
        self.level
    }

    /// The selected row of a list level, if the list is not empty.
    pub fn selected(&self, level: Level) -> Option<&RowKey> {
        self.sel[depth(level)].key.as_ref()
    }

    /// The index of the selected row of a list level (`Detail` reads `Tasks`).
    pub fn selected_index(&self, level: Level) -> usize {
        self.sel[depth(level)].index
    }

    /// How far the detail is scrolled, in lines.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Rows a page key moves; the view sets it from the height of the list.
    pub fn set_page(&mut self, rows: usize) {
        self.page = rows.max(1);
    }

    /// The farthest the detail can scroll (its wrapped lines minus the height),
    /// which only the view knows.
    pub fn set_detail_max(&mut self, max: usize) {
        self.detail_max = max;
        self.scroll = self.scroll.min(max);
    }

    /// The rows of `level` for the current parents. `Detail` has the rows of
    /// `Tasks`: it shows one of them.
    pub fn rows(&self, ctx: &Ctx, level: Level) -> Vec<Row> {
        let sel = &self.sel;
        match depth(level) {
            0 => {
                let mut rows: Vec<Row> = ctx
                    .history
                    .specs
                    .iter()
                    .enumerate()
                    .map(|(i, s)| Row { key: RowKey::Spec(s.path.clone()), at: RowRef::Spec(i) })
                    .collect();
                if !ctx.history.no_spec.is_empty() {
                    rows.push(Row { key: RowKey::NoSpec, at: RowRef::NoSpec });
                }
                rows
            }
            1 => {
                let items = match &sel[0].key {
                    Some(RowKey::Spec(path)) => {
                        ctx.history.specs.iter().find(|s| &s.path == path).map(|s| &s.items)
                    }
                    Some(RowKey::NoSpec) => Some(&ctx.history.no_spec),
                    _ => None,
                };
                items
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(i, item)| Row { key: RowKey::Item(item.id.clone()), at: RowRef::Item(i) })
                    .collect()
            }
            _ => {
                let Some(RowKey::Item(id)) = &sel[1].key else {
                    return Vec::new();
                };
                let mut rows = Vec::new();
                let mut seen: Vec<&str> = Vec::new();
                for (i, record) in ctx.history.records.iter().enumerate() {
                    if record.backlog.as_deref() != Some(id.as_str()) {
                        continue;
                    }
                    let nth = seen.iter().filter(|t| **t == record.todo_id).count();
                    seen.push(&record.todo_id);
                    rows.push(Row {
                        key: RowKey::Record { todo_id: record.todo_id.clone(), nth },
                        at: RowRef::Record(i),
                    });
                }
                // The TODO items of the current task that have no record yet.
                if let Some(ok) = ctx.ok.filter(|ok| ok.active_backlog_id.as_deref() == Some(id)) {
                    for (i, entry) in ok.todo.iter().enumerate() {
                        if !seen.contains(&entry.id.as_str()) {
                            rows.push(Row { key: RowKey::Pending(entry.id.clone()), at: RowRef::Pending(i) });
                        }
                    }
                }
                rows
            }
        }
    }

    /// Applies one input. The selection is first brought up to date with `ctx`,
    /// so an input never acts on a row that is gone.
    pub fn handle(&mut self, input: Input, ctx: &Ctx) -> Effect {
        self.reconcile(ctx);
        if self.confirming {
            // The question is answered by this key, whatever it is.
            self.confirming = false;
            return match input {
                Input::Esc | Input::Enter | Input::Confirm | Input::Quit => Effect::Quit,
                _ => Effect::None,
            };
        }
        match input {
            Input::Quit => return Effect::Quit,
            Input::Reload => return Effect::Reload,
            Input::Confirm | Input::Other => {}
            Input::Toggle => {
                self.pane = match self.pane {
                    Pane::Now => Pane::History,
                    Pane::History => Pane::Now,
                };
            }
            // `Esc` in Agora asks before leaving; `Backspace` does nothing there.
            Input::Esc if self.pane == Pane::Now => self.confirming = true,
            _ if self.pane == Pane::Now => {}
            Input::Esc | Input::Backspace => self.back(),
            Input::Enter => self.open(ctx),
            Input::Up | Input::WheelUp => self.step(ctx, -1),
            Input::Down | Input::WheelDown => self.step(ctx, 1),
            Input::PageUp => self.step(ctx, -(self.page as isize)),
            Input::PageDown => self.step(ctx, self.page as isize),
            Input::Click(row) => self.click(ctx, row),
        }
        Effect::None
    }

    /// One level back; from the first level, back to Agora.
    fn back(&mut self) {
        match self.level {
            Level::Specs => self.pane = Pane::Now,
            Level::Items => self.level = Level::Specs,
            Level::Tasks => self.level = Level::Items,
            Level::Detail => {
                self.level = Level::Tasks;
                self.scroll = 0;
            }
        }
    }

    /// Opens the next level with the selected row, whose own list starts at its
    /// first row. At `Detail`, and on an empty list, nothing happens.
    fn open(&mut self, ctx: &Ctx) {
        let next = match self.level {
            Level::Specs => Level::Items,
            Level::Items => Level::Tasks,
            Level::Tasks => Level::Detail,
            Level::Detail => return,
        };
        if self.sel[depth(self.level)].key.is_none() {
            return;
        }
        self.level = next;
        self.scroll = 0;
        if next != Level::Detail {
            let first = self.rows(ctx, next).into_iter().next();
            self.sel[depth(next)] = Sel { key: first.map(|r| r.key), index: 0 };
        }
    }

    /// Moves the selection by `delta` rows, never past the first or the last;
    /// in the detail it scrolls instead.
    fn step(&mut self, ctx: &Ctx, delta: isize) {
        if self.level == Level::Detail {
            let to = (self.scroll as isize + delta).clamp(0, self.detail_max as isize);
            self.scroll = to as usize;
            return;
        }
        let rows = self.rows(ctx, self.level);
        if rows.is_empty() {
            return;
        }
        let at = &mut self.sel[depth(self.level)];
        let to = (at.index as isize + delta).clamp(0, rows.len() as isize - 1) as usize;
        *at = Sel { key: Some(rows[to].key.clone()), index: to };
    }

    /// A click on a row selects it and opens the next level. Outside the rows
    /// (and in the detail) it is ignored.
    fn click(&mut self, ctx: &Ctx, row: usize) {
        if self.level == Level::Detail {
            return;
        }
        let rows = self.rows(ctx, self.level);
        let Some(hit) = rows.get(row) else {
            return;
        };
        self.sel[depth(self.level)] = Sel { key: Some(hit.key.clone()), index: row };
        self.open(ctx);
    }

    /// Brings every selection in line with `ctx` after the lists were rebuilt.
    ///
    /// A selection is found again by its key. If its row is gone, it falls to the
    /// nearest row of the same list; and if that was a parent of the open level,
    /// the lists below it no longer exist, so the view goes back up to the level
    /// that does.
    pub fn reconcile(&mut self, ctx: &Ctx) {
        let open = depth(self.level);
        for d in 0..=open {
            let level = [Level::Specs, Level::Items, Level::Tasks][d];
            let rows = self.rows(ctx, level);
            let at = &mut self.sel[d];
            let found = at.key.as_ref().and_then(|k| rows.iter().position(|r| &r.key == k));
            let moved = match found {
                Some(i) => {
                    at.index = i;
                    false
                }
                None if rows.is_empty() => {
                    let moved = at.key.is_some();
                    *at = Sel::default();
                    moved
                }
                None => {
                    let moved = at.key.is_some();
                    let i = at.index.min(rows.len() - 1);
                    *at = Sel { key: Some(rows[i].key.clone()), index: i };
                    moved
                }
            };
            if d < open && (moved || self.sel[d].key.is_none()) {
                // The parent of the open level is gone: back to its own list.
                self.level = level;
                self.scroll = 0;
                return;
            }
        }
        if self.level == Level::Detail && self.sel[2].key.is_none() {
            self.level = Level::Tasks;
            self.scroll = 0;
        }
    }
}

fn depth(level: Level) -> usize {
    match level {
        Level::Specs => 0,
        Level::Items => 1,
        Level::Tasks | Level::Detail => 2,
    }
}
