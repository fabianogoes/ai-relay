# relay-tui: the terminal panel

**English** · [Português](TUI.pt-BR.md)

`relay-tui` is a read-only panel meant to sit in a terminal split: the harness
on one side, the panel on the other. Handoff, TODO and backlog change on screen
as the files change, with no reload. It never writes the records: the skills
do, inside a harness.

It is a single binary, and needs no clone of the repository.

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

## Update

Before setting up another project, check `relay-tui --version` and compare it
with the latest version on [Releases](https://github.com/fabianogoes/ai-relay/releases).
Keep the installed binary when it is already current. If it is missing or older,
download the latest archive for your platform, verify its checksum, and replace
the installed binary:

```sh
# Select the latest release and your platform.
latest_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' https://github.com/fabianogoes/ai-relay/releases/latest)"
version="${latest_url##*/relay-tui-v}"
target=aarch64-apple-darwin
archive="relay-tui-${version}-${target}.tar.gz"
release="https://github.com/fabianogoes/ai-relay/releases/download/relay-tui-v${version}"
curl -fLO "${release}/${archive}"
curl -fLO "${release}/${archive}.sha256"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c "${archive}.sha256"
else
  sha256sum -c "${archive}.sha256"
fi
tar -xzf "${archive}"
mkdir -p "$HOME/.local/bin"
install -m 755 "relay-tui-${version}-${target}/relay-tui" "$HOME/.local/bin/relay-tui"
relay-tui --version
```

Use `aarch64-apple-darwin` for Apple Silicon, `x86_64-apple-darwin` for Intel
macOS, or the matching Linux target from the table above. If `relay-tui` points
to a source checkout instead of a downloaded binary, update that checkout and
rebuild it with `cargo build --release --locked` rather than replacing the link.

## Open it in a split

Ask the agent to do it. In Claude Code the `relay-tui-split` skill is the command
`/relay-tui-split`; in Codex and OpenCode, ask `Use relay-tui-split`. It finds out
which terminal the harness is running in and opens a pane to the right with
`relay-tui --workspace '<your repository>'`. It is a package skill outside the
Relay protocol: it touches no record, installs nothing and never tries a
terminal other than the one it detected.

| Terminal | Detected by | What it runs |
| --- | --- | --- |
| **tmux** | `TMUX` | `tmux split-window -h` |
| **zellij** | `ZELLIJ` | `zellij action new-pane --direction right --` |
| **WezTerm** | `WEZTERM_PANE` | `wezterm cli split-pane --right` |
| **kitty** | `KITTY_WINDOW_ID` | `kitten @ launch --location=vsplit` (needs `allow_remote_control yes` in `kitty.conf`) |
| **iTerm2** | `TERM_PROGRAM=iTerm.app` | AppleScript: splits the session that ran it vertically and types the command |
| **Warp** | `TERM_PROGRAM=WarpTerminal` | AppleScript through System Events: `Cmd+D`, then types the command |

The first match wins, and multiplexers come before emulators because they run
inside them: tmux inside iTerm2 opens a tmux pane. Anything else (Terminal.app, an
editor's built-in terminal) gets the manual instruction below.

Harness sandboxes can get in the way, because the script talks to the terminal
(the tmux socket, Apple Events). Verified from inside tmux: Claude Code and
OpenCode open the pane. **Codex's default sandbox blocks the tmux socket**
(`error connecting to ... (Operation not permitted)`): the script then prints the
manual instruction and exits with `2`, and the pane opens only if you run the
script outside the sandbox (for example `codex exec -s danger-full-access`, or
approving the command when Codex asks). The terminal's environment variables do
reach the harness shell in every case checked.

On macOS the first run may ask for permission, and denying it makes the skill
fall back to the manual instruction:

- **iTerm2**: allow your terminal to control iTerm2 under *System Settings >
  Privacy & Security > Automation*.
- **Warp**: it has no split command, so the skill sends `Cmd+D` through System
  Events, which needs *Privacy & Security > Accessibility*. The keystrokes go to
  whatever app is in front, so the skill first checks that Warp is, and stops
  otherwise instead of typing into another application.

Outside the harness, `sh skills/relay-tui-split/scripts/open-split.sh [dir]` does
the same, and `--dry-run` prints what it would run without running it. Exit
status: `0` opened (or `--dry-run`), `1` `relay-tui` is not on the `PATH` (it
prints the link to this guide and installs nothing), `2` terminal not recognized
or the split failed (it prints what to do by hand).

Doing it by hand: open a pane next to the harness, go to the Relay repository and
run `relay-tui`:

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

The panel opens on **Agora**, what is in progress. `Tab` (or `t`) opens the
second view, **Histórico**, and goes back; `r` reads the workspace again as a
whole (only needed when the watcher delivers no events, on a network volume for
example). `q` and `Ctrl-C` quit from any view, at once. `Esc` in Agora does not quit by
itself: it asks `Sair?` in the footer, and `Esc`, `Enter` or `y` confirm while any
other key cancels. In Histórico `Esc` goes back. Switching views, moving the selection and reloading write nothing: the panel
stays read-only.

Press `c` in Agora or Histórico to open **Settings**. Use `↑` and `↓` to select
**Portuguese (Brazil)** or **English**, `Enter` to apply, and `Esc` to return
with the previous view and selection intact. The change applies for this run of
the panel, including when `--lang` was used at startup; it is not saved to the
workspace, and a workspace reload keeps the selected language.

### Histórico

Four levels, each deepening the one before: the **specs** (newest first, with the
`done/total` of their items), the spec's **backlog items**, the item's **tasks**
(the changelog records) and the task's **detail**, with Result, Evidence,
Criteria and Decisions. Each list row takes one line and ends in `…` when it does
not fit; the detail wraps its text and never cuts it. The list scrolls to keep
the selection visible and says how many rows there are above and below.

A spec whose changelog has a `## Closed` section is **closed** (`closed ·` before
its count in Portuguese: `fechada ·`): its entries left `BACKLOG.md` and Histórico
is where they stay visible, listed first among the spec's items. An entry dropped
with `[-]` shows as `× descartado`, in the muted color, and is left out of the
`done/total` count because it is no longer work to do; opening it says why
(`Descartado: <reason>`). Agora never shows dropped entries.

| Key | Effect |
| --- | --- |
| `↑` `↓`, `j` `k`, mouse wheel | move the selection (in the detail, scroll) |
| `PgUp` `PgDn` | move a page |
| `Enter` or a click on a row | opens the next level |
| `Esc` or `Backspace` | go back one level; from the specs level, back to Agora. In Histórico `Esc` goes back instead of quitting |
| `Tab`, `t` | switch between Agora and Histórico, at the same level and selection |
| `r` | reads the workspace again |
| `c` | opens Settings to change the language for this run |
| `q`, `Ctrl-C` | quit |

The mouse is captured only while Histórico is open, so Agora still lets you select
and copy text; the capture is turned off when going back to Agora, on quitting
and on an internal error. While it is on, many terminals ask for a modifier key
(`Shift`, or `Option` in iTerm2) to select text. Every mouse gesture has a key
equivalent. Below 40 columns Histórico shows only a notice, and the keys keep
working.

External `SIGTERM`, `SIGINT` and `SIGHUP` signals also restore raw mode, the
alternate screen and mouse capture before exiting with status `128 + signal`
(`143`, `130` and `129`, respectively).

### What the screen shows

- **Handoff**: the status (`Em andamento`, `Bloqueado`), who left it and when,
  the objective and the next step. When blocked, it also shows the blocker and
  the condition to resume.
- **TODO**: a bar with one segment per subtask and the list, with `✓` done, `●`
  in progress, `○` available, `◌` waiting on another subtask and `!` blocked.
- **Spec in hand and pending specs**: the backlog grouped by spec. The **spec
  card** is titled with the spec's id and title, with `em curso` (or `a seguir`
  when nothing is active, the first available item's spec) and `done/total` on the
  right, and lists the spec's items that are not done: `●` in progress, `○`
  available, `◌` waiting (with `após B-NNN`) and `!` blocked. **Specs pendentes**
  has one line per other spec with work left, and **Sem spec** for the items that
  point at no spec file. When the screen is short they give way in this order:
  the pending specs become one line, the spec card is cut at `+N itens`, and both
  become a one-line count of the backlog.
- **Next step**, one line above the footer: what to do next and which skill to
  call (for example `Retome T-002 de B-001 com relay-session.`), worked out from
  the same records. The panel only suggests: it never runs anything.
- **`● atualizando` / `● atualizado`**, in the corner: the panel waits for the
  files to stop changing (150 ms) and reads everything again.
- **`Inconsistente`**, in red: the records contradict each other, and the panel
  lists each violation. It is the same state the `relay-status` skill reports.

The color follows the status (green in progress or done, blue ready or
available, yellow blocked, red inconsistent), but every state is also spelled
out in words, never by color alone. Fixed interface text is available in
English and Brazilian Portuguese. Choose it with `relay-tui --lang en` or
`relay-tui --lang pt-BR`. Without that option, a valid `Language` entry in
`.orchestration/SETTINGS.md` wins, then `LC_ALL`, `LC_MESSAGES`, and `LANG`
are checked in that order (`pt*` selects Brazilian Portuguese); English is the
fallback. The help and command-line errors use the same language. Text from
workspace records, including objective and next step, is shown as written.

When there is not enough height, the next-step line goes first, then the Handoff
is compacted and the TODO is cut at `+N itens`. Below 40
columns only the header and the status are shown. A directory without
`.orchestration/` shows "Não é um workspace Relay" and switches to the state as
soon as Relay is installed there (`relay-setup`).

The background is not painted: it comes from the terminal. The colors assume a
dark terminal.

## Known limits

- Dark theme only.

## From source

Without downloading anything, with Rust installed (`rustup`):

```sh
cd app/relay-tui
cargo build --release
./target/release/relay-tui --workspace /path/to/repo
```
