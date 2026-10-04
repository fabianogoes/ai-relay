# Install Relay

**English** · [Português](INSTALL.pt-BR.md)

Relay version one is a plugin/skill package. It has no `relay` command-line
program: the skills read and write the protocol files directly. Other clients
may read, validate, derive state, and launch a harness with a Relay skill, but
they do not mutate the five protocol records themselves.

The public repository is `https://github.com/fabianogoes/ai-relay`.

For local development, replace `/absolute/path/to/relay` below with this
checkout's absolute path.

The no-clone installer copies only Relay's published skill files into
`.claude/skills/` and preserves other skills. The development instructions
below use one symlink per skill rather than symlinking the `skills/` directory
itself, so they also work when the harness's skills directory already exists.

## Claude Code

### Install in the current project

From the root of a new project, run this command in a terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/fabianogoes/ai-relay/main/install.py | python3 -
```

It downloads only the Relay skill files from GitHub and installs them under
`.claude/skills/`. It does not clone or download the Relay repository. It also
checks the latest `relay-tui` release and installs or updates the user binary
when needed. It requires Python 3, which is used only from the standard library.
Run it again to update the skills and TUI safely. Start a new Claude Code
session, then run `/relay-setup` from the same project to initialize Relay's
project records.

Claude Code plugins discover skills under the plugin's `skills/` directory.
Test the checkout directly with:

```sh
git clone https://github.com/fabianogoes/ai-relay.git
claude --plugin-dir ./ai-relay
```

For a marketplace install, add the GitHub repository and install the `relay`
entry:

```text
/plugin marketplace add fabianogoes/ai-relay
/plugin install relay@relay
```

Alternatively, expose the shared skills directly in a target project:

```sh
mkdir -p .claude/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .claude/skills/"$skill"
done
```

The package metadata is `.claude-plugin/plugin.json`.

## Codex

The Codex plugin manifest is `.codex-plugin/plugin.json`, and it exposes the
canonical `./skills/` directory. Install the local checkout through the Codex
plugin development flow, or link the skills for repository-scoped development:

```sh
mkdir -p .agents/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .agents/skills/"$skill"
done
```

In Codex, open Plugins, choose the GitHub-imported `relay` marketplace, review
the listed skills, and select Install. The repository includes the Codex
catalog at `.agents/plugins/marketplace.json` for workspace import.

## OpenCode

OpenCode uses native Agent Skills discovery. Execute these commands to install
Relay from GitHub globally for your user:

```sh
git clone https://github.com/fabianogoes/ai-relay.git ~/.config/opencode/relay
mkdir -p ~/.config/opencode/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s ~/.config/opencode/relay/skills/"$skill" ~/.config/opencode/skills/"$skill"
done
```

Open a new OpenCode session and execute this test prompt:

```text
Use relay-status to report the current Relay state.
```

Expected result: OpenCode finds `relay-status` and reports the state without
altering files. OpenCode recognizes skills from `.opencode/skills/`,
`.claude/skills/`, and `.agents/skills/`, as well as the global
`~/.config/opencode/skills/` directory.

OpenCode does not turn `SKILL.md` files into `/relay` slash commands. The
skills appearing in `/skills` confirms discovery; invoke them through a
natural-language request such as `Use relay-status ...` and OpenCode loads
the matching skill with its native `skill` tool.

For project-local installation instead:

```sh
mkdir -p .opencode/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .opencode/skills/"$skill"
done
```

See [.opencode/INSTALL.md](../.opencode/INSTALL.md) for the adapter notes. The
OpenCode installation remains a native Agent Skills discovery link; no custom
runtime is installed.

## Set up another project on this machine

From the root of a new project, install the skills once for its harness. In
Claude Code, run the installer in the section above. In Codex or OpenCode, use
the matching installation instructions above. Start a new session and invoke
`relay-setup` in the harness:

| Harness | Invocation |
| --- | --- |
| Claude Code | `/relay-setup` |
| Codex or OpenCode | `Use relay-setup` |

`relay-setup` configures the repository where it runs. It creates missing Relay
files and adds the managed section to `AGENTS.md` without replacing existing
instructions or records. If no workspace language is set, it asks once and
saves the choice. Running it again is safe: it preserves existing content and
creates nothing that is already present. Run `relay-status` from the same
repository to check the result.

The `relay-tui` binary is installed for the machine, not copied into each
project. Run `relay-tui --version` and compare it with the latest version on
[Releases](https://github.com/fabianogoes/ai-relay/releases). If it is current,
no update is needed; if it is missing or older, follow [the TUI update steps in
TUI.md](TUI.md#update). Then invoke `relay-tui-split` from the new repository
to open the panel on that workspace. Review and commit the generated project
configuration that should be shared with the repository.

## Updating

Relay keeps one canonical `skills/` directory. To update a project-local
installation, run the install command above again from that project's root and
start a new Claude Code session so its skills are loaded again. For plugin and
checkout installations, refresh the plugin or checkout and start a new session.
