<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  CONSENT_OPTIONS,
  allHarnesses,
  selectConsent,
  selectHarness,
  selection,
  type Harness,
} from '../lib/harness'
import { apiPostJson, useRelayClient } from '../lib/relay-client'
import { closePreflight, localPreview, usePreflight, type LaunchPlan } from '../lib/launch'
import { launchEmbedded, markRunConsumed, useExecution } from '../lib/execution'
import { mountDialog, type DialogHandle } from '../lib/focus-trap'
import HarnessConsentPicker from './HarnessConsentPicker.vue'

const props = defineProps<{ workspace: string }>()

const client = useRelayClient()
const exec = useExecution()
const sel = selection()
const preflight = usePreflight()

const overlay = ref<HTMLElement | null>(null)
const promptInput = ref<HTMLInputElement | null>(null)
let dialog: DialogHandle | null = null

const intent = ref('')
const harnessId = ref<string | null>(null)
const plan = ref<LaunchPlan | null>(null)
const planForRevision = ref(-1)
const revision = ref(0)
const loading = ref(false)
const launching = ref(false)
const error = ref('')
const step = ref<'plan' | 'keyboard-choice'>('plan')

const harnesses = computed<Harness[]>(() => allHarnesses())

const activeHarness = computed<Harness | null>(() => {
  if (harnessId.value) {
    const chosen = harnesses.value.find((h) => h.id === harnessId.value && h.state !== 'absent')
    if (chosen) return chosen
  }
  return harnesses.value.find((h) => h.state !== 'absent') ?? null
})

const consentScope = computed(
  () => CONSENT_OPTIONS.find((o) => o.id === sel.consent)?.description ?? '',
)

const planIsCurrent = computed(() => plan.value !== null && planForRevision.value === revision.value)

const stale = computed(() => client.freshness.value === 'stale')

const canConfirm = computed(
  () => !!activeHarness.value && planIsCurrent.value && !launching.value && !stale.value,
)

let previewTimer: number | null = null
let previewToken = 0

function loadPlan(): void {
  if (previewTimer !== null) window.clearTimeout(previewTimer)
  const harness = activeHarness.value
  if (!harness) {
    plan.value = null
    loading.value = false
    return
  }
  const rev = ++revision.value
  const payload = { harness: harness.id, skill: preflight.skill, intent: intent.value }
  loading.value = true
  plan.value = null
  if (!client.hostMode) {
    plan.value = localPreview(harness.id, preflight.skill, intent.value, props.workspace)
    planForRevision.value = rev
    loading.value = false
    error.value = ''
    return
  }
  const token = ++previewToken
  previewTimer = window.setTimeout(async () => {
    try {
      const result = await apiPostJson<LaunchPlan>('/api/launch/preview', payload)
      if (token !== previewToken) return
      if (rev !== revision.value) return
      plan.value = result
      planForRevision.value = rev
      error.value = ''
    } catch {
      if (token !== previewToken) return
      error.value = 'Não foi possível compor o lançamento.'
    } finally {
      if (token === previewToken) loading.value = false
    }
  }, 120)
}

watch(
  () => preflight.open,
  async (open) => {
    if (!open) {
      dialog?.release()
      dialog = null
      return
    }
    intent.value = preflight.intent
    harnessId.value = sel.harnessId ?? harnesses.value.find((h) => h.state !== 'absent')?.id ?? null
    launching.value = false
    error.value = ''
    loading.value = false
    plan.value = null
    step.value = 'plan'
    previewToken += 1
    await nextTick()
    if (overlay.value) {
      dialog = mountDialog(overlay.value, promptInput.value)
    }
    loadPlan()
  },
)

onBeforeUnmount(() => {
  dialog?.release()
  dialog = null
})

watch(harnessId, () => loadPlan())
watch(intent, () => loadPlan())
watch(() => preflight.skill, () => loadPlan())

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') closePreflight()
}

function onHarnessClick(id: string): void {
  selectHarness(props.workspace, id)
  harnessId.value = id
}

function onConsentChange(level: string): void {
  selectConsent(props.workspace, level as typeof sel.consent)
}

async function confirm(): Promise<void> {
  const harness = activeHarness.value
  if (!canConfirm.value || !harness) return
  if (!client.hostMode) {
    closePreflight()
    return
  }
  // primeira execução da sessão: a escolha embutido/externo precisa
  // acontecer antes de qualquer spawn (A-004) — sem isso, escolher externo
  // depois de já ter lançado embutido deixaria essa run embutida viva.
  if (exec.firstRunThisSession) {
    step.value = 'keyboard-choice'
    return
  }
  await launchEmbeddedWithExternalFallback(harness)
}

async function launchEmbeddedWithExternalFallback(harness: Harness): Promise<void> {
  launching.value = true
  error.value = ''
  try {
    await launchEmbedded({
      harness: harness.id,
      harnessName: harness.name,
      skill: preflight.skill,
      intent: intent.value,
      processName: harness.name,
    })
    closePreflight()
  } catch {
    // launchEmbedded rejeitou sem chegar a spawnar nada (ex.: PTY
    // indisponível) — cai no modo externo (comportamento de antes da spec 009).
    try {
      await apiPostJson('/api/launch', {
        harness: harness.id,
        skill: preflight.skill,
        intent: intent.value,
      })
      closePreflight()
    } catch {
      error.value = 'Falha ao lançar o processo.'
      launching.value = false
    }
  }
}

async function chooseEmbedded(): Promise<void> {
  const harness = activeHarness.value
  if (!harness) return
  markRunConsumed()
  launching.value = true
  error.value = ''
  try {
    await launchEmbedded({
      harness: harness.id,
      harnessName: harness.name,
      skill: preflight.skill,
      intent: intent.value,
      processName: harness.name,
    })
    closePreflight()
  } catch {
    launching.value = false
    step.value = 'plan'
    error.value = 'Falha ao lançar embutido.'
  }
}

async function chooseExternal(): Promise<void> {
  const harness = activeHarness.value
  if (!harness) return
  markRunConsumed()
  launching.value = true
  error.value = ''
  try {
    await apiPostJson('/api/launch', {
      harness: harness.id,
      skill: preflight.skill,
      intent: intent.value,
    })
    closePreflight()
  } catch {
    launching.value = false
    step.value = 'plan'
    error.value = 'Falha ao lançar externamente.'
  }
}

const rows = computed(() => {
  if (!plan.value) return []
  return [
    { label: 'bin', value: plan.value.bin },
    ...plan.value.args.map((arg) => ({ label: 'arg', value: arg })),
    { label: 'prompt', value: plan.value.prompt },
    { label: 'cwd', value: plan.value.cwd },
  ]
})
</script>

<template>
  <div
    v-if="preflight.open"
    ref="overlay"
    class="selector-overlay"
    role="dialog"
    aria-modal="true"
    :aria-label="preflight.title"
    tabindex="-1"
    @keydown="onKeydown"
  >
    <div v-if="step === 'plan'" class="selector preflight">
      <header class="selector__header">
        <h2 class="selector__title">{{ preflight.title }}</h2>
        <button class="selector__close" aria-label="Fechar" @click="closePreflight()">×</button>
      </header>

      <section class="selector__section">
        <h3 class="selector__section-title">Processo a lançar</h3>
        <table class="preflight__table">
          <tbody>
            <tr v-for="row in rows" :key="row.label">
              <th class="preflight__cell-label mono">{{ row.label }}</th>
              <td class="preflight__cell mono">{{ row.value }}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <section class="selector__section">
        <h3 class="selector__section-title">Prompt</h3>
        <input
          ref="promptInput"
          v-model="intent"
          class="preflight__prompt mono"
          aria-label="Prompt da intenção"
        />
      </section>

      <HarnessConsentPicker
        :selected-harness-id="harnessId"
        :selected-consent="sel.consent"
        radio-name="preflight-consent"
        @update:selected-harness-id="onHarnessClick"
        @update:selected-consent="onConsentChange"
      />

      <p class="preflight__warning">
        Ao executar, esta janela fecha e o harness assume a partir de agora.
      </p>

      <p v-if="loading" class="preflight__error" role="status">Compondo o plano…</p>
      <p v-if="error" class="preflight__error" role="alert">{{ error }}</p>
      <p v-if="stale" class="preflight__error" role="alert">
        Dado desatualizado — aguardando reconexão antes de poder executar.
      </p>

      <footer class="selector__footer mono">{{ consentScope }}</footer>

      <div class="preflight__actions">
        <button class="button button--secondary" @click="closePreflight()">Cancelar (Esc)</button>
        <button
          class="button button--primary"
          :disabled="!canConfirm"
          @click="confirm"
        >
          ▶ Executar no {{ activeHarness?.name ?? '…' }}
        </button>
      </div>
    </div>

    <div v-else class="selector">
      <header class="selector__header">
        <h2 class="selector__title">Atalhos do terminal</h2>
      </header>
      <p class="kb-warning__text">
        A primeira execução usa atalhos como <code class="mono">Cmd+W</code> e
        <code class="mono">Cmd+T</code>, que o terminal também captura. Se quiser
        evitar o conflito, lance no modo externo.
      </p>
      <p v-if="error" class="preflight__error" role="alert">{{ error }}</p>
      <div class="preflight__actions">
        <button class="button button--secondary" :disabled="launching" @click="chooseExternal">
          Lançar no modo externo
        </button>
        <button class="button button--primary" :disabled="launching" @click="chooseEmbedded">
          Continuar embutido
        </button>
      </div>
    </div>
  </div>
</template>
