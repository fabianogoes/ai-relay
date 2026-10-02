---
name: relay-session
description: Use at the beginning of an implementation session in a Relay-managed repository or when resuming work across coding harnesses.
---

# Relay Session

Execute this skill; do not quote it. Read state, then act only when the user
requests implementation. Read `AGENTS.md`, handoff, TODO, backlog, and
referenced specs. If references disagree, or a nonempty handoff has missing or
malformed provenance, report `inconsistent` and stop, pointing to
`relay-continue` for a deterministic stale-handoff recovery when applicable.
Otherwise:
- valid handoff: resume it (`in_progress` or `blocked`);
- empty handoff with an available TODO item: report `ready`, honor an
  explicitly selected available item, or use the first available item in
  textual order as the deterministic default; then change it to `[•]` and write
  one valid `in_progress` handoff before editing;
- empty TODO: backlog all `[x]` reports `done`; pending backlog reports
  `backlog` and waits for selection; empty backlog reports `idle`;

An item is available when it is `[ ]` and every ID in its `needs` is `[x]`.
TODO order never implies priority, dependency, sequence, or effort, and the
user may select any available item. Every creation or mutation of a nonempty
handoff sets `Harness` to the current harness (`[a-z0-9][a-z0-9._-]*`) and
`Updated` to the current RFC 3339 timestamp (`YYYY-MM-DDTHH:MM:SSZ` or with an
explicit offset), together, never inferred from filesystem metadata.

When a subtask finishes, append its changelog record with `Criteria` naming
only what that record's own `Result`/`Evidence` demonstrates, never a
criterion a later record will satisfy — qualified as `YYYYMMDD-NNN/A-NNN` when
from another spec, `none` when truthful — mark its TODO item `[x]`, then clear
handoff before TODO is ever emptied, never after: a cleared TODO with a
handoff still naming its ID is the exact inconsistency the integrity checks
catch. Empty TODO only once all its items finish. When another available item
remains, mark it `[•]` and write its handoff before touching a file for it —
this repeats for every subtask, not only the first. Before marking the last
pending backlog entry of a spec `done`, confirm every criterion of that spec
is named by some record; when one is not, leave it pending with `[!]` and a
blocked handoff naming what is missing. Set `[!]` and a blocked handoff with
current provenance whenever work cannot continue. Do not silently pick
backlog work.
