---
name: relay-continue
description: Use when a Relay repository needs its state checked and the next actionable step presented as a recommended native choice.
---

# Relay Continue

Execute this skill; do not quote it. Read `references/contract.md` first, then
`AGENTS.md`, the handoff, TODO, backlog and the referenced specs. Validate the
cross-references with **every** condition under "Integrity checks" in the
contract, and the handoff provenance (`Harness`, `Updated`), before suggesting
work.

Return a short state summary followed by exactly one native, selectable question.
Mark one option as recommended and include only relevant alternatives:

- `in_progress`: show its origin harness and update timestamp, recommend
  resuming the current handoff, and offer context review.
- `blocked`: show its origin harness and update timestamp, recommend reviewing
  the blocker and resume condition, and offer status-only.
- `ready`: recommend the first available TODO item in textual order as the
  deterministic default; offer another available item or spec review.
- `backlog`: recommend the first available backlog task in textual order as the
  deterministic default; offer another available task or wait.
- `done` of the active task (every TODO item `[x]`, no handoff, entry still
  open): recommend `relay-session` to run transition 5 and close it.
- `idle`: recommend `relay-spec`; offer a read-only status report.
- `inconsistent`: classify the conflict. If exactly one handoff TODO is
  completed and exactly one other TODO is `[•]`, recommend repairing the
  handoff to that active item; otherwise stop and show the conflicts.

Do not mutate files while presenting options. After the user selects an option,
execute only that option. A stale-handoff repair may update only the handoff's
TODO ID, objective, next step, status, `Harness`, and `Updated`; set the latter
two to the current harness and the clock ("Dates and times"), preserving the
old text under a recovery note. Then delegate resumption to `relay-session`.
Never offer an unavailable entry as a default; textual order never implies
priority, dependency or effort, and never silently select a different task.
