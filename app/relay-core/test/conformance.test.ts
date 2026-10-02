import { test } from 'node:test'
import assert from 'node:assert/strict'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { deriveState } from '../src/index.ts'
import type { InconsistentState, RelayFiles } from '../src/index.ts'

const CONFORMANCE_DIR = fileURLToPath(new URL('../../conformance', import.meta.url))

const STATUS_CASES = ['idle', 'backlog', 'ready', 'in_progress', 'blocked', 'done', 'inconsistent']

// As 13 verificacoes de integridade do protocolo (docs/PROTOCOL.md).
const CHECK_IDS = [
  'handoff-names-no-pending-todo',
  'backlog-id-mismatch',
  'spec-path-mismatch',
  'handoff-harness-invalid',
  'handoff-updated-invalid',
  'multiple-handoffs',
  'todo-cleared-before-changelog',
  'backlog-done-with-pending-todo',
  'unknown-marker',
  'needs-unknown-id',
  'needs-cycle',
  'needs-incomplete-on-done',
  'criteria-without-evidence',
]

function safeRead(path: string): string {
  return existsSync(path) ? readFileSync(path, 'utf8') : ''
}

// Mesmo mapeamento que o host usa: um registro ausente le como string vazia e
// cada spec entra com a chave `.specs/<nome>`.
function loadWorkspace(workspace: string): RelayFiles {
  const orchestration = join(workspace, '.orchestration')
  const specsDir = join(workspace, '.specs')
  const specs: Record<string, string> = {}
  if (existsSync(specsDir)) {
    for (const name of readdirSync(specsDir).filter((f) => f.endsWith('.md')).sort()) {
      specs[`.specs/${name}`] = readFileSync(join(specsDir, name), 'utf8')
    }
  }
  return {
    backlog: safeRead(join(orchestration, 'BACKLOG.md')),
    todo: safeRead(join(orchestration, 'TODO.md')),
    handoff: safeRead(join(orchestration, 'HANDOFF.md')),
    changelog: safeRead(join(orchestration, 'CHANGELOG.md')),
    specs,
  }
}

const caseNames = existsSync(CONFORMANCE_DIR)
  ? readdirSync(CONFORMANCE_DIR, { withFileTypes: true })
      .filter((e) => e.isDirectory())
      .map((e) => e.name)
      .sort()
  : []

test('a suite tem os sete casos de status e um caso por verificacao de integridade', () => {
  const expected = [...STATUS_CASES.map((s) => `status-${s}`), ...CHECK_IDS.map((c) => `check-${c}`)].sort()
  assert.deepEqual(caseNames, expected)
})

for (const name of caseNames) {
  test(`conformance: ${name}`, () => {
    const dir = join(CONFORMANCE_DIR, name)
    assert.ok(existsSync(join(dir, 'expected.json')), `${name} sem expected.json`)
    assert.ok(existsSync(join(dir, 'workspace')), `${name} sem workspace/`)
    const expected = JSON.parse(readFileSync(join(dir, 'expected.json'), 'utf8')) as unknown
    assert.deepStrictEqual(deriveState(loadWorkspace(join(dir, 'workspace'))), expected)
  })
}

test('cada caso check-* produz exatamente a violacao que nomeia', () => {
  for (const id of CHECK_IDS) {
    const path = join(CONFORMANCE_DIR, `check-${id}`, 'expected.json')
    if (!existsSync(path)) continue
    const state = JSON.parse(readFileSync(path, 'utf8')) as InconsistentState
    assert.equal(state.kind, 'inconsistent', `check-${id} nao e inconsistent`)
    assert.deepEqual(
      state.violations.map((v) => v.check),
      [id],
      `check-${id} deve conter somente a violacao ${id}`,
    )
  }
})

test('cada caso status-* chega ao status que nomeia', () => {
  for (const status of STATUS_CASES) {
    const path = join(CONFORMANCE_DIR, `status-${status}`, 'expected.json')
    if (!existsSync(path)) continue
    const state = JSON.parse(readFileSync(path, 'utf8')) as { kind: string; status?: string }
    if (status === 'inconsistent') assert.equal(state.kind, 'inconsistent')
    else assert.equal(state.status, status, `status-${status} nao deriva ${status}`)
  }
})
