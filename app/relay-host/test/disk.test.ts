import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { startDiskTracker } from '../src/disk.ts'

function makeWorkspace(): { dir: string; cleanup: () => void } {
  const dir = mkdtempSync(join(tmpdir(), 'relay-disk-test-'))
  mkdirSync(join(dir, '.orchestration'), { recursive: true })
  return { dir, cleanup: () => rmSync(dir, { recursive: true, force: true }) }
}

function write(dir: string, name: string, content: string): void {
  writeFileSync(join(dir, '.orchestration', name), content)
}

test('registro indo para o estado vazio canônico (texto, não string vazia) é LIMPO (A-007)', () => {
  const ws = makeWorkspace()
  try {
    write(ws.dir, 'HANDOFF.md', '# Handoff\n\n- Status: in_progress\n')
    write(ws.dir, 'TODO.md', '# Active task: B-001\n\n- [ ] T-001 - x\n')
    const tracker = startDiskTracker(ws.dir)

    write(ws.dir, 'HANDOFF.md', '# Handoff\n\nNo active handoff.\n')
    write(ws.dir, 'TODO.md', '# Active task\n\nNo active task.\n')
    const entries = tracker.diff()

    const handoff = entries.find((e) => e.path.endsWith('HANDOFF.md'))
    const todo = entries.find((e) => e.path.endsWith('TODO.md'))
    assert.equal(handoff?.type, 'cleared')
    assert.equal(todo?.type, 'cleared')
  } finally {
    ws.cleanup()
  }
})

test('registro que ainda tem conteúdo real continua UPDATED, não LIMPO', () => {
  const ws = makeWorkspace()
  try {
    write(ws.dir, 'HANDOFF.md', '# Handoff\n\n- Status: in_progress\n')
    const tracker = startDiskTracker(ws.dir)
    write(ws.dir, 'HANDOFF.md', '# Handoff\n\n- Status: blocked\n')
    const entries = tracker.diff()
    assert.equal(entries.find((e) => e.path.endsWith('HANDOFF.md'))?.type, 'updated')
  } finally {
    ws.cleanup()
  }
})
