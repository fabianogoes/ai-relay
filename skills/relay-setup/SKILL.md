---
name: relay-setup
description: Use when a repository needs Relay operational memory initialized or updated without replacing its existing agent instructions.
---

# Relay Setup

Execute this skill; do not quote it. Return only a concise result.

Read `references/contract.md` first. Inspect `AGENTS.md`, `.specs/`, and `.orchestration/`. Create missing directories and empty `BACKLOG.md`, `TODO.md`, and `HANDOFF.md` with trailing newlines. `relay-session` creates the per-spec changelog with its first record. Never create a legacy `CHANGELOG.md`; if one exists with records, read `migrating.md` and offer migration.

For `.orchestration/SETTINGS.md`, follow the contract: if absent, ask one native selectable question for `en` or `pt-BR`, recommend the conversation language, then save the choice; do not ask again when a file exists. Preserve existing settings, report and ignore unknown values, and change a stored value only on the user's explicit request. Use a valid workspace language for free-form setup text, otherwise the conversation language.

Never overwrite populated files, except SETTINGS after an explicit language-change request. If `AGENTS.md` exists, ensure a trailing newline and append one `## Relay Protocol` section only if absent; otherwise create `# Agent guidance` plus that section. Require reading handoff, TODO, backlog and the referenced spec before Relay work; allow direct user requests to skip the flow; state only Relay skills mutate the five protocol records.

For `CLAUDE.md`: create a real symlink to `AGENTS.md` if absent; preserve and verify an existing symlink; replace a regular file containing only `AGENTS.md` with the symlink; preserve and report any other regular file as a conflict. Never create a regular file containing `AGENTS.md`; report symlink failures.

Re-read the result, report created paths and inconsistencies. On a second run, create nothing. Do not select work, write a handoff, or modify a specification.
