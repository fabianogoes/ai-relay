import { test } from 'node:test'
import assert from 'node:assert/strict'

import { resolveHarnessName, wireRunLifecycle, type RunLifecycleServer } from '../src/index.ts'
import type { EmbeddedRun } from '../src/executor.ts'
import type { DiskEntry } from '../src/disk.ts'
import type { Harness } from '../src/harness.ts'

function fakeRun(diffResults: DiskEntry[][]): {
  run: EmbeddedRun
  fireData: (chunk: string) => void
  fireExit: (code: number | null) => void
} {
  const dataCbs = new Set<(chunk: string) => void>()
  const exitCbs = new Set<(code: number | null) => void>()
  let call = 0
  const run = {
    runId: 'run-1',
    plan: { bin: '/bin/sh', args: [], prompt: '', cwd: '/tmp' },
    processName: 'sh',
    handle: {
      runId: 'run-1',
      status: () => 'exited' as const,
      exitCode: () => 0,
      startedAt: () => new Date().toISOString(),
      write: () => {},
      terminate: () => {},
      scrollback: () => '',
      onData: (cb: (chunk: string) => void) => {
        dataCbs.add(cb)
        return () => dataCbs.delete(cb)
      },
      onExit: (cb: (code: number | null) => void) => {
        exitCbs.add(cb)
        return () => exitCbs.delete(cb)
      },
    },
    disk: {
      entries: () => [],
      diff: () => diffResults[Math.min(call++, diffResults.length - 1)] ?? [],
    },
  } as unknown as EmbeddedRun

  return {
    run,
    fireData: (chunk) => {
      for (const cb of dataCbs) cb(chunk)
    },
    fireExit: (code) => {
      for (const cb of exitCbs) cb(code)
    },
  }
}

test('wireRunLifecycle manda o diff final por termDisk antes de termExit quando há mudança (A-007)', () => {
  const { run, fireExit } = fakeRun([[{ id: 1, type: 'updated', path: 'a', at: '', meaning: '', before: '', after: '' }]])
  const calls: string[] = []
  const server: RunLifecycleServer = {
    termBroadcast: () => calls.push('broadcast'),
    termDisk: () => calls.push('disk'),
    termExit: () => calls.push('exit'),
  }
  wireRunLifecycle(run, server)
  fireExit(0)
  assert.deepEqual(calls, ['disk', 'exit'])
})

test('wireRunLifecycle não manda termDisk vazio quando o diff final não tem mudança', () => {
  const { run, fireExit } = fakeRun([[]])
  const calls: string[] = []
  const server: RunLifecycleServer = {
    termBroadcast: () => calls.push('broadcast'),
    termDisk: () => calls.push('disk'),
    termExit: () => calls.push('exit'),
  }
  wireRunLifecycle(run, server)
  fireExit(0)
  assert.deepEqual(calls, ['exit'])
})

test('wireRunLifecycle repassa dados de onData por termBroadcast', () => {
  const { run, fireData } = fakeRun([[]])
  const calls: string[] = []
  const server: RunLifecycleServer = {
    termBroadcast: (_id, data) => calls.push(data),
    termDisk: () => {},
    termExit: () => {},
  }
  wireRunLifecycle(run, server)
  fireData('hello')
  assert.deepEqual(calls, [JSON.stringify({ kind: 'data', runId: 'run-1', data: 'hello' })])
})

test('resolveHarnessName: usa o nome do harness detectado quando encontrado, cai para o id quando desconhecido (A-009)', () => {
  const harnesses: Harness[] = [{ id: 'claude-code', name: 'Claude Code', version: '1.0.0', state: 'installed' }]
  assert.equal(resolveHarnessName(harnesses, 'claude-code'), 'Claude Code')
  assert.equal(resolveHarnessName(harnesses, 'harness-desconhecido'), 'harness-desconhecido')
})
