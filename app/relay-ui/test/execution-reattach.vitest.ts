import { describe, expect, it, vi } from 'vitest'

const apiPostJson = vi.fn()
vi.mock('../src/lib/relay-client.ts', () => ({ apiPostJson }))

let diskCb: ((entries: unknown[]) => void) | null = null
const fakeSocket = {
  connect: vi.fn(),
  close: vi.fn(),
  send: vi.fn(),
  onData: vi.fn(),
  onExit: vi.fn(),
  onDisk: (cb: (entries: unknown[]) => void) => {
    diskCb = cb
  },
}
vi.mock('../src/lib/term-client.ts', () => ({ createRunSocket: () => fakeSocket }))

const { launchEmbedded, reattachRun, useDisk } = await import('../src/lib/execution.ts')

describe('reanexar reenvia os diffs acumulados sem duplicar entradas já conhecidas (A-006)', () => {
  it('a mesma entrada (mesmo id) recebida de novo no reattach não duplica na lista', async () => {
    apiPostJson.mockResolvedValueOnce({ runId: 'run-1', scrollback: '' })
    await launchEmbedded({
      harness: 'claude-code',
      harnessName: 'Claude Code',
      skill: 'relay-session',
      intent: 'x',
      processName: 'p',
    })

    const entry = { id: 1, type: 'updated', path: 'a', at: '', meaning: '', before: '', after: '' }
    diskCb?.([entry])
    expect(useDisk().length).toBe(1)

    // detach + reattach: o servidor reenvia TODOS os diffs acumulados, incluindo o já visto
    reattachRun()
    diskCb?.([entry])

    expect(useDisk().length).toBe(1)
    expect(useDisk()[0]).toEqual(entry)
  })

  it('uma entrada nova (id diferente) no reattach é adicionada normalmente', async () => {
    const entry2 = { id: 2, type: 'updated', path: 'b', at: '', meaning: '', before: '', after: '' }
    diskCb?.([entry2])
    expect(useDisk().length).toBe(2)
  })
})
