<script setup lang="ts">
import { computed } from 'vue'
import type { Handoff } from '../types'
import { formatAbsolute, formatRelative } from '../lib/presentation'
import { allHarnesses, harnessById, selection, openSelector } from '../lib/harness'
import { openPreflight } from '../lib/launch'
import HarnessIcon from './HarnessIcon.vue'

const props = defineProps<{
  handoff: Handoff
  blocked: boolean
  execEnabled: boolean
  stale: boolean
}>()

const sel = selection()

const writer = computed(() => harnessById(props.handoff.harness))

const activeHarness = computed(() => {
  const chosen = harnessById(sel.harnessId)
  if (chosen && chosen.state !== 'absent') return chosen
  if (writer.value && writer.value.state !== 'absent') return writer.value
  return allHarnesses().find((h) => h.state !== 'absent') ?? writer.value ?? { id: props.handoff.harness, name: props.handoff.harness }
})

function resume(): void {
  openPreflight({ title: 'Retomar sessão', skill: 'relay-session', intent: 'Retomar sessão' })
}
</script>

<template>
  <article class="handoff-card" :class="blocked ? 'is-blocked' : 'is-in-progress'">
    <div class="handoff-card__provenance">
      <span class="harness-avatar" aria-hidden="true">
        <HarnessIcon :id="handoff.harness" />
      </span>
      <span class="handoff-card__writer">
        Escrito no {{ writer?.name ?? handoff.harness }} ·
        {{ formatRelative(handoff.updated) }}
      </span>
    </div>
    <div class="handoff-card__meta mono">
      {{ handoff.backlogId }} / {{ handoff.todoId }} ·
      {{ formatAbsolute(handoff.updated) }}
    </div>
    <h2 class="handoff-card__objective">
      <span class="handoff-card__objective-label mono">
        OBJETIVO · {{ handoff.backlogId }} / {{ handoff.todoId }}
      </span>
      <span class="handoff-card__objective-text">{{ handoff.objective }}</span>
    </h2>
    <div class="handoff-card__columns">
      <section class="handoff-card__column">
        <h3 class="handoff-card__column-title handoff-card__column-title--accent">
          Próximo passo
        </h3>
        <p class="handoff-card__next-step">{{ handoff.nextStep }}</p>
      </section>
      <section class="handoff-card__column">
        <h3 class="handoff-card__column-title">Contexto deixado</h3>
        <p class="handoff-card__context">{{ handoff.context }}</p>
      </section>
    </div>
    <footer class="handoff-card__footer">
      <span v-if="execEnabled && stale" class="handoff-card__stale-note">
        Desatualizado — aguardando reconexão
      </span>
      <button v-if="execEnabled" class="button button--primary" :disabled="stale" @click="resume">
        ▶ Retomar {{ handoff.todoId }} no {{ activeHarness.name }}
      </button>
      <button v-if="execEnabled" class="button button--secondary" @click="openSelector()">
        Trocar harness
      </button>
      <span class="handoff-card__spec mono">{{ handoff.spec }}</span>
    </footer>
  </article>
</template>
