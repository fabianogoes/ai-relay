import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import type { UiPayload } from '../src/types.ts'

const openPreflight = vi.fn()

vi.mock('../src/lib/launch.ts', () => ({
  openPreflight,
}))

const { default: MainScreen } = await import('../src/components/MainScreen.vue')
const { default: HandoffCard } = await import('../src/components/HandoffCard.vue')

function payloadReadyWithOneAvailable(): UiPayload {
  return {
    state: {
      kind: 'ok',
      status: 'ready',
      handoff: null,
      activeBacklogId: 'B-028',
      todo: [{ id: 'T-001', text: 'Tarefa disponível', marker: ' ', needs: [], available: true }],
      backlog: [],
      completed: 0,
      total: 1,
    },
    environment: { workspace: '/work/proj', execEnabled: true },
  }
}

describe('nenhuma ação de execução usa estado desatualizado (A-008)', () => {
  beforeEach(() => {
    openPreflight.mockClear()
  })

  it('tela Escolher: "Começar" e "Iniciar entrevista" desabilitam quando freshness é stale', () => {
    const wrapper = mount(MainScreen, {
      props: { payload: payloadReadyWithOneAvailable(), freshness: 'stale' },
    })

    const comecar = wrapper.find('.choose__action')
    expect(comecar.attributes('disabled')).toBeDefined()

    const interview = wrapper.findAll('button').find((b) => b.text().includes('Iniciar entrevista'))
    expect(interview?.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Desatualizado')
  })

  it('tela Escolher: "Começar" habilita quando freshness é current', () => {
    const wrapper = mount(MainScreen, {
      props: { payload: payloadReadyWithOneAvailable(), freshness: 'current' },
    })

    const comecar = wrapper.find('.choose__action')
    expect(comecar.attributes('disabled')).toBeUndefined()
  })

  it('HandoffCard: "Retomar" desabilita quando freshness é stale', () => {
    const wrapper = mount(HandoffCard, {
      props: {
        handoff: {
          backlogId: 'B-028',
          todoId: 'T-001',
          spec: '.specs/x.md',
          harness: 'claude-code',
          updated: '2026-09-16T10:00:00Z',
          objective: 'Objetivo curto.',
          nextStep: 'Próximo passo.',
          context: 'Contexto.',
        },
        blocked: false,
        execEnabled: true,
        stale: true,
      },
    })

    const resume = wrapper.find('.handoff-card__footer .button--primary')
    expect(resume.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Desatualizado')
  })
})
