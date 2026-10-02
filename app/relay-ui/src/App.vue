<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import Header from './components/Header.vue'
import MainScreen from './components/MainScreen.vue'
import WorkScreen from './components/WorkScreen.vue'
import HarnessSelector from './components/HarnessSelector.vue'
import PreflightModal from './components/PreflightModal.vue'
import ExecutionMode from './components/ExecutionMode.vue'
import BackgroundStrip from './components/BackgroundStrip.vue'
import KeyboardWarning from './components/KeyboardWarning.vue'
import { useRelayClient, apiGetJson } from './lib/relay-client'
import { discoverActiveRun, useExecution } from './lib/execution'
import { fixtures, fixtureNames } from './fixtures'
import { initHarnessSelection, setHarnesses, type Harness } from './lib/harness'

const client = useRelayClient()
const execution = useExecution()
const hasFixturesParam = new URLSearchParams(window.location.search).has('fixtures')
const fixtureMode = ref(!client.hostMode || hasFixturesParam)

const active = ref('in_progress')

const VIEW_KEY = 'relay.view'

function storedView(): 'agora' | 'trabalho' {
  try {
    return sessionStorage.getItem(VIEW_KEY) === 'trabalho' ? 'trabalho' : 'agora'
  } catch {
    return 'agora'
  }
}

const view = ref<'agora' | 'trabalho'>(storedView())

watch(view, (current) => {
  try {
    sessionStorage.setItem(VIEW_KEY, current)
  } catch {
    // sem storage disponível: a aba volta ao padrão no próximo reload
  }
})

const inExecution = computed(() => execution.activeRunId !== null && !execution.detached)
const detached = computed(() => execution.activeRunId !== null && execution.detached)

const payload = computed(() => {
  if (fixtureMode.value) return fixtures[active.value]
  return client.payload.value
})

const workspace = computed(() => payload.value?.environment.workspace ?? '')
const execEnabled = computed(() => payload.value?.environment.execEnabled ?? false)
const freshness = computed(() => (fixtureMode.value ? 'current' : client.freshness.value))

watch(
  workspace,
  (w) => {
    if (w) initHarnessSelection(w)
  },
  { immediate: true },
)

onMounted(() => {
  if (client.hostMode && !hasFixturesParam) client.connect()
})

let discoveryAttempted = false

watch(
  execEnabled,
  (enabled) => {
    if (enabled && client.hostMode) {
      apiGetJson<Harness[]>('/api/harnesses')
        .then((list) => setHarnesses(list))
        .catch(() => setHarnesses([]))
      // reabrir a aba durante uma run viva reanexa a ela; só tenta uma vez
      // por sessão, não a cada payload (A-009).
      if (!discoveryAttempted) {
        discoveryAttempted = true
        void discoverActiveRun()
      }
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="app">
    <nav v-if="fixtureMode && !inExecution" class="fixture-switcher" aria-label="Fixtures">
      <button
        v-for="name in fixtureNames"
        :key="name"
        class="fixture-switcher__button"
        :class="{ 'is-active': name === active }"
        @click="active = name"
      >
        {{ name }}
      </button>
    </nav>

    <ExecutionMode v-if="inExecution" />

    <template v-else>
      <div v-if="!payload" class="empty-state">Conectando ao relay-host…</div>

      <template v-else>
        <main class="app__body">
          <Header :payload="payload" :view="view" :freshness="freshness" @update:view="view = $event" />
          <BackgroundStrip v-if="detached" />
          <MainScreen
            v-if="view === 'agora'"
            :payload="payload"
            :freshness="freshness"
            id="view-agora"
            role="tabpanel"
            aria-labelledby="tab-agora"
          />
          <WorkScreen
            v-else
            :payload="payload"
            :freshness="freshness"
            id="view-trabalho"
            role="tabpanel"
            aria-labelledby="tab-trabalho"
          />
        </main>
        <template v-if="execEnabled">
          <HarnessSelector :workspace="payload.environment.workspace" />
          <PreflightModal :workspace="payload.environment.workspace" />
        </template>
      </template>
    </template>

    <KeyboardWarning v-if="execEnabled" />
  </div>
</template>
