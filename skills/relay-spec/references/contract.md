# Relay contract for relay-spec

> Copied verbatim from docs/PROTOCOL.md; read before writing a spec, a backlog entry, a drop or a closing.

## Workspace language

An optional `.orchestration/SETTINGS.md` stores the workspace's default
language:

```markdown
# Settings

- Language: en
```

`Language` accepts `en` or `pt-BR`. This is workspace configuration, not one of
the five Relay records, and it is excluded from integrity checks. Only
`relay-setup` writes it. A valid value controls free-form interview text and
specification and session prose. If the file is absent or the value is unknown,
skills use the language of the conversation. An existing value changes only
when the user explicitly asks `relay-setup` to change it. Section headings,
keys, statuses, markers, and IDs stay in English.
This rule applies to newly written prose; existing record text is not translated
implicitly.

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

## Unique IDs

A `B-NNN` appears once across `BACKLOG.md` and the closing sections of the
changelog (apart from the closing window described in "Closing a
specification"), and a `T-NNN` appears once in `TODO.md`.

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
