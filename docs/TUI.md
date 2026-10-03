# relay-tui: the terminal panel

**English** · [Português](TUI.pt-BR.md)

`relay-tui` is a read-only panel meant to sit in a terminal split: the harness
on one side, the panel on the other. Handoff, TODO and backlog change on screen
as the files change, with no reload. It never writes the records: the skills
do, inside a harness.

It is a single binary: no Node, and no clone of the repository.

## Install

Versions are on [Releases](https://github.com/fabianogoes/ai-relay/releases),
one file per target:

| Target | File | Status |
| --- | --- | --- |
| macOS, Apple Silicon | `relay-tui-<version>-aarch64-apple-darwin.tar.gz` | required |
| macOS, Intel | `relay-tui-<version>-x86_64-apple-darwin.tar.gz` | required |
| Linux x64 and arm64 | `…-x86_64-unknown-linux-musl.tar.gz`, `…-aarch64-unknown-linux-musl.tar.gz` | best effort |
| Windows | — | out for now |

On macOS, `uname -m` tells which one is yours (`arm64` or `x86_64`). Downloading
with `curl`, macOS does not block the file:

```sh
curl -fLO https://github.com/fabianogoes/ai-relay/releases/download/relay-tui-v<version>/relay-tui-<version>-aarch64-apple-darwin.tar.gz
tar -xzf relay-tui-<version>-aarch64-apple-darwin.tar.gz
mkdir -p ~/.local/bin
mv relay-tui-<version>-aarch64-apple-darwin/relay-tui ~/.local/bin/
```

Make sure `~/.local/bin` is on your `PATH`, and check with `relay-tui --version`.

**Downloaded with the browser?** The binary is not signed, and macOS refuses to
open a file downloaded that way ("cannot be opened"). Clear the quarantine with:

```sh
xattr -d com.apple.quarantine ~/.local/bin/relay-tui
```

Every file has a `.sha256` next to it: `shasum -a 256 -c <file>.sha256`.

## Open it in a split

Open a pane next to the harness, go to the Relay repository and run
`relay-tui`:

| Terminal | How to open the pane next to it |
| --- | --- |
| **Warp** | `Cmd+D` splits to the right (`Cmd+Shift+D`, downwards). |
| **iTerm2** | `Cmd+D` splits to the right (`Cmd+Shift+D`, downwards). |
| **tmux** | `tmux split-window -h relay-tui`, or `Ctrl-b %` and then `relay-tui`. |
| **Terminal.app** | It has no split. Use Warp, iTerm2 or tmux (`brew install tmux`). |

```sh
cd ~/Developer/my-repository
relay-tui
```

Leave the harness in the other pane and work as usual: the panel follows along.

## Use it

```sh
relay-tui                              # watches the current directory
relay-tui --workspace ~/Developer/repo # watches another repository
relay-tui --version
relay-tui --help
```

Keys: `q`, `Esc` or `Ctrl-C` quit. There is nothing else to press: it is a panel
to look at.

### What the screen shows

- **Handoff**: the status (`Em andamento`, `Bloqueado`), who left it and when,
  the objective and the next step. When blocked, it also shows the blocker and
  the condition to resume.
- **TODO**: a bar with one segment per subtask and the list, with `✓` done, `●`
  in progress, `○` available, `◌` waiting on another subtask and `!` blocked.
- **Backlog**: how many items are done, in progress, available or waiting.
- **`● atualizando` / `● atualizado`**, in the corner: the panel waits for the
  files to stop changing (150 ms) and reads everything again.
- **`Inconsistente`**, in red: the records contradict each other, and the panel
  lists each violation. It is the same state the `relay-status` skill reports.

The color follows the status (green in progress or done, blue ready or
available, yellow blocked, red inconsistent), but every state is also spelled
out in words, never by color alone. The screen text is in Portuguese.

When there is not enough height, the TODO is cut at `+N itens`. Below 40
columns only the header and the status are shown. A directory without
`.orchestration/` shows "Não é um workspace Relay" and switches to the state as
soon as Relay is installed there (`relay-setup`).

The background is not painted: it comes from the terminal. The colors assume a
dark terminal.

## Known limits

- Record lines ending in CRLF (common on Windows with `autocrlf`) are ignored.
- A `SIGTERM` from outside the keyboard does not restore the terminal; `q`,
  `Esc`, `Ctrl-C` and an internal error do.
- Dark theme only.

## From source

Without downloading anything, with Rust installed (`rustup`):

```sh
cd app/relay-tui
cargo build --release
./target/release/relay-tui --workspace /path/to/repo
```
