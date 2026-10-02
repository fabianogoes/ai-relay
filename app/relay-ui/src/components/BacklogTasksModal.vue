<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { mountDialog, type DialogHandle } from '../lib/focus-trap'

export interface TaskRow {
  id: string
  text: string
  marker: string
  label: string
  tone: 'done' | 'current' | 'blocked' | 'available' | ''
}

const props = defineProps<{
  backlogId: string
  backlogText: string
  rows: TaskRow[]
  countLabel: string
}>()

const emit = defineEmits<{ close: [] }>()

const overlay = ref<HTMLElement | null>(null)
let dialog: DialogHandle | null = null

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') emit('close')
}

watch(
  () => props.backlogId,
  async () => {
    dialog?.release()
    await nextTick()
    if (overlay.value) dialog = mountDialog(overlay.value, overlay.value)
  },
  { immediate: true },
)

onBeforeUnmount(() => dialog?.release())
</script>

<template>
  <div
    ref="overlay"
    class="selector-overlay"
    role="dialog"
    aria-modal="true"
    :aria-label="`Tarefas de ${backlogId}`"
    tabindex="-1"
    @keydown="onKeydown"
  >
    <div class="selector tasks-modal">
      <header class="selector__header">
        <h2 class="selector__title">Tarefas de {{ backlogId }}</h2>
        <button class="selector__close" aria-label="Fechar" @click="emit('close')">×</button>
      </header>

      <p class="tasks-modal__subject">{{ backlogText }}</p>

      <div class="checklist__header">
        <h3 class="tasks-modal__list-title">Subtarefas</h3>
        <span class="checklist__count mono">{{ countLabel }}</span>
      </div>

      <ul v-if="rows.length > 0" class="checklist">
        <li v-for="row in rows" :key="row.id" class="checklist__item" :class="row.tone && `is-${row.tone}`">
          <span class="checklist__marker mono" aria-hidden="true">{{ row.marker }}</span>
          <span class="checklist__label">{{ row.label }}</span>
          <span class="checklist__id mono">{{ row.id }}</span>
          <span class="checklist__text">{{ row.text }}</span>
        </li>
      </ul>
      <p v-else class="tasks-modal__empty">
        Nenhuma subtarefa registrada para este backlog ainda.
      </p>
    </div>
  </div>
</template>
