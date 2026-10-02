<script setup lang="ts">
import { computed, ref } from 'vue'
import StatusPill from './StatusPill.vue'
import Terminal from './Terminal.vue'
import DiskLog from './DiskLog.vue'
import { closeRun, detachRun, terminateRun, useExecution, useDisk } from '../lib/execution'
import HarnessIcon from './HarnessIcon.vue'

const exec = useExecution()
const disk = useDisk()

const confirming = ref(false)

const pillStatus = computed(() => (exec.status === 'exited' ? 'done' : 'in_progress'))

function onDetach(): void {
  detachRun()
}

function onTerminate(): void {
  if (!confirming.value) {
    confirming.value = true
    return
  }
  terminateRun()
  confirming.value = false
}

function onClose(): void {
  closeRun()
}
</script>

<template>
  <main class="exec-mode">
    <header class="exec-bar">
      <div class="exec-bar__identity">
        <span class="harness-avatar" aria-hidden="true">
          <HarnessIcon :id="exec.harnessId" />
        </span>
        <h1 class="exec-bar__name">{{ exec.processName }}</h1>
        <span v-if="exec.reattached" class="exec-bar__reattach">
          RECONECTADO À EXECUÇÃO VIVA
        </span>
      </div>
      <StatusPill :status="pillStatus" />
      <div class="exec-bar__controls">
        <template v-if="exec.status === 'running'">
          <button class="button button--secondary" @click="onDetach">
            Deixar em segundo plano
          </button>
          <button
            class="button button--danger"
            :aria-label="confirming ? `Confirmar encerrar ${exec.processName}` : `Encerrar processo ${exec.processName}`"
            @click="onTerminate"
          >
            {{ confirming ? `Descartar ${exec.processName}?` : 'Encerrar processo' }}
          </button>
        </template>
        <button v-else class="button button--secondary" @click="onClose">Fechar</button>
      </div>
    </header>

    <div class="exec-body">
      <Terminal v-if="exec.activeRunId" :run-id="exec.activeRunId" />
      <DiskLog :entries="disk" />
    </div>
  </main>
</template>
