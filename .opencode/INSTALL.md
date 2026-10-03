# OpenCode Adapter

The canonical Relay skills live in the repository's `skills/` directory.
OpenCode recognizes Agent Skills from `.opencode/skills/`, `.claude/skills/`,
and `.agents/skills/`, in addition to its global skill directories.

## Install from GitHub

Clone the public repository into a stable local directory:

```sh
git clone https://github.com/fabianogoes/ai-relay.git ~/.config/opencode/relay
```

Expose each Relay skill in OpenCode's global discovery directory:

```sh
mkdir -p ~/.config/opencode/skills
ln -s ~/.config/opencode/relay/skills/relay-setup ~/.config/opencode/skills/relay-setup
ln -s ~/.config/opencode/relay/skills/relay-spec ~/.config/opencode/skills/relay-spec
ln -s ~/.config/opencode/relay/skills/relay-status ~/.config/opencode/skills/relay-status
ln -s ~/.config/opencode/relay/skills/relay-continue ~/.config/opencode/skills/relay-continue
ln -s ~/.config/opencode/relay/skills/relay-session ~/.config/opencode/skills/relay-session
ln -s ~/.config/opencode/relay/skills/relay-tui-split ~/.config/opencode/skills/relay-tui-split
```

Start a new OpenCode session and test the installation with:

```text
Use relay-status to report the current Relay state.
```

OpenCode does not expose these skills as `/relay` slash commands. Seeing
`relay-setup`, `relay-spec`, `relay-status`, `relay-continue`, `relay-session`, and
`relay-tui-split` in `/skills`
is the expected discovery result; the agent loads one by name through its
native `skill` tool when the prompt requires it.

To update later:

```sh
git -C ~/.config/opencode/relay pull --ff-only
```

For project-local development, create a symbolic link from one of those discovery
paths to this repository's `skills/` directory. For example, from a target
project:

```sh
mkdir -p .opencode
ln -s /absolute/path/to/relay/skills .opencode/skills
```

For an installed release, use the same layout with the release's local path.
Relay intentionally relies on OpenCode's native skill discovery rather than a
custom UI or a background service.
