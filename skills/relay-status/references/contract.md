# Relay contract for relay-status

> Copied verbatim from docs/PROTOCOL.md; the checks below are every condition that makes the state inconsistent.

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

## Backlog template

```markdown
# Backlog

- [ ] B-001 - <Independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-002 - <Another independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-003 - <Outcome that requires B-001> (spec: `.specs/20260905-001-<slug>.md`) (needs: B-001)
```

Use `[ ]` for `backlog`, `[x]` for `done`, `[!]` for a blocked entry, and `[-]`
for a dropped one. A dropped entry carries `(dropped: <reason>)` after its
spec, is never deleted, is not available, and does not satisfy `needs`. Keep outcome and acceptance
details in the source spec; each entry points to exactly one spec. Textual order
may define only the deterministic default recommendation: the first available
entry. It does not encode priority, a queue, or a dependency. A dependency is
declared with `needs`, never implied by position, and the user may select any
available entry.

The next backlog ID is one more than the highest `B-NNN` anywhere under
`.orchestration/`, archived sections and the legacy changelog included; resolve
it with `grep`, without reading whole files. No existing ID is renumbered.

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

## Changelog template

The changelog is one file per specification:
`.orchestration/changelog/<YYYYMMDD-NNN>.md`, named by the spec's date and
sequence. The file is the spec, so a record does not repeat `Spec`: the spec of
the backlog entry named in `Backlog` must be the file's own. A session reads
only the file of the active spec.

```markdown
# Change log 20260905-001

## 2026-09-05 - T-001 - <Subtask title>
- Backlog: B-001
- Result: <What changed.>
- Evidence: <Test, inspection, commit, or other verifiable result.>
- Criteria: <Criterion IDs advanced, qualified when from another spec, or none.>
- Decisions: <Decision retained for future sessions, or none.>
```

An unqualified criterion belongs to the file's spec; `YYYYMMDD-NNN/A-NNN` still
qualifies a criterion of another spec. The file is created with its first
record.

## Closing a specification

Closing is archiving. When the last pending backlog entry of a spec becomes
`[x]` or `[-]`, the writer appends to that spec's changelog a section, and only
then removes the spec's entries from `BACKLOG.md`:

```markdown
## Closed 2026-09-05
- [x] B-001 - <Independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [-] B-002 - <Another outcome> (spec: `.specs/20260905-001-<slug>.md`) (dropped: <reason>)
- Waived: A-003 - <reason>
```

The entries are copied literally. **The closing section is the authority:** a
backlog entry whose ID already appears in the closing section of its own spec
is archived, and removing it from `BACKLOG.md` is cleanup. A session interrupted
between the two writes therefore leaves a coherent state.

`Waived` exists only in the closing of a spec with at least one `[-]` entry, and
only `relay-spec` writes it. A criterion with no evidence in a spec that has a
drop must carry a waiver with a reason before the spec closes. A criterion that
stopped making sense without a drop is a change to the specification, and leaves
the spec.

Dropping a spec means dropping each of its pending entries; dropping the last
one closes it. A pending entry that needs a dropped one is a violation, so
dropping requires adjusting or dropping its dependents.

## Unique IDs

A `B-NNN` appears once across `BACKLOG.md` and the closing sections of the
changelog (apart from the closing window described in "Closing a
specification"), and a `T-NNN` appears once in `TODO.md`.

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
