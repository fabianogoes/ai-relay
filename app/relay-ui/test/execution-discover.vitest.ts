import { describe, expect, it, vi, beforeEach } from 'vitest'

const apiGetJson = vi.fn()
const apiPostJson = vi.fn()
vi.mock('../src/lib/relay-client.ts', () => ({ apiGetJson, apiPostJson }))

const fakeSocket = {
  connect: vi.fn(),
  close: vi.fn(),
  send: vi.fn(),
  onData: vi.fn(),
  onExit: vi.fn(),
  onDisk: vi.fn(),
}
vi.mock('../src/lib/term-client.ts', () => ({ createRunSocket: () => fakeSocket }))

const { closeRun, discoverActiveRun, useExecution } = await import('../src/lib/execution.ts')

describe('discoverActiveRun: reabrir a aba durante uma run viva reanexa a ela (A-009)', () => {
  beforeEach(() => {
    apiGetJson.mockReset()
    apiPostJson.mockReset().mockResolvedValue({})
    fakeSocket.connect.mockClear()
    closeRun()
  })

  it('run ativa reportada pelo host é adotada e o socket é conectado', async () => {
    apiGetJson.mockResolvedValueOnce([
      {
        runId: 'run-9',
        status: 'running',
        exitCode: null,
        startedAt: '2026-01-01T00:00:00Z',
        harnessId: 'claude-code',
        harnessName: 'Claude Code',
        processName: 'claude · relay-session',
      },
    ])

    await discoverActiveRun()

    const execution = useExecution()
    expect(execution.activeRunId).toBe('run-9')
    expect(execution.harnessId).toBe('claude-code')
    expect(execution.harnessName).toBe('Claude Code')
    expect(execution.processName).toBe('claude · relay-session')
    expect(execution.reattached).toBe(true)
    expect(fakeSocket.connect).toHaveBeenCalledWith('run-9')
  })

  it('nenhuma run em execução no host não altera o estado local', async () => {
    apiGetJson.mockResolvedValueOnce([])

    await discoverActiveRun()

    const execution = useExecution()
    expect(execution.activeRunId).toBeNull()
    expect(fakeSocket.connect).not.toHaveBeenCalled()
  })

  it('já existindo uma run ativa localmente, não consulta o host de novo', async () => {
    apiGetJson.mockResolvedValueOnce([
      {
        runId: 'run-9',
        status: 'running',
        exitCode: null,
        startedAt: '',
        harnessId: 'claude-code',
        harnessName: 'Claude Code',
        processName: 'p',
      },
    ])
    await discoverActiveRun()
    apiGetJson.mockClear()

    await discoverActiveRun()

    expect(apiGetJson).not.toHaveBeenCalled()
  })
})
