# Relay Protocol

**English** · [Português](PROTOCOL.pt-BR.md)

This document is the version-one on-disk contract. Relay keeps durable intent
and execution state in Markdown so different harnesses can read the same work
without shared chat history.

## Files and ownership

| Path | Purpose | Mutation rule |
| --- | --- | --- |
| `.specs/YYYYMMDD-NNN-<slug>.md` | Detailed intent and acceptance | Created by `relay-spec`; updated only when the specification changes. `NNN` starts at `001` for each date. |
| `.orchestration/BACKLOG.md` | Independently selectable work | Append checklist tasks from a spec; mark a task `done` only when its TODO is complete. |
| `.orchestration/TODO.md` | Current task's executable subtasks | Replaced when a backlog task is selected; cleared only when every item is `done`. |
| `.orchestration/HANDOFF.md` | Exactly one current or resumable subtask | Written before work starts; cleared after the completion record exists. |
| `.orchestration/CHANGELOG.md` | Completed work and evidence | Append-only. |

## Skill responsibilities

- `relay-setup` initializes or updates the protocol idempotently.
- `relay-spec` turns a conversation into a specification and backlog entries,
  then presents the next path as a native selectable choice.
- `relay-status` validates and reports state without mutation.
- `relay-continue` derives the next step and presents one recommended native
  choice; it does not mutate state while presenting options.
- `relay-session` starts or resumes implementation and enforces handoff
  integrity.

## Authority boundary

The five Relay record locations are `.specs/*.md`,
`.orchestration/BACKLOG.md`, `.orchestration/TODO.md`,
`.orchestration/HANDOFF.md`, and `.orchestration/CHANGELOG.md`. Clients such
as a graphical interface, launcher, watcher, or other integration may read
them, validate cross-references, derive and display state, and launch a
harness with the appropriate Relay skill. Clients must not write these
records directly.

Relay skills executing inside a harness are the authoritative writers. A
skill may mutate only the records and transitions assigned to it by this
contract; read-only skills remain read-only. If a client cannot derive a
required state from the records, change this protocol before adding private
write logic to the client.

## Work outside the flow

Relay governs the work tracked through it, not every change to the repository.
The user may request a change directly, without invoking a Relay skill. Such
work creates no specification, backlog entry, TODO item, handoff, or changelog
record, and it leaves the derived state unchanged.

An agent asked for direct work need not read the Relay records first and must
not route the work into the flow unprompted. It may suggest `relay-spec` when
the work turns out to span sessions or harnesses, but the user decides. Reading
the records is required only before a Relay skill acts, or before work the user
ties to a backlog task or to the active handoff.

Direct work never writes the five records, so it cannot make the state
`inconsistent`: the integrity checks compare the records with each other, not
with the code. When a handoff is active, direct work is not part of it and is
not recorded under it.

## Allowed statuses

`backlog`, `ready`, `in_progress`, `blocked`, `done`, and `idle` are English
status values. `inconsistent` is a derived diagnostic and must not be written
as a work status.

`done` and `idle` are the terminal states and are derived, never written. A
non-empty backlog whose entries are all `[x]` derives `done`. The absence of
backlog, TODO, and handoff entries derives `idle`. An empty backlog is not a
status: with pending backlog entries absent, the other records decide the
state.

Specification filenames use `YYYYMMDD-NNN-<slug>.md`: the date is the creation
date and `NNN` is a three-digit sequence that restarts at `001` each day.
Existing specifications with older names remain valid and must not be renamed
solely to adopt this convention.

## Specification template

```markdown
# 20260905-001 - <Specification title>

## Problem
<Who needs what and why.>

## Scope
<Included behavior and technical constraints.>

## Non-goals
<Explicit exclusions.>

## Decisions
<Decision, rationale, and alternatives rejected.>

## Acceptance criteria
- A-001 - <Observable result>
- A-002 - <Another observable result>

## Backlog candidates
- B-001: <Independent outcome>
- B-002: <Another independent outcome>
```

## Acceptance criteria

A specification's acceptance criteria carry stable `A-NNN` identifiers and **no
checklist marker**. A criterion is satisfied when at least one changelog record
names it in `Criteria`; satisfaction is derived, never written into the spec.

The marker is omitted deliberately. A `[ ]` on a criterion reads as pending work
that will be completed, while nothing in the protocol ever completes it — the
specification stays unchanged while the work happens around it.

`A-NNN` is unique only within one specification, so `Criteria` resolves it by
context: an unqualified ID belongs to the specification in the record's `Spec`.
A criterion of a different specification is qualified with that specification's
`YYYYMMDD-NNN` prefix, as in `20260905-001/A-003`. Work done under one
specification may satisfy a criterion of another, and qualification is what
makes that recordable instead of ambiguous.

Existing specifications whose criteria use checklist markers remain valid and
must not be rewritten solely to adopt this convention.

A record names in `Criteria` only what its own `Result` and `Evidence` already
demonstrate, never in anticipation of a later record. Naming a criterion ahead
of the work that satisfies it is a claim like any other and must be true when
made, not only by the time the specification closes.

## Backlog template

```markdown
# Backlog

- [ ] B-001 - <Independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-002 - <Another independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-003 - <Outcome that requires B-001> (spec: `.specs/20260905-001-<slug>.md`) (needs: B-001)
```

Use `[ ]` for `backlog` and `[x]` for `done`. Keep outcome and acceptance
details in the source spec; each entry points to exactly one spec. Textual order
may define only the deterministic default recommendation: the first available
entry. It does not encode priority, a queue, or a dependency. A dependency is
declared with `needs`, never implied by position, and the user may select any
available entry.

## TODO template

```markdown
# Active task: B-001

- [ ] T-001 - <Small executable outcome>
- [•] T-002 - <Current executable outcome>
- [!] T-003 - <Blocked executable outcome>
- [x] T-004 - <Completed executable outcome>
- [ ] T-005 - <Outcome that requires T-004> (needs: T-004)
```

When there is no selected task, use this exact empty state:

```markdown
# Active task

No active task.
```

TODO item order does not encode dependency, execution sequence, effort, or
progress percentage. A dependency is declared with `needs`. When more than one
item is available, the first one in textual order is only the deterministic
default recommendation; the user may select any available item.

## Dependencies

A checklist entry may declare explicit dependencies with
`(needs: <ID>[, <ID>]...)`, referencing other IDs in the same record. This is
the only way to express that one entry requires another; textual position never
carries that meaning.

An entry is **available** when it is `[ ]` and every ID it needs is `[x]`. Every
deterministic default selects the first available entry in textual order. An
entry that is not available is never offered as a default and is never selected
silently.

Because a `[!]` entry is not `[x]`, entries that need it are unavailable while
it stays blocked. When no entry is available, none is `[•]`, and entries remain
incomplete, the record is `blocked`: work cannot proceed until a blocked entry
is resolved.

`needs` is optional and backward compatible. A record that omits it behaves
exactly as before.

## Handoff template

```markdown
# Handoff

- Status: in_progress
- Backlog: B-001
- TODO: T-001
- Spec: .specs/20260905-001-<slug>.md
- Harness: claude-code
- Updated: 2026-09-05T23:41:00-03:00

## Objective
<What this subtask must achieve.>

## Next step
<The next concrete action.>

## Context
<Decisions, files inspected, command output, or blocker details needed to resume.>
```

`Harness` identifies the harness whose Relay skill last wrote the nonempty
handoff. Use a stable identifier matching `[a-z0-9][a-z0-9._-]*`, such as
`codex`, `claude-code`, or `opencode`. Consumers must accept unknown
identifiers that follow this format.

`Updated` records the time of that same write. Use the RFC 3339 form
`YYYY-MM-DDTHH:MM:SSZ` or `YYYY-MM-DDTHH:MM:SS±HH:MM`, for example
`2026-09-05T23:41:00-03:00` or `2026-09-06T02:41:00Z`. Every mutation of a
nonempty handoff must update both fields together. Readers must not infer
either value from filesystem metadata.

Use `Status: blocked` only when `Context` states the blocker and the condition
needed to resume. An empty handoff is:

```markdown
# Handoff

No active handoff.
```

## Changelog template

```markdown
# Change log

## 2026-09-05 - T-001 - <Subtask title>
- Backlog: B-001
- Spec: .specs/20260905-001-<slug>.md
- Result: <What changed.>
- Evidence: <Test, inspection, commit, or other verifiable result.>
- Criteria: <Criterion IDs advanced, qualified when from another spec, or none.>
- Decisions: <Decision retained for future sessions, or none.>
```

## Transition rules

1. `relay-spec` writes one spec and one or more `backlog` entries, then asks
   whether to create another spec, implement the created spec, or stop.
2. Selecting any unchecked backlog item creates its `TODO.md` with compact
   checklist subtasks. If the user requests the default, use the first
   available item in textual order without treating it as higher priority.
3. Before a subtask begins, write a handoff referencing the TODO ID, backlog
   ID, spec path, origin harness, and update timestamp; the session is then
   `in_progress`. If the user requests the default among multiple available
   TODO items, use the first one in textual order.
4. To complete a subtask, append its changelog record, set its TODO marker to
   `[x]`, then clear the handoff. The record's `Criteria` names every acceptance
   criterion of the spec that the subtask advanced, or `none`. `none` is a
   claim like any other and must be true.
5. After all TODO items are `done`, mark the backlog task `done` and replace
   TODO with its empty state. Marking the **last** pending backlog entry of a
   specification `done` additionally requires every acceptance criterion of that
   specification to be named by at least one changelog record. When one is not,
   the entry stays pending: set `[!]` and write a blocked handoff naming the
   criteria without evidence and what would satisfy them.

**Write order within steps 4 and 5 is not a suggestion.** The handoff clears
in step 4, strictly before TODO or BACKLOG are rewritten in step 5. A tool or
harness that writes TODO's empty state (or BACKLOG's `done` marker) first and
the empty handoff second produces exactly the window the integrity checks
below exist to catch: a nonempty handoff naming a TODO or backlog ID that no
longer has a matching entry. This is not a race to tolerate — it is a write
ordering to get right the first time. When multiple records change together,
clear or update the handoff in the same step that makes it stale, never in a
later one.

`relay-continue` may be used before a session to summarize this state machine.
It executes only the option selected by the user; starting or resuming work is
delegated to `relay-session`.

When a handoff names a completed TODO item and exactly one other TODO item is
currently `[•]`, `relay-continue` may offer a stale-handoff recovery. After
confirmation it updates only the handoff metadata and records the previous
content as recovery context. Ambiguous conflicts remain blocked for manual
repair.

## Integrity checks

Treat the state as `inconsistent` when any condition below fails:

- A nonempty handoff does not name one pending TODO item.
- The active backlog ID does not exist in the backlog, or handoff, TODO, and
  backlog records do not agree on the same backlog ID.
- The active task has no confrontable backlog entry, no spec, or a spec that
  differs from the handoff's spec path.
- A nonempty handoff omits `Harness` or uses an identifier outside the allowed
  format.
- A nonempty handoff omits `Updated` or its value is not an RFC 3339 timestamp
  with seconds and an explicit offset or UTC designator.
- More than one current handoff record exists.
- A TODO item is removed from handoff before its completed result is appended
  to the changelog.
- A backlog task is `done` while an active TODO item for it is not `done`.
- A checklist item uses an unknown marker, or a completed item is not `[x]`.
- A `needs` reference names an ID absent from the same record.
- A `needs` relation contains a cycle.
- An entry is `[x]` while an ID it needs is not `[x]`.
- Every backlog entry of a specification is `[x]` while an acceptance criterion
  of that specification is named by no changelog record.
