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
| `skills/relay-*/SKILL.md` | Before changing one skill. Each is under 40 lines; read the one you are changing, not all five. |
| `docs/adr/NNNN-*.md` | Before making or revisiting an architectural decision. Index below. |
| `docs/INSTALL.md` | When changing installation for Claude Code, Codex, or OpenCode. |
| `docs/TUI.md` | When changing how the `relay-tui` is installed or used. |
| `README.md` | When changing what Relay claims to do or how it is explained. |
| `app/` | Before touching the interface. The folder carries its own `AGENTS.md`; its boundary is ADR-0004 and the data crossing it is ADR-0003. |
| `docs/design-system/` | Before any UI change. The folder carries its own `AGENTS.md` with the reading order and the rule that its `.html` files are never read by an agent; the `relay-design-system` skill carries the same rules and fires on intent. Start at its `README.md`. |

## Architecture decisions

ADRs live in `docs/adr/NNNN-<slug>.md`, numbered sequentially, in the format
Title, Status, Context, Decision, Consequences, Compliance, Notes. Status is
`Proposed`, `Accepted`, or `Superseded`. Never delete a superseded ADR;
supersede it and keep the chain, because the chain is the answer to "why not
the other option?".

Write one when a decision affects structure, an architecture characteristic, a
dependency, an interface, or a construction technique. Record the reasoning,
not only the choice.

- `docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md` — Accepted —
  `relay-tui`, a read-only, passive terminal observer in Rust (`ratatui`),
  shipped as a single downloadable binary from a GitHub Release (macOS
  is the requirement, Linux is best effort and Windows is out for now). It
  carries a second implementation of the core, held in line with `relay-core` by
  a language-neutral conformance suite in `app/conformance/`; it revises
  ADR-0001 decision 2 and the repository-only distribution premise for the TUI
  alone, and keeps decision 5 (the app never writes a record). It is not a Relay
  CLI. A file-based question channel between harness and TUI is recorded as a
  considered evolution, gated on a protocol amendment and a Claude Code spike.
- `docs/adr/0008-testes-de-ui-com-vitest.md` — Accepted — UI component tests
  use Vitest + `@vue/test-utils` + `jsdom`, as a `devDependency` of
  `app/relay-ui` only, alongside (not replacing) the existing `node --test`
  suite; Playwright and Testing Library were considered and rejected because
  none of spec 013's six regression scenarios need a real browser or a second
  DOM-query layer.
- `docs/adr/0006-contrato-http-ws-do-relay-host.md` — Accepted — the HTTP/WS
  surface between `relay-host` and `relay-ui`: bind only on `127.0.0.1` on an
  ephemeral port, a per-execution token delivered in the initial HTML (never
  in URL or query string), same-origin required on every API route with
  `GET /` as the sole token bootstrap, raw-content routes for the second view,
  the launch route reserved for spec 008 and absent under `--no-exec`
  (authed request gets `404`, not `403`), harness detection on its own
  endpoint outside the `UiPayload`, and whole-directory watching that pushes a
  fresh `UiPayload` over WebSocket.
- `docs/adr/0007-observador-read-only-como-primeira-entrega.md` — Accepted —
  the first release opens read-only by default, keeps integrated execution
  behind explicit `--exec`, accepts an explicit workspace path, preserves the
  Agora/Trabalho hierarchy, and exposes transition/stale state while publishing
  snapshots after 150 ms of quiescence. Decision 6 (added after visual
  validation) serves the changelog **structured** as well, through
  `relay-core`'s existing parser and a dedicated host route, so the Trabalho
  view can cascade spec → backlog cards → changelog cards without the UI ever
  parsing protocol grammar; the same revision drops the full spec text from
  that view, and decision 3 carries the note.
- `docs/adr/0001-arquitetura-inicial-da-ui.md` — Accepted — initial UI
  architecture: the `relay-core` / `relay-host` / `relay-ui` boundary over
  loopback HTTP and WebSocket, TypeScript throughout, browser UI with the
  application shell deferred, and the rule that the application never writes a
  protocol record.
- `docs/adr/0005-framework-da-relay-ui.md` — Accepted — Vue 3 for `relay-ui`,
  with Lit, Preact and React + Vite rejected; the view holds no logic; authoring
  in SFC with TypeScript, because ADR-0003's guarantee about `inconsistent` is
  the compiler's. Records that migration cost was weighed and **discarded** as a
  criterion, since the loopback boundary already makes the UI cheap to replace.
  Expected to be superseded; nothing else depends on it.
- `docs/adr/0004-fronteira-e-estrutura-do-app.md` — Accepted — where the UI
  lives and what it may not do: everything under `app/`, no root `package.json`,
  no mandatory build step, the folder carrying its own instructions, and a third
  layer declared — package surface, product, repository tooling — so the UI is
  never shipped to someone who only wanted the skills.
- `docs/adr/0003-contrato-do-estado-derivado.md` — Accepted — the derived-state
  contract between `relay-core` and `relay-ui`: content in and state out with no
  disk access, `inconsistent` as a separate shape rather than another status,
  availability derived in the core, counts but never percentages or positions,
  and no presentation field, so the design system keeps authority over how state
  looks.
- `docs/adr/0002-carregamento-condicional-de-instrucoes.md` — Accepted — how
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
- `.agents/`, `.claude/`, and `.opencode/plugin/` are tooling for developing
  *this* repository, never package surface: the manifests ship `./skills/` only.

## Development rules

- Keep Relay harness-neutral. Do not require a custom UI or rely on chat memory
  that another harness cannot access.
- Keep statuses in English: `backlog`, `ready`, `in_progress`, `blocked`,
  `done`, and `idle`.
- Do not add a Relay CLI until the Markdown protocol has been validated in real
  repositories.
- Keep installation guidance aligned across Claude Code, Codex, and OpenCode.
- Keep each document in its layer: the contract in `docs/PROTOCOL.md`,
  decisions and their reasoning in `docs/adr/`, exploratory analysis in
  `docs/design-system/`. Do not copy content between layers.
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
