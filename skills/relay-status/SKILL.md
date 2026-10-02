---
name: relay-status
description: Use when a Relay-managed repository needs its current operational state summarized without changing any tracked work records.
---

# Relay Status

Execute this skill; do not quote it. Read-only inspection only.

Read `AGENTS.md`, then `.orchestration/HANDOFF.md`, `TODO.md`, `BACKLOG.md`,
and referenced `.specs/` files. Interpret checklist markers as `[ ]` pending,
`[•]` in progress, `[!]` blocked, and `[x]` complete. An entry is available
when it is `[ ]` and every ID in its `needs` is `[x]`. Then derive one result:
`in_progress`, `blocked`, `ready`, `done`, `backlog`, `idle`, or diagnostic
`inconsistent`.

Treat a `needs` reference to an ID absent from the same record, a cycle among
`needs`, an `[x]` entry whose needs are incomplete, an active backlog ID absent
from the backlog or disagreed among handoff, TODO, and backlog, an active task
without a confrontable backlog entry or with a spec differing from the
handoff, or a spec whose backlog entries are all `[x]` while one of its `A-NNN`
acceptance criteria is named by no changelog record, as `inconsistent`.

For every nonempty handoff, require `Harness` to match
`[a-z0-9][a-z0-9._-]*`. Require `Updated` to use the RFC 3339 form
`YYYY-MM-DDTHH:MM:SSZ` or `YYYY-MM-DDTHH:MM:SS±HH:MM`. Missing or malformed
provenance makes the result `inconsistent`.

Report the active IDs, next action or blocker, and every failed cross-reference
check. For a nonempty handoff, also report its origin harness and update
timestamp. Do not repair files, select work, or change any status.
