---
name: relay-setup
description: Use when a repository needs Relay operational memory initialized or updated without replacing its existing agent instructions.
---

# Relay Setup

Execute this skill; do not quote it. Return only a concise result.

Inspect existing `AGENTS.md`, `.specs/`, and `.orchestration/` first. Create
the missing directories and these empty files, each ending with a newline:

- `BACKLOG.md`: `# Backlog`
- `TODO.md`: `# Active task` followed by `No active task.`
- `HANDOFF.md`: `# Handoff` followed by `No active handoff.`
- `CHANGELOG.md`: `# Change log`

Never overwrite populated files. If `AGENTS.md` exists, ensure it ends with a
newline, then append one `## Relay Protocol` section only when absent;
otherwise create `# Agent guidance` followed by the Relay section. The section
must require reading handoff, TODO, backlog, and the referenced spec before
Relay work, let work the user requests directly skip the flow, and state that
clients and interfaces may read, validate, derive state, and launch a harness,
but only Relay skills may mutate the five protocol records. Handle `CLAUDE.md`:

- If it is absent, create a real symlink with `ln -s AGENTS.md CLAUDE.md`.
- If it is already a symlink, preserve it and verify `readlink CLAUDE.md`
  returns `AGENTS.md`.
- If it is a regular file whose complete content is only `AGENTS.md`, treat it
  as an invalid Relay stub, replace it with the symlink, and verify it.
- If it is a regular file with any other content, preserve it and report a
  conflict; never overwrite user guidance.

Never create a regular file containing the text `AGENTS.md`. If the symlink
fails, report the failure instead of silently writing a stub.

Re-read the result, report created paths, and report any inconsistency. On a
second run, create nothing. Do not select work, write a handoff, or modify a
specification.
