import { test } from 'node:test'
import assert from 'node:assert/strict'

import { createLatestRequest, reconcileSpecId } from '../src/lib/work.ts'

test('selecao preserva a spec existente e cai deterministicamente para a primeira', () => {
  const specs = [{ id: '002.md' }, { id: '001.md' }]
  assert.equal(reconcileSpecId(specs, '001.md'), '001.md')
  assert.equal(reconcileSpecId(specs, 'removed.md'), '002.md')
  assert.equal(reconcileSpecId([], 'removed.md'), null)
})

test('nova requisicao aborta a anterior e somente a revisao atual pode publicar', () => {
  const requests = createLatestRequest()
  const first = requests.start()
  const second = requests.start()

  assert.equal(first.signal.aborted, true)
  assert.equal(requests.isCurrent(first.revision), false)
  assert.equal(second.signal.aborted, false)
  assert.equal(requests.isCurrent(second.revision), true)
})

test('selecao de backlog cai no primeiro card e nunca fica orfa', () => {
  const cards = [{ id: 'B-002' }, { id: 'B-001' }]
  assert.equal(reconcileSpecId(cards, null), 'B-002')
  assert.equal(reconcileSpecId(cards, 'B-001'), 'B-001')
  assert.equal(reconcileSpecId(cards, 'B-999'), 'B-002')
  assert.equal(reconcileSpecId([], 'B-001'), null)
})
