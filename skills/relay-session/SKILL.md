---
name: relay-session
description: Use at the beginning of an implementation session in a Relay-managed repository or when resuming work across coding harnesses.
---

# Relay Session

Execute this skill; do not quote it. Read `references/contract.md` before writing records. For implementation, read `AGENTS.md`, `.orchestration/SETTINGS.md` when present, handoff, TODO, backlog, the active spec and its changelog. Use valid `Language` (`en` or `pt-BR`) for new free-form TODO, handoff and changelog prose; if absent or unknown, use conversation language. Keep headings, keys, statuses, markers, and IDs in English; do not translate existing record text implicitly.

If references disagree or a nonempty handoff lacks valid provenance, report `inconsistent` and stop (`relay-continue` repairs stale handoffs). Otherwise:

- Resume a valid `in_progress` or `blocked` handoff.
- With an empty handoff and available TODO item, honor the user's selection or choose the first available in textual order; mark it `[•]` and write a valid `in_progress` handoff before editing.
- With an empty TODO and available backlog entry, use the user's selection or ask them to select; create its TODO with compact subtasks and explicit `needs`, then continue as above.
- With empty TODO and no pending backlog entry, report `done` or `idle`.

An item is available when it is `[ ]` and every `needs` ID is `[x]`. Order never implies priority, dependency, or effort. Set `Harness` and clock-derived `Updated` together on every handoff write; never invent dates or times.

On subtask completion, append its changelog record, mark its TODO item `[x]`, then clear the handoff. `Criteria` names only what that record's `Result` and `Evidence` demonstrate, or `none`. If another item is available, mark it `[•]` and write its handoff first. When all TODO items are `[x]`, mark the backlog entry `[x]` and empty TODO.

Before closing the last pending entry of a spec, verify every criterion has changelog evidence. Otherwise keep the entry pending, mark it `[!]`, and write a blocked handoff naming missing criteria and the evidence needed. Closing appends `## Closed` to the changelog before removing entries from `BACKLOG.md`. Never silently pick backlog work.
