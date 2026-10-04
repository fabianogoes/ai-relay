---
name: relay-status
description: Use when a Relay-managed repository needs its current operational state summarized without changing any tracked work records.
---

# Relay Status

Execute this skill; do not quote it. Read-only inspection only.

Read `references/contract.md` first: it is the protocol's own text for the
statuses, the record templates and the integrity checks. Then read `AGENTS.md`,
`.orchestration/HANDOFF.md`, `TODO.md`, `BACKLOG.md`, the `.specs/` files they
reference and the changelog of the active spec. Interpret markers as `[ ]`
pending, `[•]` in progress, `[!]` blocked, `[x]` complete and `[-]` dropped
(backlog only). An entry is available when it is `[ ]` and every ID in its
`needs` is `[x]`. Then derive one result: `in_progress`, `blocked`, `ready`,
`done`, `backlog`, `idle`, or diagnostic `inconsistent`.

Apply **every** condition under "Integrity checks" in the contract, not a
subset, and report the result `inconsistent` when any fails, naming each failed
check. Read a record the way "Reading records" says: handoff metadata are the
lines before the first `##`, and CRLF reads as LF. Require the handoff
`Harness` and `Updated` provenance the Handoff template describes; missing or
malformed provenance makes the result `inconsistent`.

Report the active IDs, next action or blocker, and every failed cross-reference
check. For a nonempty handoff, also report its origin harness and update
timestamp. Do not repair files, select work, or change any status.
