import { describe, expect, it, vi } from 'vitest'

const apiPostJson = vi.fn()
vi.mock('../src/lib/relay-client.ts', () => ({ apiPostJson }))

let exitCb: ((code: number | null) => void) | null = null
const fakeSocket = {
  connect: vi.fn(),
  close: vi.fn(),
  send: vi.fn(),
  onData: vi.fn(),
  onExit: (cb: (code: number | null) => void) => {
    exitCb = cb
  },
  onDisk: vi.fn(),
}
vi.mock('../src/lib/term-client.ts', () => ({ createRunSocket: () => fakeSocket }))

const { launchEmbedded, terminateRun, useExecution } = await import('../src/lib/execution.ts')

describe('pedir término só confirma conclusão depois da saída real do host (A-008)', () => {
  it('terminateRun não marca exited otimista; só o exit real via socket muda o status', async () => {
    apiPostJson.mockResolvedValueOnce({ runId: 'run-1', scrollback: '' })
    await launchEmbedded({
      harness: 'claude-code',
      harnessName: 'Claude Code',
      skill: 'relay-session',
      intent: 'x',
      processName: 'p',
    })

    const exec = useExecution()
    expect(exec.status).toBe('running')

    apiPostJson.mockResolvedValueOnce({ terminated: true })
    terminateRun()

    // logo depois de pedir término, sem resposta do host ainda: continua running
    expect(exec.status).toBe('running')

    // a saída real chega pelo WebSocket de terminal
    exitCb?.(0)
    expect(exec.status).toBe('exited')
  })
})
