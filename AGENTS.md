# Relay Agent Instructions

Relay is a portable operational-memory protocol for coding agents. This
repository develops the Relay package **and is managed by Relay itself**: the
work of building Relay is recorded in its own `.specs/` and `.orchestration/`.

Friction found while working this way is a defect in `docs/PROTOCOL.md`, to be
fixed there. It is never a reason to add a second convention beside the
protocol this repository owns.

## How to load context

This file is read in full at the start of every session. Everything it names is
read on demand. Do not restate `docs/PROTOCOL.md`, a skill, or an ADR here: a
summary that drifts from its source is worse than a pointer to it.

| Read | When |
| --- | --- |
| `docs/PROTOCOL.md` | Before changing how any skill reads or writes a Relay record. It is the on-disk contract and holds every transition rule and integrity check in full. |
| `skills/relay-*/SKILL.md` | Before changing one skill. Each is under 40 lines; read the one you are changing, not all of them. |
| `docs/adr/NNNN-*.md` | Before making or revisiting an architectural decision. Index below. |
| `docs/INSTALL.md` | When changing installation for Claude Code, Codex, or OpenCode. |
| `docs/TUI.md` | When changing how the `relay-tui` is installed or used. |
| `README.md` | When changing what Relay claims to do or how it is explained. |
| `app/` | Before touching the interface, the `relay-tui` terminal panel. The folder carries its own `AGENTS.md`. |
| `app/relay-tui/DESIGN.md` | Before any change to how the `relay-tui` looks: palette, status, layout, keys. |

## Architecture decisions

ADRs live in `docs/adr/NNNN-<slug>.md`, numbered sequentially, in the format
Title, Status, Context, Decision, Consequences, Compliance, Notes. Status is
`Proposed`, `Accepted`, or `Superseded`. Never delete a superseded ADR;
supersede it and keep the chain, because the chain is the answer to "why not
the other option?".

Write one when a decision affects structure, an architecture characteristic, a
dependency, an interface, or a construction technique. Record the reasoning,
not only the choice.

- `docs/adr/0010-visibilidade-da-entrevista-da-spec-na-tui.md` — Accepted —
  the `idle` screen explains that the `relay-spec` interview happens in the
  agent and that the spec appears after it is saved; the derived state stays
  `idle` until a record changes.
- `docs/adr/0008-configuracoes-locais-da-relay-tui.md` — Accepted — `c` opens
  a local Settings screen from Agora or Histórico; the selected language
  overrides startup resolution for this process, preserves navigation and
  reloads, and never writes workspace configuration. Mouse capture pauses on
  Settings and resumes when returning to Histórico.
- `docs/adr/0009-instalacao-sem-checkout.md` — Accepted — the project-local
  installer fetches only Relay skill files, keeps `relay-tui` current, and
  leaves protocol-record creation to `relay-setup`.
- `docs/adr/0007-ci-do-pacote.md` — Accepted — the package has its own workflow,
  `package-ci.yml`, apart from the `relay-tui` ones: the `.agents/tests/` suites
  on macOS and `shellcheck -S warning` on Ubuntu, triggered by `skills/**`,
  `.agents/tests/**` and `docs/PROTOCOL.md`; the `relay-tui` workflows stay
  limited to their two files.
- `docs/adr/0006-changelog-por-spec-e-fechamento-como-arquivamento.md` —
  Accepted — the changelog is one file per spec in
  `.orchestration/changelog/`, closing a spec archives its backlog entries into
  that changelog, entries are dropped with `[-]` and a reason (never deleted), a
  legacy `CHANGELOG.md` stays readable and is migrated by `relay-setup`.
- `docs/adr/0005-skill-de-pacote-fora-do-protocolo.md` — Accepted —
  `relay-tui-split`, a package skill that opens a terminal split running the
  `relay-tui` and touches no record: it lives in `skills/` but outside the
  protocol, with a name distinct from the binary, a POSIX `sh` script as the
  first executable in the package, and its tests in `.agents/tests/`, not in
  `skills/`.
- `docs/adr/0004-navegacao-da-relay-tui-como-estado-local.md` — Accepted —
  navigation in the `relay-tui` is local screen state and stays read-only: a
  second view, **Histórico**, beside Agora, with `Esc` going back instead of
  quitting there, mouse capture only while Histórico is open (every gesture with
  a key equivalent), and the changelog and spec-title extraction kept out of the
  protocol rules.
- `docs/adr/0003-relay-tui-observador-de-terminal-em-rust.md` — Accepted —
  `relay-tui`, the only interface: a read-only terminal observer in Rust
  (`ratatui`), shipped as a single downloadable binary from a GitHub Release
  (macOS is the requirement, Linux is best effort and Windows is out for now).
  Its pure `core` is the only protocol reader and fixes the derived-state
  contract (content in, state out; `inconsistent` as a separate shape; counts,
  never percentages; stable integrity-check ids), checked by golden fixtures in
  `app/relay-tui/tests/fixtures/`. It re-reads the workspace after 150 ms of
  quiescence, is not a Relay CLI, and records a file-based question channel as a
  considered evolution, gated on a protocol amendment and a Claude Code spike.
- `docs/adr/0002-fronteira-e-estrutura-do-app.md` — Accepted — where the
  interface lives and what it may not do: everything under `app/`, no build
  manifest at the root, no build step between clone and skills, the folder
  carrying its own instructions, and a third layer declared — package surface,
  product, repository tooling — so the interface is never shipped to someone
  who only wanted the skills.
- `docs/adr/0001-carregamento-condicional-de-instrucoes.md` — Accepted — how
  agent instructions reach a session: each rule sits at the trigger where it
  becomes relevant (skill, nested `AGENTS.md`, tool hook) instead of in the root
  router; one executable rule with a shell per harness; the neutral directory
  holds the file and each harness gets a per-item symlink; repository tooling is
  never package surface.

## Package boundaries

- `skills/relay-*` is the canonical, shared skill source.
- `docs/PROTOCOL.md` defines the on-disk contract. Change it before changing a
  skill's interpretation of a Relay record, never after. Per-skill
  responsibilities live there and are not repeated here.
- `relay-setup` is idempotent and adds a delimited Relay section to an existing
  `AGENTS.md`; it never replaces local instructions. It generates that section
  itself and does not copy this file.
- Clients and interfaces may read, validate, derive state, and launch a
  harness, but only Relay skills mutate the five protocol records. If a client
  cannot derive a state it needs, change the protocol rather than adding a
  private write.
- `.agents/` and `.claude/` are tooling for developing *this* repository,
  never package surface: the manifests ship `./skills/` only.

## Development rules

- Keep Relay harness-neutral. Do not require a custom UI or rely on chat memory
  that another harness cannot access.
- Keep statuses in English: `backlog`, `ready`, `in_progress`, `blocked`,
  `done`, and `idle`.
- Do not add a Relay CLI until the Markdown protocol has been validated in real
  repositories.
- Keep installation guidance aligned across Claude Code, Codex, and OpenCode.
- Keep each document in its layer: the contract in `docs/PROTOCOL.md`,
  decisions and their reasoning in `docs/adr/`, the terminal design in
  `app/relay-tui/DESIGN.md`. Do not copy content between layers.
- The package surface is English: `README.md`, `docs/PROTOCOL.md`,
  `docs/INSTALL.md`, `docs/TUI.md`, and the skills. Each of those documents has
  a Portuguese (Brazil) translation beside it, `<name>.pt-BR.md`. Change both in
  the same commit; the code blocks (the on-disk contract and the commands) stay
  identical, and the English file wins when they diverge. ADRs and design analysis are currently
  written in Portuguese; keep each document in the language it already uses.

## Relay Protocol

Read `.orchestration/HANDOFF.md`, `.orchestration/TODO.md`,
`.orchestration/BACKLOG.md`, and the spec they reference in `.specs/` before
starting Relay work. Work the user requests directly may skip the flow; see
"Work outside the flow" in `docs/PROTOCOL.md`. Clients and interfaces may read,
validate, derive state, and launch a harness, but only Relay skills may mutate
the five protocol records.
