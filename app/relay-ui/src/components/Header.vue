<script setup lang="ts">
import { computed } from 'vue'
import type { UiPayload } from '../types'
import StatusPill from './StatusPill.vue'
import FreshnessStatus from './FreshnessStatus.vue'
import HarnessIcon from './HarnessIcon.vue'
import type { Freshness } from '../lib/observer'
import { HARNESS_FIXTURE, allHarnesses, harnessById, openSelector, selection } from '../lib/harness'

const props = defineProps<{
  payload: UiPayload
  view: 'agora' | 'trabalho'
  freshness: Freshness
}>()
const emit = defineEmits<{ (e: 'update:view', view: 'agora' | 'trabalho'): void }>()

const status = computed(() =>
  props.payload.state.kind === 'ok' ? props.payload.state.status : 'inconsistent',
)

const sel = selection()

const activeHarness = computed(() => {
  const chosen = harnessById(sel.harnessId)
  if (chosen) return chosen
  const handoff = props.payload.state.kind === 'ok' ? props.payload.state.handoff : null
  const writer = handoff ? harnessById(handoff.harness) : null
  if (writer && writer.state !== 'absent') return writer
  return allHarnesses()[0] ?? HARNESS_FIXTURE[0]
})

const workspaceName = computed(() => {
  const path = props.payload.environment.workspace
  const base = path.split('/').filter(Boolean).pop()
  return base ?? path
})
</script>

<template>
  <header class="header">
    <div class="header__brand">
      <h1 class="header__logo">Relay</h1>
      <span class="header__workspace mono">
        {{ workspaceName }}<span class="header__workspace-path">{{ payload.environment.workspace }}</span>
      </span>
    </div>
    <nav class="header__tabs" role="tablist" aria-label="Visões">
      <button
        id="tab-agora"
        class="header__tab"
        role="tab"
        :aria-selected="view === 'agora'"
        aria-controls="view-agora"
        :class="{ 'is-active': view === 'agora' }"
        @click="emit('update:view', 'agora')"
      >
        Agora
      </button>
      <button
        id="tab-trabalho"
        class="header__tab"
        role="tab"
        :aria-selected="view === 'trabalho'"
        aria-controls="view-trabalho"
        :class="{ 'is-active': view === 'trabalho' }"
        @click="emit('update:view', 'trabalho')"
      >
        Trabalho
      </button>
    </nav>
    <span class="header__spacer"></span>
    <button v-if="payload.environment.execEnabled" class="harness-badge" @click="openSelector()">
      <span class="harness-avatar" aria-hidden="true">
        <HarnessIcon :id="activeHarness.id" />
      </span>
      <span class="harness-badge__name">{{ activeHarness.name }}</span>
    </button>
    <FreshnessStatus :freshness="freshness" />
    <StatusPill :status="status" />
  </header>
</template>
