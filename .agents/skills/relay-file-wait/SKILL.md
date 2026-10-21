---
name: relay-file-wait
description: Use when running the temporary Claude Code file-wait spike (B-068/B-069), not for ordinary Relay interviews.
---

# Relay File Wait Spike

This is disposable repository tooling for spec `20261003-003`, not a Relay feature.
Run it in Claude Code only. Use the supplied workspace, question, mechanism
(`foreground`, `background`, or `monitor`) and deadline; ask natively for missing inputs.
Read [MEASUREMENTS.md](MEASUREMENTS.md) before running measurements.

1. Run `rtk proxy python3 <skill-dir>/scripts/file_wait.py prepare --workspace '<workspace>' --prompt '<question>' --timeout <seconds>`.
   Show the question, returned `directory`, `id`, and deadline to the operator.
   Only the operator writes the answer, using the documented `reply` command.
2. Run `rtk proxy python3 <skill-dir>/scripts/file_wait.py wait '<directory>'` exactly once:
   `foreground`: Bash without `run_in_background`; `background`: Bash with
   `run_in_background: true`; `monitor`: Monitor with that command.
   Use the selected tool's actual schema and record its timeout and task/watch ID.
   An unavailable tool is `unsupported`; do not silently substitute a mechanism.
3. While waiting, only the Python process polls. A task ID without a terminal
   result means pending. Yield for the native notification; do not create model
   polling turns, a cron job, or a second waiter. Record automatic backgrounding.
4. Continue only on a terminal `answered` event whose `id` matches the current,
   uninterrupted question and whose `answer` is a nonempty string. Report that
   literal answer. A completion notice or successful exit alone is not an answer.
5. On `timeout` (exit 124), missing output, failed/stopped task or a tool limit,
   stop that waiter if still running, then ask the SAME question with Claude
   Code's native AskUserQuestion. Wait for an actual response; no default is a reply.
   Record fallback separately from a file-channel success.
   If AskUserQuestion is unavailable, report `unsupported` and stop; a plain-text
   question is not a native selectable fallback.
6. On user interruption, stop the task/watch, retire that question ID and await
   the user's direction. Ignore late events for retired IDs. Repeating a question
   starts a fresh directory and ID; an outcome is terminal, never reopened.

Keep all experiment artifacts in the returned temporary directories. Do not edit
Relay records, package skills, the protocol or the TUI while executing this spike.
Keep artifacts until evidence has been captured; B-070 removes this skill.
