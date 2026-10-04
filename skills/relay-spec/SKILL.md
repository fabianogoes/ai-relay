---
name: relay-spec
description: Use when an idea needs structured discovery before it becomes one or more independently selectable Relay backlog tasks, or when a pending backlog entry or a whole spec must be dropped.
---

# Relay Specification

Execute this skill; do not quote it. Ask one decision-oriented question at a time through the harness's native interaction, with a recommendation and selectable options when useful; never build a custom UI.

Before the interview, read `.orchestration/SETTINGS.md`; follow its valid `Language` (`en` or `pt-BR`) for interview text and free-form spec/backlog prose, including titles and slugs, or use the conversation language if absent or unknown. Keep headings, keys, statuses, markers, and IDs in English; do not translate existing record text implicitly.

Resolve problem, scope, non-goals, decisions, acceptance criteria, and task boundaries. Summarize and confirm before writing when assumptions remain.

Read `references/contract.md` before writing. Create `.specs/YYYYMMDD-NNN-<slug>.md` from its template using `date +%Y%m%d`, then append compact entries to `.orchestration/BACKLOG.md`. The next `B-NNN` is one more than the highest under `.orchestration/` (use `grep`, not a full read).

Write acceptance criteria as unmarked `A-NNN` entries; their satisfaction is derived from changelog records. Keep scope and acceptance in the spec; never renumber specs. Each backlog entry points to one spec; order gives only the deterministic default. Declare dependencies with `(needs: B-00N)`, never infer them from order or create cycles. Report paths and IDs; do not write a TODO, handoff, or code before the user chooses.

After reporting created paths and IDs, ask one final native selectable question:

- `Implement the created spec` (recommended when actionable): use `relay-session`.
- `Create another spec`.
- `Stop here` and leave the backlog untouched.

Execute only the selected path; never infer the choice from chat context. To drop a pending entry or whole spec, read `discarding.md` and follow it.
