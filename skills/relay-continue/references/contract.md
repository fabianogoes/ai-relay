# Relay contract for relay-continue

> Copied verbatim from docs/PROTOCOL.md; the checks below are every condition that makes the state inconsistent.

## Dates and times

A date or time in a record comes from the system clock at the moment of writing,
read with the command below. It never comes from memory, from the conversation or
from file-system metadata, because a value made up by the writer passes the
format check and still lies about when the work happened.

| Field | Command | Example |
| --- | --- | --- |
| Handoff `Updated` | `date -u +%Y-%m-%dT%H:%M:%SZ` | `2026-09-05T23:41:00Z` |
| Date of a changelog record and of `## Closed` | `date +%Y-%m-%d` | `2026-09-05` |
| Date in a specification filename | `date +%Y%m%d` | `20260905` |

`Updated` is written in UTC with `Z`: the `%z` of `date` prints `-0300`, without
the colon RFC 3339 requires.

## Allowed statuses

`backlog`, `ready`, `in_progress`, `blocked`, `done`, and `idle` are English
status values. `inconsistent` is a derived diagnostic and must not be written
as a work status.

`done` and `idle` are the terminal states and are derived, never written. A
non-empty backlog whose entries are all `[x]` derives `done`; backlogs written
before specifications were archived on closing still do. Once a specification
closes, its entries leave the backlog, so with everything closed the state is
`idle`. The absence of backlog, TODO, and handoff entries derives `idle`. An
empty backlog is not a status: with pending backlog entries absent, the other
records decide the state.

Specification filenames use `YYYYMMDD-NNN-<slug>.md`: the date is the creation
date and `NNN` is a three-digit sequence that restarts at `001` each day.
Existing specifications with older names remain valid and must not be renamed
solely to adopt this convention.

The state between the last TODO item and step 5 of the transition rules (every
TODO item `[x]`, no handoff, backlog entry still open) is `done` for the active
task, and the next step is transition 5. No status is added for it.

## Reading records

Readers treat `\r\n` as `\n`: a record written with CRLF line endings derives
the same state as the same record written with LF.

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
needed to resume.

The handoff metadata are the `- Key: value` lines before the first `##`; what
follows in the sections is free text and never changes the metadata, so a
`- Status:` line inside `## Context` is ignored. `Status` accepts only
`in_progress` and `blocked`; any other value is a violation, never read as
`in_progress`. The handoff is empty only when it is exactly the empty form
below: mentioning "No active handoff" anywhere else does not make a handoff
empty. An empty handoff is:

```markdown
# Handoff

No active handoff.
```

## Unique IDs

A `B-NNN` appears once across `BACKLOG.md` and the closing sections of the
changelog (apart from the closing window described in "Closing a
specification"), and a `T-NNN` appears once in `TODO.md`.

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
4. To complete a subtask, append its changelog record to the spec's changelog
   file, set its TODO marker to
   `[x]`, then clear the handoff. The record's `Criteria` names every acceptance
   criterion of the spec that the subtask advanced, or `none`. `none` is a
   claim like any other and must be true.
5. After all TODO items are `done`, mark the backlog task `done` and replace
   TODO with its empty state. When that entry was the last pending one of its
   spec, close the spec as described in "Closing a specification". Marking the **last** pending backlog entry of a
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

6. `relay-spec` may drop a pending backlog entry with `[-]` and a reason, or a
   whole spec. Subtasks in a TODO are never dropped.

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
- A handoff `Status` is neither `in_progress` nor `blocked`.
- A `B-NNN` appears more than once across the backlog and the closing sections,
  or a `T-NNN` appears more than once in the TODO.
- A changelog record names a backlog entry whose spec is not the spec of the
  file that holds the record.
- A TODO item is removed from handoff before its completed result is appended
  to the changelog.
- A backlog task is `done` while an active TODO item for it is not `done`.
- A checklist item uses an unknown marker, or a completed item is not `[x]`.
- A `needs` reference names an ID absent from the same record.
- A `needs` relation contains a cycle.
- An entry is `[x]` while an ID it needs is not `[x]`.
- A pending backlog entry needs a dropped (`[-]`) entry.
- A dropped entry has no reason.
- A `Waived` line appears in a closing section with no `[-]` entry.
- Every backlog entry of a specification is `[x]` or `[-]` while an acceptance criterion
  of that specification is named by no changelog record.
