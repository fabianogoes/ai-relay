//! The wording of the next-step line, in Portuguese, from a `Suggestion`.
//!
//! The skill is bold in `fg`, ids are `id`, the rest is `meta`: the skill reads
//! as text and bold, never only as a color.

use crate::suggest::{Case, Suggestion};
use crate::theme;

use super::cards::{Seg, seg};
use super::text;

/// The line, for `width` columns: when it is too long, the task's title gives
/// way first (cut with `…`, or left out when there is no room for even a few
/// letters of it), so the skill to call is never what is cut.
pub(super) fn segments(s: &Suggestion, width: usize) -> Vec<Seg> {
    let whole = build(s, width, false);
    let fits = |segs: &[Seg]| segs.iter().map(|(t, _)| text::width(t)).sum::<usize>() <= width;
    if fits(&whole) {
        return whole;
    }
    // Still too long with the title gone: the long wordings give way to a short
    // one, so that the skill at their end is not what is cut.
    let short = build(s, width, true);
    if fits(&short) { short } else { whole }
}

fn build(s: &Suggestion, width: usize, short: bool) -> Vec<Seg> {
    let mut b = Builder(Vec::new(), None);
    let backlog = s.backlog_id.as_deref();
    let todo = s.todo_id.as_deref();
    let title = s.title.as_deref();
    match s.case {
        Case::NotAWorkspace => {
            b.meta("Instale o protocolo com ").skill(s.skill).meta(".");
        }
        Case::Inconsistent if short => {
            b.meta("Registros em conflito: ").skill(s.skill).meta(" e ").skill("relay-continue").meta(".");
        }
        Case::Inconsistent => {
            b.meta("Os registros se contradizem: ")
                .skill(s.skill)
                .meta(" mostra o diagnóstico e ")
                .skill("relay-continue")
                .meta(" pode propor o reparo.");
        }
        Case::InProgress => {
            b.meta("Retome ").id(todo).meta(" de ").id(backlog).meta(" com ").skill(s.skill).meta(".");
        }
        Case::BlockedWithHandoff if short => {
            b.meta("Bloqueio em ").id(todo).meta(": retome com ").skill(s.skill).meta(".");
        }
        Case::BlockedWithHandoff => {
            b.meta("Resolva o bloqueio de ")
                .id(todo)
                .meta(" (ver Handoff) e retome com ")
                .skill(s.skill)
                .meta(".");
        }
        Case::BlockedWithoutHandoff if short => {
            b.meta("Sem subtarefa disponível: ").skill(s.skill).meta(".");
        }
        Case::BlockedWithoutHandoff => {
            b.meta("Nenhuma subtarefa disponível");
            if backlog.is_some() {
                b.meta(" em ").id(backlog);
            }
            b.meta(": ").skill(s.skill).meta(" mostra o bloqueio e as opções.");
        }
        Case::Ready => {
            b.meta("Comece ").id(todo);
            if let Some(title) = title {
                b.title(title);
            }
            b.meta(" com ").skill(s.skill).meta(".");
        }
        Case::DoneWithTodo if short => {
            b.id(backlog).meta(" concluído: feche com ").skill(s.skill).meta(".");
        }
        Case::DoneWithTodo => {
            b.meta("Subtarefas de ")
                .id(backlog)
                .meta(" concluídas: feche o item com ")
                .skill(s.skill)
                .meta(".");
        }
        Case::BacklogWithAvailable => {
            b.meta(if short { "Próximo item: " } else { "Próximo item disponível: " }).id(backlog);
            if let Some(title) = title {
                b.title(title);
            }
            b.meta(". Comece com ").skill(s.skill).meta(".");
        }
        Case::BacklogWithoutAvailable if short => {
            b.meta("Nenhum item disponível: ").skill(s.skill).meta(".");
        }
        Case::BacklogWithoutAvailable => {
            b.meta("Nenhum item disponível: resolva os bloqueios ou dependências do backlog; ")
                .skill(s.skill)
                .meta(" mostra as opções.");
        }
        Case::Done => {
            b.meta("Tudo concluído. Para uma nova ideia: ").skill(s.skill).meta(".");
        }
        Case::Idle => {
            b.meta("Nada em andamento. Para começar: ").skill(s.skill).meta(".");
        }
    }
    b.fit_title(width);
    b.0
}

/// The segments, and where the task's title (in parentheses) is among them.
struct Builder(Vec<Seg>, Option<usize>);

impl Builder {
    fn meta(&mut self, text: &str) -> &mut Self {
        self.0.push(seg(text, theme::color(theme::META)));
        self
    }

    fn title(&mut self, title: &str) -> &mut Self {
        self.1 = Some(self.0.len());
        self.0.push(seg(format!(" ({title})"), theme::color(theme::META)));
        self
    }

    /// Cuts the title so that the whole line fits `width`; with room for fewer
    /// than a few letters, the title is dropped.
    fn fit_title(&mut self, width: usize) {
        let Some(at) = self.1 else {
            return;
        };
        let total: usize = self.0.iter().map(|(t, _)| text::width(t)).sum();
        if total <= width {
            return;
        }
        let title_width = text::width(&self.0[at].0);
        let others = total - title_width;
        // " (" + letters + "…" + ")".
        let room = width.saturating_sub(others);
        if room < 7 {
            self.0.remove(at);
            return;
        }
        let full = self.0[at].0.clone();
        let inner = full.strip_prefix(" (").and_then(|t| t.strip_suffix(')')).unwrap_or(&full);
        let kept = text::truncate(inner, room - 3);
        self.0[at].0 = format!(" ({kept})");
    }

    fn skill(&mut self, name: &str) -> &mut Self {
        self.0.push(seg(name, theme::bold(theme::FG)));
        self
    }

    fn id(&mut self, id: Option<&str>) -> &mut Self {
        if let Some(id) = id {
            self.0.push(seg(id, theme::color(theme::ID)));
        }
        self
    }
}
