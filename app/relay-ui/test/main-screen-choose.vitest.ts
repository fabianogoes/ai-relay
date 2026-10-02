import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import type { UiPayload } from '../src/types.ts'

const openPreflight = vi.fn()

vi.mock('../src/lib/launch.ts', () => ({
  openPreflight,
}))

// importado depois do mock, como o vitest exige para hoisting de vi.mock
const { default: MainScreen } = await import('../src/components/MainScreen.vue')

function payloadWithTwoAvailable(): UiPayload {
  return {
    state: {
      kind: 'ok',
      status: 'ready',
      handoff: null,
      activeBacklogId: 'B-028',
      todo: [
        { id: 'T-001', text: 'Primeira tarefa disponível', marker: ' ', needs: [], available: true },
        { id: 'T-002', text: 'Segunda tarefa disponível', marker: ' ', needs: [], available: true },
      ],
      backlog: [],
      completed: 0,
      total: 2,
    },
    environment: { workspace: '/work/proj', execEnabled: true },
  }
}

describe('escolher uma tarefa envia o ID correspondente, nunca sempre o mesmo (A-013)', () => {
  beforeEach(() => {
    openPreflight.mockClear()
  })

  it('clicar em Começar de T-001 e depois de T-002 envia dois IDs distintos', async () => {
    const wrapper = mount(MainScreen, {
      props: { payload: payloadWithTwoAvailable(), freshness: 'current' },
    })

    const buttons = wrapper.findAll('.choose__action')
    expect(buttons).toHaveLength(2)

    await buttons[0].trigger('click')
    await buttons[1].trigger('click')

    expect(openPreflight).toHaveBeenCalledTimes(2)
    const firstCallIntent = openPreflight.mock.calls[0][0].intent
    const secondCallIntent = openPreflight.mock.calls[1][0].intent

    expect(firstCallIntent).toContain('T-001')
    expect(secondCallIntent).toContain('T-002')
    expect(firstCallIntent).not.toBe(secondCallIntent)
  })
})
