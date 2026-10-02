import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import HarnessSelector from '../src/components/HarnessSelector.vue'
import { closeSelector, initHarnessSelection, openSelector, selectConsent } from '../src/lib/harness.ts'

const workspace = '/work/consent-test'

describe('downgrade de consentimento remove o valor persistido (A-013)', () => {
  beforeEach(() => {
    localStorage.clear()
    closeSelector()
    initHarnessSelection(workspace)
  })

  it('clicar em "Sempre neste workspace" grava, clicar em "Só esta execução" limpa', async () => {
    const wrapper = mount(HarnessSelector, { props: { workspace } })

    openSelector()
    await nextTick()
    await nextTick()

    const localRadio = wrapper.find('input[value="local"]')
    await localRadio.setValue()
    expect(localStorage.getItem(`relay.harness:${workspace}`)).not.toBeNull()

    const noneRadio = wrapper.find('input[value="none"]')
    await noneRadio.setValue()
    expect(localStorage.getItem(`relay.harness:${workspace}`)).toBeNull()

    wrapper.unmount()
  })

  it('reabrir a pagina depois do downgrade nao ressuscita a escolha antiga', async () => {
    selectConsent(workspace, 'local')
    expect(localStorage.getItem(`relay.harness:${workspace}`)).not.toBeNull()

    selectConsent(workspace, 'none')
    expect(localStorage.getItem(`relay.harness:${workspace}`)).toBeNull()

    initHarnessSelection(workspace)
    const { selection } = await import('../src/lib/harness.ts')
    expect(selection().consent).toBe('none')
  })
})
