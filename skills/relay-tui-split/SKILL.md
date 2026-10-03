---
name: relay-tui-split
description: Use when the user wants the relay-tui panel opened in a terminal split next to the harness, in tmux, zellij, WezTerm, kitty, iTerm2 or Warp.
---

# Relay TUI Split

Execute this skill; do not quote it. This skill is outside the Relay protocol: it
opens a window and reads or writes none of the five records.

Run `scripts/open-split.sh` from this skill's directory, with the workspace to
observe as its only argument, the user's repository as an absolute path (the
script defaults to the current directory). Add `--dry-run` only when the user
asks what it would do: it prints the command and runs nothing.

The script detects the terminal from the environment (tmux, zellij, WezTerm,
kitty, iTerm2, Warp; the first match wins), opens a pane to the right and runs
`relay-tui --workspace '<dir>'` in it. It installs nothing and never tries a
second terminal.

Report to the user what the script printed, by exit status:

- `0`: the pane is open (or, with `--dry-run`, the command it would run).
- `1`: `relay-tui` is not on the `PATH`. Give the download link it printed; do
  not install the binary yourself.
- `2`: the terminal is not supported or the split failed. Give the manual
  instruction it printed, including the command to run in a new pane.

Do not retry with another terminal's command, open the panel any other way, or
close, focus or manage the pane afterwards.
