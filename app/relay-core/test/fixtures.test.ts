import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { deriveState } from '../src/index.ts'
import type { RelayFiles } from '../src/index.ts'

const FIXTURES_DIR = fileURLToPath(new URL('../../fixtures', import.meta.url))
const SPEC_001 = '.specs/20260907-001-ui-primeiro-marco-visual.md'

function fixtureState(name: string): unknown {
  const raw = readFileSync(join(FIXTURES_DIR, `${name}.json`), 'utf8')
  return (JSON.parse(raw) as { state: unknown }).state
}

function files(partial: Partial<RelayFiles>): RelayFiles {
  return { backlog: '', todo: '', handoff: '', changelog: '', specs: {}, ...partial }
}

const EMPTY_TODO = '# Active task\n\nNo active task.\n'
const EMPTY_HANDOFF = '# Handoff\n\nNo active handoff.\n'

function changelogRecords(list: Array<{ backlog: string; todo: string; spec?: string; criteria?: string }>): string {
  const blocks = list.map(
    (r) =>
      `## 2026-09-07 - ${r.todo} - Registro\n` +
      `- Backlog: ${r.backlog}\n` +
      `- Spec: ${r.spec ?? SPEC_001}\n` +
      `- Result: nada.\n` +
      `- Evidence: nada.\n` +
      `- Criteria: ${r.criteria ?? 'none'}\n` +
      `- Decisions: none.\n`,
  )
  return `# Change log\n\n${blocks.join('\n')}\n`
}

test('idle', () => {
  const state = deriveState(files({ todo: EMPTY_TODO, handoff: EMPTY_HANDOFF }))
  assert.deepStrictEqual(state, fixtureState('idle'))
})

test('backlog', () => {
  const backlog = [
    `- [ ] B-001 - Contrato do estado derivado (spec: ${SPEC_001})`,
    `- [ ] B-002 - Fronteira e estrutura do app/ (spec: ${SPEC_001})`,
    `- [ ] B-003 - Framework da relay-ui (spec: ${SPEC_001})`,
    `- [ ] B-004 - Tela principal renderizando os sete fixtures (spec: ${SPEC_001}) (needs: B-001, B-002, B-003)`,
  ].join('\n')
  const state = deriveState(files({ backlog: `# Backlog\n\n${backlog}\n`, todo: EMPTY_TODO, handoff: EMPTY_HANDOFF }))
  assert.deepStrictEqual(state, fixtureState('backlog'))
})

test('ready', () => {
  const todo = [
    `- [x] T-001 - Materializar os sete fixtures`,
    `- [x] T-002 - Escrever tokens.css`,
    `- [ ] T-003 - Scaffold da relay-ui (needs: T-001, T-002)`,
  ].join('\n')
  const backlog = `- [ ] B-004 - Tela principal renderizando os sete fixtures (spec: ${SPEC_001})`
  const changelog = changelogRecords([
    { backlog: 'B-004', todo: 'T-001' },
    { backlog: 'B-004', todo: 'T-002' },
  ])
  const state = deriveState(
    files({ backlog: `# Backlog\n\n${backlog}\n`, todo: `# Active task: B-004\n\n${todo}\n`, handoff: EMPTY_HANDOFF, changelog }),
  )
  assert.deepStrictEqual(state, fixtureState('ready'))
})

test('in_progress', () => {
  const backlog = `- [ ] B-001 - Contrato do estado derivado (spec: ${SPEC_001})`
  const todo = [
    `- [x] T-001 - Escrever a ADR-0003`,
    `- [•] T-002 - Validar os fixtures (needs: T-001)`,
    `- [ ] T-003 - Registrar no índice (needs: T-001)`,
  ].join('\n')
  const handoff = [
    `# Handoff`,
    ``,
    `- Status: in_progress`,
    `- Backlog: B-001`,
    `- TODO: T-002`,
    `- Spec: ${SPEC_001}`,
    `- Harness: claude-code`,
    `- Updated: 2026-09-07T06:49:14Z`,
    ``,
    `## Objective`,
    `Validar os fixtures contra o protocolo.`,
    ``,
    `## Next step`,
    `Rodar as verificações de integridade.`,
    ``,
    `## Context`,
    `T-001 concluída; o tipo está na ADR-0003.`,
    ``,
  ].join('\n')
  const changelog = changelogRecords([{ backlog: 'B-001', todo: 'T-001' }])
  const state = deriveState(
    files({ backlog: `# Backlog\n\n${backlog}\n`, todo: `# Active task: B-001\n\n${todo}\n`, handoff, changelog }),
  )
  assert.deepStrictEqual(state, fixtureState('in_progress'))
})

test('blocked', () => {
  const todo = [
    `- [x] T-001 - Materializar os sete fixtures`,
    `- [x] T-002 - Escrever tokens.css`,
    `- [!] T-003 - Scaffold da relay-ui (needs: T-001, T-002)`,
  ].join('\n')
  const backlog = `- [ ] B-004 - Tela principal renderizando os sete fixtures (spec: ${SPEC_001})`
  const handoff = [
    `# Handoff`,
    ``,
    `- Status: blocked`,
    `- Backlog: B-004`,
    `- TODO: T-003`,
    `- Spec: ${SPEC_001}`,
    `- Harness: opencode`,
    `- Updated: 2026-09-07T11:00:00-03:00`,
    ``,
    `## Objective`,
    `Scaffold da relay-ui com Vue 3.`,
    ``,
    `## Next step`,
    `Resolvido o bloqueio, rodar a criação do projeto Vite.`,
    ``,
    `## Context`,
    `Bloqueio: node_modules não pode ser instalado sem acesso ao registro npm neste ambiente. Condição de retomada: acesso ao registro npm restabelecido.`,
    ``,
  ].join('\n')
  const changelog = changelogRecords([
    { backlog: 'B-004', todo: 'T-001' },
    { backlog: 'B-004', todo: 'T-002' },
  ])
  const state = deriveState(
    files({ backlog: `# Backlog\n\n${backlog}\n`, todo: `# Active task: B-004\n\n${todo}\n`, handoff, changelog }),
  )
  assert.deepStrictEqual(state, fixtureState('blocked'))
})

test('done', () => {
  const backlog = [
    `- [x] B-001 - Contrato do estado derivado (spec: ${SPEC_001})`,
    `- [x] B-002 - Fronteira e estrutura do app/ (spec: ${SPEC_001})`,
    `- [x] B-003 - Framework da relay-ui (spec: ${SPEC_001})`,
    `- [x] B-004 - Tela principal renderizando os sete fixtures (spec: ${SPEC_001}) (needs: B-001, B-002, B-003)`,
  ].join('\n')
  const spec = [
    `# 20260907-001 - Primeiro marco visual da relay-ui`,
    ``,
    `## Acceptance criteria`,
    `- A-001 - instalação funciona sem npm install`,
    `- A-002 - rm -rf app/ devolve o estado anterior`,
    `- A-003 - sete fixtures existem como arquivos`,
    `- A-004 - tela renderiza os sete fixtures`,
    `- A-005 - todo token vem de var(--token)`,
    `- A-006 - nada em app/ escreve nos registros`,
    `- A-007 - três ADRs registradas`,
    `- A-008 - AGENTS.md ganha uma linha`,
  ].join('\n')
  const changelog = changelogRecords([
    { backlog: 'B-001', todo: 'T-001', criteria: 'A-001, A-002, A-003, A-004, A-005, A-006, A-007, A-008' },
  ])
  const state = deriveState(
    files({
      backlog: `# Backlog\n\n${backlog}\n`,
      todo: EMPTY_TODO,
      handoff: EMPTY_HANDOFF,
      changelog,
      specs: { [SPEC_001]: spec },
    }),
  )
  assert.deepStrictEqual(state, fixtureState('done'))
})

test('inconsistent', () => {
  const backlog = `- [ ] B-001 - Contrato do estado derivado (spec: ${SPEC_001})`
  const todo = [
    `- [x] T-001 - Escrever a ADR-0003`,
    `- [•] T-002 - Validar os fixtures (needs: T-001)`,
    `- [ ] T-003 - Registrar no índice (needs: T-001)`,
  ].join('\n')
  const handoff = [
    `# Handoff`,
    ``,
    `- Status: in_progress`,
    `- Backlog: B-001`,
    `- TODO: T-001`,
    `- Spec: ${SPEC_001}`,
    `- Harness: claude-code`,
    `- Updated: 2026-09-07T06:49:14Z`,
    ``,
    `## Objective`,
    `Validar os fixtures contra o protocolo.`,
    ``,
    `## Next step`,
    `Rodar as verificações de integridade.`,
    ``,
    `## Context`,
    `T-001 concluída; o tipo está na ADR-0003.`,
    ``,
  ].join('\n')
  const changelog = changelogRecords([{ backlog: 'B-001', todo: 'T-001' }])
  const state = deriveState(
    files({ backlog: `# Backlog\n\n${backlog}\n`, todo: `# Active task: B-001\n\n${todo}\n`, handoff, changelog }),
  )
  assert.deepStrictEqual(state, fixtureState('inconsistent'))
})
