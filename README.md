# Relay

**English** · [Português](README.pt-BR.md)

Relay gives coding agents operational memory. The state of the work lives in
Markdown files in the repository, not in the chat history, so any agent (Claude
Code, Codex, OpenCode) can resume where another one stopped.

## Why it exists

You start a task in Codex, stop halfway (by choice or because you hit a usage
limit) and want to continue in Claude Code. The new agent does not know what was
decided or where you stopped, and the one who rebuilds that context is you.

Relay keeps that context in the repository:

- what to do and why (the spec);
- what is left (backlog and subtasks);
- what is in progress right now, who left it and when (the handoff);
- what was completed, with evidence (the changelog).

## How to use it

| Skill | When to use |
| --- | --- |
| `relay-setup` | Once, to install the protocol in the repository. |
| `relay-spec` | When you have an idea: an interview turns it into a spec and tasks. |
| `relay-session` | When you open a session: resumes or starts the work. |
| `relay-continue` | When you want to know what the next step is. |
| `relay-status` | To see the state without changing anything. |

## Terminal panel

Relay has a read-only terminal panel, `relay-tui`: leave it in a split next to
the harness and watch handoff, TODO and backlog change while the agent works. It
is a single binary, no Node.

## Documents

| Subject | English | Português |
| --- | --- | --- |
| **Installation** | [docs/INSTALL.md](docs/INSTALL.md) | [docs/INSTALL.pt-BR.md](docs/INSTALL.pt-BR.md) |
| **Files, states and rules** | [docs/PROTOCOL.md](docs/PROTOCOL.md) | [docs/PROTOCOL.pt-BR.md](docs/PROTOCOL.pt-BR.md) |
| **Terminal panel** | [docs/TUI.md](docs/TUI.md) | [docs/TUI.pt-BR.md](docs/TUI.pt-BR.md) |

## Frequently asked questions

**Aren't spec, backlog, TODO, handoff and changelog too much bureaucracy?**
They would be, if you wrote those files. The skills write them. You answer an
interview when you have a new idea and pick among options when you open a
session. The rest is record the agent keeps while it works.

**Doesn't this increase my cognitive load?**
The load is already there: without Relay, you are the one holding the context
between sessions and agents. Relay takes that state out of your head and puts it
in files any agent can read.

**Do I need Relay for every task?**
No. To fix a typo or a five-minute bug, ask the agent directly, without invoking
any skill. Relay pays off on work that spans sessions, agents or interruptions.

**What does it cost?**
The spec interview takes time before the first code, and every session starts by
reading the records, which spends tokens. In return, resuming does not depend on
anyone's memory.

**What if I stop halfway through a task?**
In the next session, in any harness, `relay-session` reads the handoff, finds the
subtask in progress and asks whether to resume from there.

**What if the records are wrong?**
The agent stops, explains the problem and asks before acting. It never keeps
working on top of an inconsistent state.

**Do I need a specific harness, a UI or a CLI?**
No. Relay is Markdown in the repository plus skills for Claude Code, Codex and
OpenCode. Interfaces may display the state but never write the records;
`relay-tui` is one of them (see [docs/TUI.md](docs/TUI.md)).

**Does Relay replace my project manager?**
No. It keeps the minimum agents need to work with continuity, and not roadmap,
priority or estimates.

## License

[MIT](LICENSE).
