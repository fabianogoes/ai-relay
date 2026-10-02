import { test } from 'node:test'
import assert from 'node:assert/strict'

import { initHarnessSelection, selectConsent, selectHarness, selection, allHarnesses, setHarnesses, HARNESS_FIXTURE } from '../src/lib/harness.ts'

function makeStorage(): Storage {
  const map = new Map<string, string>()
  return {
    get length() {
      return map.size
    },
    clear() {
      map.clear()
    },
    getItem(key: string): string | null {
      return map.get(key) ?? null
    },
    key(index: number): string | null {
      return [...map.keys()][index] ?? null
    },
    removeItem(key: string) {
      map.delete(key)
    },
    setItem(key: string, value: string) {
      map.set(key, value)
    },
  } as Storage
}

test('reduzir consentimento de local/sessao para "So esta execucao" remove o valor persistido', () => {
  const local = makeStorage()
  globalThis.localStorage = local
  const workspace = '/work/project'

  initHarnessSelection(workspace)
  selectHarness(workspace, 'claude-code')
  selectConsent(workspace, 'local')
  assert.ok(local.getItem(`relay.harness:${workspace}`) !== null)

  selectConsent(workspace, 'none')
  assert.equal(local.getItem(`relay.harness:${workspace}`), null)

  initHarnessSelection(workspace)
  assert.equal(selection().consent, 'none')
  assert.equal(selection().harnessId, null)
})

test('consentimento de sessao nao grava em localStorage e some ao reduzir', () => {
  const local = makeStorage()
  globalThis.localStorage = local
  const workspace = '/work/other'

  initHarnessSelection(workspace)
  selectHarness(workspace, 'codex')
  selectConsent(workspace, 'session')
  assert.equal(local.getItem(`relay.harness:${workspace}`), null)

  selectConsent(workspace, 'none')
  initHarnessSelection(workspace)
  assert.equal(selection().consent, 'none')
})

test('setHarnesses sem fallback: lista vazia nao inventa fixture instalada', () => {
  setHarnesses([])
  assert.deepEqual(allHarnesses(), [])
  setHarnesses(HARNESS_FIXTURE)
  assert.equal(allHarnesses().length, HARNESS_FIXTURE.length)
})