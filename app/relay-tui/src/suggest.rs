//! The next-step suggestion shown above the footer of the Agora view.
//!
//! A pure function of the state the core already delivered: no clock, no disk,
//! and no knowledge of how the sentence is worded, which is the view's job. It
//! lives outside `core` because the core holds only what `docs/PROTOCOL.md`
//! defines and the conformance suite checks; a suggestion is not protocol.

use crate::core::{ChecklistEntry, OkState, RelayState, WorkStatus};

/// Which row of the suggestion table matched. The table is tested in this
/// order and the first match wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    NotAWorkspace,
    Inconsistent,
    InProgress,
    BlockedWithHandoff,
    BlockedWithoutHandoff,
    Ready,
    DoneWithTodo,
    BacklogWithAvailable,
    BacklogWithoutAvailable,
    Done,
    Idle,
}

/// What to suggest: the case, the one skill to call, and the ids and title the
/// sentence names. Ids and title are `None` where the case has none to name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub case: Case,
    pub skill: &'static str,
    pub todo_id: Option<String>,
    pub backlog_id: Option<String>,
    pub title: Option<String>,
}

impl Suggestion {
    fn new(case: Case, skill: &'static str) -> Self {
        Suggestion { case, skill, todo_id: None, backlog_id: None, title: None }
    }

    fn backlog(mut self, id: Option<&String>) -> Self {
        self.backlog_id = id.cloned();
        self
    }

    fn entry(mut self, entry: &ChecklistEntry, is_todo: bool) -> Self {
        if is_todo {
            self.todo_id = Some(entry.id.clone());
        } else {
            self.backlog_id = Some(entry.id.clone());
        }
        self.title = Some(entry.text.clone());
        self
    }
}

/// `None` means the directory is not a Relay workspace (no `.orchestration/`).
pub fn suggest(state: Option<&RelayState>) -> Suggestion {
    match state {
        None => Suggestion::new(Case::NotAWorkspace, "relay-setup"),
        Some(RelayState::Inconsistent { .. }) => Suggestion::new(Case::Inconsistent, "relay-status"),
        Some(RelayState::Ok(ok)) => suggest_ok(ok),
    }
}

fn suggest_ok(ok: &OkState) -> Suggestion {
    let first_available = |list: &'_ [ChecklistEntry]| list.iter().find(|e| e.available).cloned();

    match ok.status {
        WorkStatus::InProgress | WorkStatus::Blocked if ok.handoff.is_some() => {
            let h = ok.handoff.as_ref().expect("guarded above");
            let case = if ok.status == WorkStatus::InProgress {
                Case::InProgress
            } else {
                Case::BlockedWithHandoff
            };
            let mut s = Suggestion::new(case, "relay-session").backlog(Some(&h.backlog_id));
            s.todo_id = Some(h.todo_id.clone());
            s
        }
        WorkStatus::Blocked => {
            Suggestion::new(Case::BlockedWithoutHandoff, "relay-continue")
                .backlog(ok.active_backlog_id.as_ref())
        }
        WorkStatus::Ready => match first_available(&ok.todo) {
            Some(entry) => Suggestion::new(Case::Ready, "relay-session")
                .backlog(ok.active_backlog_id.as_ref())
                .entry(&entry, true),
            None => Suggestion::new(Case::BlockedWithoutHandoff, "relay-continue")
                .backlog(ok.active_backlog_id.as_ref()),
        },
        WorkStatus::Done if todo_finished_but_item_open(ok) => {
            Suggestion::new(Case::DoneWithTodo, "relay-session").backlog(ok.active_backlog_id.as_ref())
        }
        WorkStatus::Backlog => match first_available(&ok.backlog) {
            Some(entry) => Suggestion::new(Case::BacklogWithAvailable, "relay-session").entry(&entry, false),
            None => Suggestion::new(Case::BacklogWithoutAvailable, "relay-continue"),
        },
        // `InProgress` without a handoff cannot be derived; both it and any other
        // `Done` end up where nothing is pending.
        WorkStatus::Done | WorkStatus::InProgress => Suggestion::new(Case::Done, "relay-spec"),
        WorkStatus::Idle => Suggestion::new(Case::Idle, "relay-spec"),
    }
}

/// Every TODO item is `[x]`, no handoff remains, and the backlog item the TODO
/// belongs to is still open: step 5 of the protocol (closing the item) is
/// missing.
fn todo_finished_but_item_open(ok: &OkState) -> bool {
    if ok.handoff.is_some() || ok.todo.is_empty() || ok.todo.iter().any(|e| e.marker != 'x') {
        return false;
    }
    let Some(active) = ok.active_backlog_id.as_ref() else {
        return false;
    };
    ok.backlog.iter().any(|e| &e.id == active && e.marker != 'x')
}
