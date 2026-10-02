import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FreshnessStatus from '../src/components/FreshnessStatus.vue'

describe('desconexão fica visível na UI, não só no dado (A-013)', () => {
  it('freshness "stale" renderiza o rótulo Desatualizado com aria-live', () => {
    const wrapper = mount(FreshnessStatus, { props: { freshness: 'stale' } })

    expect(wrapper.text()).toContain('Desatualizado')
    expect(wrapper.attributes('role')).toBe('status')
    expect(wrapper.attributes('aria-live')).toBe('polite')
    expect(wrapper.classes()).toContain('freshness-status--stale')
  })

  it('freshness "current" nunca mostra o rótulo de desatualizado', () => {
    const wrapper = mount(FreshnessStatus, { props: { freshness: 'current' } })

    expect(wrapper.text()).not.toContain('Desatualizado')
    expect(wrapper.text()).toContain('Atualizado')
  })
})
