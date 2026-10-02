<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  CONSENT_OPTIONS,
  closeSelector,
  selectConsent,
  selectHarness,
  selection,
} from '../lib/harness'
import { mountDialog, type DialogHandle } from '../lib/focus-trap'
import HarnessConsentPicker from './HarnessConsentPicker.vue'

const props = defineProps<{ workspace: string }>()

const sel = selection()

const overlay = ref<HTMLElement | null>(null)
let dialog: DialogHandle | null = null

const selectedConsent = computed(
  () => CONSENT_OPTIONS.find((o) => o.id === sel.consent) ?? CONSENT_OPTIONS[0],
)

function onBackdropClick(): void {
  closeSelector()
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') closeSelector()
}

watch(
  () => sel.open,
  async (open) => {
    if (!open) {
      dialog?.release()
      dialog = null
      return
    }
    await nextTick()
    if (overlay.value) dialog = mountDialog(overlay.value, overlay.value)
  },
)

onBeforeUnmount(() => {
  dialog?.release()
  dialog = null
})
</script>

<template>
  <div
    v-if="sel.open"
    ref="overlay"
    class="selector-overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Seletor de harness e consentimento"
    tabindex="-1"
    @click.self="onBackdropClick"
    @keydown="onKeydown"
  >
    <div class="selector">
      <header class="selector__header">
        <h2 class="selector__title">Harness e consentimento</h2>
        <button class="selector__close" aria-label="Fechar" @click="closeSelector()">
          ×
        </button>
      </header>

      <HarnessConsentPicker
        :selected-harness-id="sel.harnessId"
        :selected-consent="sel.consent"
        radio-name="standalone-consent"
        @update:selected-harness-id="selectHarness(props.workspace, $event)"
        @update:selected-consent="selectConsent(props.workspace, $event)"
      />

      <footer class="selector__footer mono">
        {{ selectedConsent.description }}
      </footer>
    </div>
  </div>
</template>