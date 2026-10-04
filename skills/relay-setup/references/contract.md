# Relay contract for relay-setup

> Copied verbatim from docs/PROTOCOL.md; read before creating or migrating a record.

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

## Files and ownership

| Path | Purpose | Mutation rule |
| --- | --- | --- |
| `.specs/YYYYMMDD-NNN-<slug>.md` | Detailed intent and acceptance | Created by `relay-spec`; updated only when the specification changes. `NNN` starts at `001` for each date. |
| `.orchestration/BACKLOG.md` | Independently selectable open work | Append checklist tasks from a spec; mark a task `done` only when its TODO is complete; drop a task with `[-]`; remove a spec's entries when it closes (see "Closing a specification"). |
| `.orchestration/TODO.md` | Current task's executable subtasks | Replaced when a backlog task is selected; cleared only when every item is `done`. |
| `.orchestration/HANDOFF.md` | Exactly one current or resumable subtask | Written before work starts; cleared after the completion record exists. |
| `.orchestration/changelog/<YYYYMMDD-NNN>.md` | Completed work and evidence of one spec | Append-only, one file per spec. A session reads only the file of the active spec. |
| `.orchestration/CHANGELOG.md` | Legacy single changelog | Read-only: valid to read, never receives a new record. Only `relay-setup` moves records out of it (see "Legacy changelog"). |

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

## Legacy changelog

An existing `.orchestration/CHANGELOG.md` stays valid to read and never
receives a new record. After confirmation, `relay-setup` splits it by `Spec`
into per-spec files, moving each record with its text unchanged. Records with no
`Spec`, or with a spec that does not exist, stay in the legacy file and are
reported. This is the only movement of a record the protocol allows, and a
second run changes nothing.
