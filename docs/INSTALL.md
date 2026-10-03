# Install Relay

**English** · [Português](INSTALL.pt-BR.md)

Relay version one is a plugin/skill package. It has no `relay` command-line
program: the skills read and write the protocol files directly. Other clients
may read, validate, derive state, and launch a harness with a Relay skill, but
they do not mutate the five protocol records themselves.

The public repository is `https://github.com/fabianogoes/ai-relay`.

For local development, replace `/absolute/path/to/relay` below with this
checkout's absolute path.

Every install below links one symlink per skill rather than symlinking the
`skills/` directory itself. Relay then sits alongside whatever skills the
project already has, and the command still works when `.claude/skills/`,
`.agents/skills/`, or `.opencode/skills/` already exists — symlinking onto an
existing directory would nest the link inside it instead of replacing it.

## Claude Code

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

The package metadata is `.claude-plugin/plugin.json`. Once Relay has a public
GitHub repository and marketplace entry, this section will add the equivalent
remote installation command.

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

This repository's own `.opencode/plugin/` directory is not part of that
installation. It holds a development guard for contributors working on Relay
itself, and it is not distributed: the Codex manifest ships `./skills/` only.

## Updating

Relay uses one shared `skills/` directory. Refresh the checkout or installed
plugin, then start a new harness session so its skill registry is reloaded.
