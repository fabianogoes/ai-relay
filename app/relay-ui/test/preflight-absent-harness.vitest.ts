import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref } from 'vue'

vi.mock('../src/lib/relay-client.ts', () => ({
  useRelayClient: () => ({ hostMode: false, freshness: ref('current') }),
  apiPostJson: vi.fn(),
  apiGetJson: vi.fn(),
  apiGetRaw: vi.fn(),
}))

const { default: PreflightModal } = await import('../src/components/PreflightModal.vue')
const { setHarnesses } = await import('../src/lib/harness.ts')
const { openPreflight, closePreflight } = await import('../src/lib/launch.ts')

describe('nenhum harness ausente pode ser confirmado para execução (A-013)', () => {
  afterEach(() => {
    closePreflight()
  })

  it('só harness ausente disponível: botão de executar fica desabilitado', async () => {
    setHarnesses([{ id: 'opencode', name: 'OpenCode', version: '', state: 'absent' }])

    const wrapper = mount(PreflightModal, { props: { workspace: '/work/proj' } })
    openPreflight({ title: 'Iniciar sessão em T-001', skill: 'relay-session', intent: 'Iniciar sessão em T-001' })
    await nextTick()
    await nextTick()

    const confirm = wrapper.find('.preflight__actions .button--primary')
    expect(confirm.attributes('disabled')).toBeDefined()

    wrapper.unmount()
  })

  it('com um harness instalado disponível, o botão de executar habilita', async () => {
    setHarnesses([{ id: 'claude-code', name: 'Claude Code', version: '1.0.0', state: 'installed' }])

    const wrapper = mount(PreflightModal, { props: { workspace: '/work/proj' } })
    openPreflight({ title: 'Iniciar sessão em T-001', skill: 'relay-session', intent: 'Iniciar sessão em T-001' })
    await nextTick()
    await nextTick()

    const confirm = wrapper.find('.preflight__actions .button--primary')
    expect(confirm.attributes('disabled')).toBeUndefined()

    wrapper.unmount()
  })
})
