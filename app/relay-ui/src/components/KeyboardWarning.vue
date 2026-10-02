<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useExecution, markRunConsumed } from '../lib/execution'
import { mountDialog, type DialogHandle } from '../lib/focus-trap'

// Só aparece para uma run já viva que esta aba não lançou (reanexo automático
// ao reabrir a aba, A-009): quando esta aba É quem lança, o PreflightModal já
// resolve a escolha embutido/externo antes do spawn e marca a run consumida
// (A-004) — então essa run nunca chega aqui com firstRunThisSession ainda true.
const exec = useExecution()

const overlay = ref<HTMLElement | null>(null)
let dialog: DialogHandle | null = null

const visible = computed(() => exec.firstRunThisSession && exec.activeRunId !== null)

function onAcknowledge(): void {
  markRunConsumed()
}

watch(
  visible,
  async (show) => {
    if (!show) {
      dialog?.release()
      dialog = null
      return
    }
    await nextTick()
    if (overlay.value) dialog = mountDialog(overlay.value, overlay.value)
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  dialog?.release()
  dialog = null
})
</script>

<template>
  <div
    v-if="visible"
    ref="overlay"
    class="selector-overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Conflito de teclado"
    tabindex="-1"
  >
    <div class="selector">
      <header class="selector__header">
        <h2 class="selector__title">Atalhos do terminal</h2>
      </header>
      <p class="kb-warning__text">
        Esta execução usa atalhos como <code class="mono">Cmd+W</code> e
        <code class="mono">Cmd+T</code>, que o terminal também captura.
      </p>
      <div class="preflight__actions">
        <button class="button button--primary" @click="onAcknowledge">Entendi</button>
      </div>
    </div>
  </div>
</template>