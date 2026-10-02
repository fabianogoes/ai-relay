import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'

const apiPostJson = vi.fn(async (path: string) => {
  if (path === '/api/launch/preview') return { bin: 'claude', args: ['-p'], prompt: 'x', cwd: '/work/proj' }
  if (path === '/api/launch/embedded') return { runId: 'run-1', scrollback: '' }
  if (path === '/api/launch') return {}
  throw new Error(`chamada inesperada: ${path}`)
})

vi.mock('../src/lib/relay-client.ts', () => ({
  useRelayClient: () => ({ hostMode: true, freshness: ref('current') }),
  apiPostJson,
  apiGetJson: vi.fn(),
  apiGetRaw: vi.fn(),
}))

const fakeSocket = { connect: vi.fn(), close: vi.fn(), send: vi.fn(), onData: vi.fn(), onExit: vi.fn(), onDisk: vi.fn() }
vi.mock('../src/lib/term-client.ts', () => ({ createRunSocket: () => fakeSocket }))

const { default: PreflightModal } = await import('../src/components/PreflightModal.vue')
const { setHarnesses } = await import('../src/lib/harness.ts')
const { openPreflight, closePreflight } = await import('../src/lib/launch.ts')
const { useExecution } = await import('../src/lib/execution.ts')

describe('escolher "modo externo" nunca cria nem deixa viva uma execução embutida (A-004)', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    closePreflight()
    vi.useRealTimers()
  })

  it('chama só /api/launch, nunca /api/launch/embedded, e activeRunId continua nulo', async () => {
    setHarnesses([{ id: 'claude-code', name: 'Claude Code', version: '1.0.0', state: 'installed' }])
    const wrapper = mount(PreflightModal, { props: { workspace: '/work/proj' } })
    openPreflight({ title: 'Iniciar sessão em T-001', skill: 'relay-session', intent: 'x' })
    await nextTick()
    await nextTick()
    await vi.advanceTimersByTimeAsync(150)
    await wrapper.find('.preflight__actions .button--primary').trigger('click')
    await nextTick()

    await wrapper.find('.preflight__actions .button--secondary').trigger('click')
    await nextTick()
    await nextTick()

    expect(apiPostJson).toHaveBeenCalledWith('/api/launch', expect.anything())
    expect(apiPostJson).not.toHaveBeenCalledWith('/api/launch/embedded', expect.anything())
    expect(useExecution().activeRunId).toBeNull()
    wrapper.unmount()
  })
})
