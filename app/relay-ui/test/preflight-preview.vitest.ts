import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'

const apiPostJson = vi.fn()

vi.mock('../src/lib/relay-client.ts', () => ({
  useRelayClient: () => ({ hostMode: true, freshness: ref('current') }),
  apiPostJson,
  apiGetJson: vi.fn(),
  apiGetRaw: vi.fn(),
}))

const { default: PreflightModal } = await import('../src/components/PreflightModal.vue')
const { setHarnesses } = await import('../src/lib/harness.ts')
const { openPreflight, closePreflight } = await import('../src/lib/launch.ts')

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((r) => (resolve = r))
  return { promise, resolve }
}

describe('preview desatualizado nunca substitui o plano mais recente (A-013)', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    apiPostJson.mockReset()
    setHarnesses([{ id: 'claude-code', name: 'Claude Code', version: '1.0.0', state: 'installed' }])
  })

  afterEach(() => {
    closePreflight()
    vi.useRealTimers()
  })

  it('resposta antiga chegando depois da nova não sobrescreve o plano exibido', async () => {
    const first = deferred<{ bin: string; args: string[]; prompt: string; cwd: string }>()
    const second = deferred<{ bin: string; args: string[]; prompt: string; cwd: string }>()
    apiPostJson.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)

    const wrapper = mount(PreflightModal, { props: { workspace: '/work/proj' } })

    openPreflight({ title: 'Iniciar sessão em T-001', skill: 'relay-session', intent: 'Iniciar sessão em T-001' })
    await nextTick()
    await nextTick()

    await vi.advanceTimersByTimeAsync(150)
    expect(apiPostJson).toHaveBeenCalledTimes(1)

    // segunda revisão: usuário edita o prompt antes da primeira resposta chegar
    const input = wrapper.find('.preflight__prompt')
    await input.setValue('Iniciar sessão em T-002')
    await vi.advanceTimersByTimeAsync(150)
    expect(apiPostJson).toHaveBeenCalledTimes(2)

    // a segunda (mais nova) chega primeiro
    second.resolve({ bin: 'claude', args: ['-p'], prompt: 'segunda', cwd: '/plan-2' })
    await nextTick()
    await nextTick()
    expect(wrapper.text()).toContain('/plan-2')

    // a primeira (desatualizada) chega depois — não pode sobrescrever
    first.resolve({ bin: 'claude', args: ['-p'], prompt: 'primeira', cwd: '/plan-1' })
    await nextTick()
    await nextTick()
    expect(wrapper.text()).toContain('/plan-2')
    expect(wrapper.text()).not.toContain('/plan-1')

    wrapper.unmount()
  })
})
