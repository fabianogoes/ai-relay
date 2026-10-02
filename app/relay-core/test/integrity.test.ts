import { test } from 'node:test'
import assert from 'node:assert/strict'

import { deriveState } from '../src/index.ts'
import type { InconsistentState, RelayFiles } from '../src/index.ts'

const SPEC = '.specs/20260907-001-ui-primeiro-marco-visual.md'

const EMPTY_TODO = '# Active task\n\nNo active task.\n'
const EMPTY_HANDOFF = '# Handoff\n\nNo active handoff.\n'

function todoText(lines: string[], active = 'B-001'): string {
  return `# Active task: ${active}\n\n${lines.join('\n')}\n`
}

function backlogText(lines: string[]): string {
  return `# Backlog\n\n${lines.join('\n')}\n`
}

function handoffText(overrides: Partial<Record<string, string>> = {}, sections: Record<string, string> = {}): string {
  const fields: Record<string, string> = {
    Status: 'in_progress',
    Backlog: 'B-001',
    TODO: 'T-002',
    Spec: SPEC,
    Harness: 'claude-code',
    Updated: '2026-09-07T06:49:14Z',
    ...overrides,
  }
  const lines = ['# Handoff', '']
  for (const [key, value] of Object.entries(fields)) lines.push(`- ${key}: ${value}`)
  lines.push('')
  for (const [name, body] of Object.entries(sections)) lines.push(`## ${name}`, body, '')
  return lines.join('\n')
}

function changelogFor(todos: string[], backlog = 'B-001'): string {
  const blocks = todos.map(
    (t) => `## 2026-09-07 - ${t} - Registro\n- Backlog: ${backlog}\n- Spec: ${SPEC}\n- Criteria: none\n`,
  )
  return `# Change log\n\n${blocks.join('\n')}\n`
}

const DEFAULT_TODO = ['- [x] T-001 - primeiro', '- [•] T-002 - segundo (needs: T-001)']
const DEFAULT_BACKLOG = [`- [ ] B-001 - Contrato do estado derivado (spec: ${SPEC})`]

function base(files: Partial<RelayFiles> = {}): RelayFiles {
  return {
    backlog: backlogText(DEFAULT_BACKLOG),
    todo: todoText(DEFAULT_TODO),
    handoff: handoffText(),
    changelog: changelogFor(['T-001']),
    specs: {},
    ...files,
  }
}

function checks(files: RelayFiles): string[] {
  const state = deriveState(files)
  assert.equal(state.kind, 'inconsistent')
  return (state as InconsistentState).violations.map((v) => v.check)
}

test('entrada válida não produz violações', () => {
  assert.equal(deriveState(base()).kind, 'ok')
})

test('handoff-names-no-pending-todo', () => {
  assert.deepEqual(checks(base({ handoff: handoffText({ TODO: 'T-001' }) })), ['handoff-names-no-pending-todo'])
})

test('backlog-id-mismatch', () => {
  assert.deepEqual(
    checks(
      base({
        backlog: backlogText([
          `- [ ] B-001 - Contrato do estado derivado (spec: ${SPEC})`,
          `- [ ] B-002 - Outra entrada (spec: ${SPEC})`,
        ]),
        todo: todoText(DEFAULT_TODO, 'B-002'),
        changelog: changelogFor(['T-001'], 'B-002'),
      }),
    ),
    ['backlog-id-mismatch'],
  )
})

test('backlog-id-mismatch: backlog ativo ausente do backlog', () => {
  const violations = checks(
    base({
      backlog: '',
      todo: todoText(DEFAULT_TODO, 'B-999'),
      changelog: changelogFor(['T-001'], 'B-999'),
    }),
  )
  assert.ok(violations.includes('backlog-id-mismatch'))
})

test('spec-path-mismatch', () => {
  assert.deepEqual(
    checks(base({ handoff: handoffText({ Spec: '.specs/20260907-009-outra.md' }) })),
    ['spec-path-mismatch'],
  )
})

test('spec-path-mismatch: tarefa sem entrada de backlog confrontavel', () => {
  const violations = checks(
    base({
      backlog: '',
      todo: todoText(DEFAULT_TODO, 'B-999'),
      changelog: changelogFor(['T-001'], 'B-999'),
    }),
  )
  assert.ok(violations.includes('spec-path-mismatch'))
})

test('spec-path-mismatch: entrada ativa sem spec', () => {
  assert.deepEqual(
    checks(base({ backlog: backlogText(['- [ ] B-001 - Contrato do estado derivado']) })),
    ['spec-path-mismatch'],
  )
})

test('estado terminal: backlog todo [x] deriva done', () => {
  const spec = '## Acceptance criteria\n- A-001 - um\n'
  const changelog = `# Change log\n\n## 2026-09-07 - T-001 - Registro\n- Backlog: B-001\n- Spec: ${SPEC}\n- Criteria: A-001\n`
  const state = deriveState(
    base({
      backlog: backlogText([`- [x] B-001 - Contrato do estado derivado (spec: ${SPEC})`]),
      todo: EMPTY_TODO,
      handoff: EMPTY_HANDOFF,
      changelog,
      specs: { [SPEC]: spec },
    }),
  )
  assert.equal(state.kind, 'ok')
  if (state.kind === 'ok') assert.equal(state.status, 'done')
})

test('estado terminal: repositorio vazio deriva idle', () => {
  const state = deriveState({ backlog: '', todo: EMPTY_TODO, handoff: EMPTY_HANDOFF, changelog: '', specs: {} })
  assert.equal(state.kind, 'ok')
  if (state.kind === 'ok') assert.equal(state.status, 'idle')
})

test('handoff-harness-invalid', () => {
  assert.deepEqual(checks(base({ handoff: handoffText({ Harness: 'BAD HARNESS' }) })), ['handoff-harness-invalid'])
})

test('handoff-updated-invalid', () => {
  assert.deepEqual(checks(base({ handoff: handoffText({ Updated: 'ontem' }) })), ['handoff-updated-invalid'])
})

test('multiple-handoffs', () => {
  assert.deepEqual(checks(base({ handoff: `${handoffText()}\n- Status: in_progress\n` })), ['multiple-handoffs'])
})

test('todo-cleared-before-changelog', () => {
  assert.deepEqual(checks(base({ changelog: '' })), ['todo-cleared-before-changelog'])
})

test('backlog-done-with-pending-todo', () => {
  assert.deepEqual(
    checks(base({ backlog: backlogText([`- [x] B-001 - Contrato do estado derivado (spec: ${SPEC})`]) })),
    ['backlog-done-with-pending-todo'],
  )
})

test('unknown-marker', () => {
  assert.deepEqual(
    checks(base({ todo: todoText(['- [x] T-001 - primeiro', '- [?] T-002 - segundo']), handoff: EMPTY_HANDOFF })),
    ['unknown-marker'],
  )
})

test('needs-unknown-id', () => {
  assert.deepEqual(
    checks(
      base({
        todo: todoText([
          '- [x] T-001 - primeiro',
          '- [•] T-002 - segundo (needs: T-001)',
          '- [ ] T-003 - terceiro (needs: T-999)',
        ]),
      }),
    ),
    ['needs-unknown-id'],
  )
})

test('needs-cycle', () => {
  assert.deepEqual(
    checks(
      base({
        todo: todoText(['- [ ] T-001 - a (needs: T-002)', '- [ ] T-002 - b (needs: T-001)']),
        handoff: EMPTY_HANDOFF,
        changelog: '',
      }),
    ),
    ['needs-cycle'],
  )
})

test('needs-incomplete-on-done', () => {
  assert.deepEqual(
    checks(
      base({
        todo: todoText(['- [x] T-001 - a (needs: T-002)', '- [ ] T-002 - b']),
        handoff: EMPTY_HANDOFF,
      }),
    ),
    ['needs-incomplete-on-done'],
  )
})

test('criteria-without-evidence', () => {
  const spec = '## Acceptance criteria\n- A-001 - um\n- A-002 - dois\n'
  assert.deepEqual(
    checks(
      base({
        backlog: backlogText([`- [x] B-001 - Contrato do estado derivado (spec: ${SPEC})`]),
        todo: EMPTY_TODO,
        handoff: EMPTY_HANDOFF,
        changelog: '',
        specs: { [SPEC]: spec },
      }),
    ),
    ['criteria-without-evidence'],
  )
})

test('backticks do spec são removidos', () => {
  const state = deriveState(base({ backlog: backlogText([`- [ ] B-001 - Contrato (spec: \`${SPEC}\`)`]) }))
  assert.equal(state.kind, 'ok')
  if (state.kind === 'ok') assert.equal(state.backlog[0].spec, SPEC)
})

test('critério qualificado de outra spec conta como evidência', () => {
  const spec = '## Acceptance criteria\n- A-001 - um\n'
  assert.equal(
    deriveState(
      base({
        backlog: backlogText([`- [x] B-001 - Contrato do estado derivado (spec: ${SPEC})`]),
        todo: EMPTY_TODO,
        handoff: EMPTY_HANDOFF,
        changelog: `# Change log\n\n## 2026-09-07 - T-001 - Registro\n- Backlog: B-002\n- Spec: .specs/20260907-002-outra.md\n- Criteria: 20260907-001/A-001\n`,
        specs: { [SPEC]: spec },
      }),
    ).kind,
    'ok',
  )
})
