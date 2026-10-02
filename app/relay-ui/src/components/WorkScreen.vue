<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import type { ChecklistEntry, UiPayload } from '../types'
import EmptyState from './EmptyState.vue'
import BacklogTasksModal, { type TaskRow } from './BacklogTasksModal.vue'
import { apiGetJson, useRelayClient } from '../lib/relay-client'
import { openPreflight } from '../lib/launch'
import { createLatestRequest, reconcileSpecId } from '../lib/work'
import { formatCalendarDate } from '../lib/presentation'
import type { Freshness } from '../lib/observer'

interface SpecSummary {
  id: string
  title: string
  taskCount: number
}

interface ChangelogEntry {
  date: string
  todoId: string
  title: string
  backlogId: string
  spec: string
  evidence: string
  criteria: string[]
}

const props = defineProps<{ payload: UiPayload; freshness: Freshness }>()

const client = useRelayClient()
const specs = ref<SpecSummary[]>([])
const changelogEntries = ref<ChangelogEntry[]>([])
const selectedSpecId = ref<string | null>(null)
const selectedBacklogId = ref<string | null>(null)
const listRequests = createLatestRequest()

const execEnabled = computed(() => props.payload.environment.execEnabled)
const stale = computed(() => props.freshness === 'stale')

async function reload(): Promise<void> {
  if (!client.hostMode) return
  const request = listRequests.start()
  let nextSpecs: SpecSummary[] = []
  try {
    nextSpecs = await apiGetJson<SpecSummary[]>('/api/specs')
  } catch {
    nextSpecs = []
  }
  if (!listRequests.isCurrent(request.revision)) return
  // mais recente no topo; a ordem textual do disco não carrega prioridade
  specs.value = nextSpecs.reverse()
  selectedSpecId.value = reconcileSpecId(specs.value, selectedSpecId.value)
  let nextEntries: ChangelogEntry[] = []
  try {
    nextEntries = await apiGetJson<ChangelogEntry[]>('/api/changelog/entries')
  } catch {
    nextEntries = []
  }
  if (!listRequests.isCurrent(request.revision)) return
  changelogEntries.value = nextEntries
}

onMounted(reload)
watch(() => props.payload, reload)

const selectedSpec = computed(
  () => specs.value.find((s) => s.id === selectedSpecId.value) ?? null,
)

const backlog = computed(() => {
  if (props.payload.state.kind !== 'ok') return []
  const specPath = selectedSpec.value ? `.specs/${selectedSpec.value.id}` : ''
  return props.payload.state.backlog.filter((e) => e.spec === specPath).reverse()
})

watch(
  backlog,
  (list) => {
    selectedBacklogId.value = reconcileSpecId(list, selectedBacklogId.value)
  },
  { immediate: true },
)

const filteredChangelog = computed(() =>
  changelogEntries.value.filter((e) => e.backlogId === selectedBacklogId.value),
)

function selectSpec(id: string): void {
  selectedSpecId.value = id
}

function selectBacklog(id: string): void {
  selectedBacklogId.value = id
}

const BADGE: Record<string, { tone: string; label: string }> = {
  x: { tone: 'done', label: 'Feito' },
  '•': { tone: 'in_progress', label: 'Em curso' },
  '!': { tone: 'blocked', label: 'Bloqueado' },
}

function badgeTone(entry: ChecklistEntry): string {
  return BADGE[entry.marker]?.tone ?? (entry.available ? 'ready' : 'idle')
}

function badgeLabel(entry: ChecklistEntry): string {
  return BADGE[entry.marker]?.label ?? (entry.available ? 'Disponível' : 'Pendente')
}

function taskCountLabel(count: number): string {
  return count === 1 ? '1 tarefa' : `${count} tarefas`
}

const tasksBacklogId = ref<string | null>(null)

const TASK_LABELS: Record<string, { label: string; tone: TaskRow['tone'] }> = {
  x: { label: 'Feito', tone: 'done' },
  '•': { label: 'Em execução', tone: 'current' },
  '!': { label: 'Bloqueado', tone: 'blocked' },
}

const tasksBacklogEntry = computed(
  () => backlog.value.find((e) => e.id === tasksBacklogId.value) ?? null,
)

const tasksAreLive = computed(
  () =>
    props.payload.state.kind === 'ok' &&
    props.payload.state.activeBacklogId === tasksBacklogId.value &&
    props.payload.state.todo.length > 0,
)

const tasksRows = computed<TaskRow[]>(() => {
  const id = tasksBacklogId.value
  if (id === null) return []
  const state = props.payload.state
  if (tasksAreLive.value && state.kind === 'ok') {
    return [...state.todo].reverse().map((entry) => {
      const known = TASK_LABELS[entry.marker]
      return {
        id: entry.id,
        text: entry.text,
        marker: entry.marker === ' ' ? '○' : entry.marker,
        label: known?.label ?? 'Pendente',
        tone: known?.tone ?? (entry.available ? 'available' : ''),
      }
    })
  }
  return changelogEntries.value
    .filter((e) => e.backlogId === id)
    .map((e) => ({ id: e.todoId, text: e.title, marker: 'x', label: 'Feito', tone: 'done' }))
})

const tasksCountLabel = computed(() => {
  const state = props.payload.state
  if (tasksAreLive.value && state.kind === 'ok') {
    return `${state.completed} de ${state.total} concluídas`
  }
  const n = tasksRows.value.length
  return n === 1 ? '1 executada' : `${n} executadas`
})

function openTasks(id: string): void {
  selectBacklog(id)
  tasksBacklogId.value = id
}

function closeTasks(): void {
  tasksBacklogId.value = null
}

function recordCountLabel(count: number): string {
  return count === 1 ? '1 registro' : `${count} registros`
}

function onNewSpec(): void {
  openPreflight({ title: 'Especificar uma ideia', skill: 'relay-spec', intent: 'Especificar uma ideia' })
}

function retomar(): void {
  openPreflight({ title: 'Retomar sessão', skill: 'relay-session', intent: 'Retomar sessão' })
}

function comecar(id: string): void {
  openPreflight({
    title: `Iniciar sessão em ${id}`,
    skill: 'relay-session',
    intent: `Iniciar sessão em ${id}`,
  })
}
</script>

<template>
  <section class="work" aria-label="Segunda visão">
    <div class="work__column">
      <header class="work__column-header">
        <h2 class="work__column-title mono">Specs</h2>
        <button
          v-if="execEnabled"
          class="button button--secondary work__new-spec"
          :disabled="stale"
          @click="onNewSpec"
        >
          + nova spec
        </button>
      </header>
      <ul class="work__specs">
        <li v-for="spec in specs" :key="spec.id">
          <button
            class="work__spec"
            :class="{ 'is-selected': spec.id === selectedSpecId }"
            @click="selectSpec(spec.id)"
          >
            <span class="work__spec-head">
              <span class="work__spec-id mono">{{ spec.id.replace(/\.md$/, '') }}</span>
              <span class="work__spec-count">{{ taskCountLabel(spec.taskCount) }}</span>
            </span>
            <span class="work__spec-title">{{ spec.title }}</span>
            <span class="work__spec-path mono">.specs/{{ spec.id }}</span>
          </button>
        </li>
      </ul>
      <EmptyState v-if="specs.length === 0" message="Nenhuma spec encontrada." />
    </div>

    <div class="work__column">
      <header class="work__column-header">
        <h2 class="work__column-title work__column-title--backlog mono">Backlog da spec</h2>
        <span v-if="selectedSpec" class="work__column-meta mono">
          spec {{ selectedSpec.id.replace(/\.md$/, '') }}
        </span>
      </header>
      <ul v-if="backlog.length > 0" class="backlog-cards">
        <li
          v-for="entry in backlog"
          :key="entry.id"
          class="backlog-card"
          :class="[
            `backlog-card--${badgeTone(entry)}`,
            { 'is-selected': entry.id === selectedBacklogId },
          ]"
        >
          <button class="backlog-card__select" @click="selectBacklog(entry.id)">
            <span class="backlog-card__header">
              <span class="backlog-card__id mono">{{ entry.id }}</span>
              <span class="backlog-card__status mono">{{ badgeLabel(entry) }}</span>
            </span>
            <span class="backlog-card__text">{{ entry.text }}</span>
          </button>
          <footer class="backlog-card__footer">
            <button
              class="button button--secondary backlog-card__action"
              @click="openTasks(entry.id)"
            >
              Tarefas
            </button>
            <button
              v-if="execEnabled && entry.marker === '•'"
              class="button button--primary backlog-card__action"
              :disabled="stale"
              @click="retomar()"
            >
              ▶ Retomar
            </button>
            <button
              v-else-if="execEnabled && entry.marker === ' ' && entry.available"
              class="button button--primary backlog-card__action"
              :disabled="stale"
              @click="comecar(entry.id)"
            >
              ▶ Começar
            </button>
          </footer>
        </li>
      </ul>
      <EmptyState v-else message="Esta spec ainda não tem entradas de backlog." />
    </div>

    <div class="work__column">
      <header class="work__column-header">
        <h2 class="work__column-title work__column-title--changelog mono">Changelog</h2>
        <span v-if="selectedBacklogId" class="work__column-meta mono">
          {{ recordCountLabel(filteredChangelog.length) }}
        </span>
      </header>
      <ul v-if="filteredChangelog.length > 0" class="changelog-cards">
        <li v-for="(entry, i) in filteredChangelog" :key="i" class="changelog-card">
          <header class="changelog-card__meta mono">
            {{ formatCalendarDate(entry.date) }} · {{ entry.todoId }} · {{ entry.backlogId }}
          </header>
          <p class="changelog-card__title">{{ entry.title }}</p>
          <p class="changelog-card__evidence mono">evidência: {{ entry.evidence }}</p>
        </li>
      </ul>
      <EmptyState
        v-else-if="!selectedBacklogId"
        message="Selecione um card de backlog para ver seu changelog."
      />
      <EmptyState v-else message="Nenhum registro de changelog para este backlog ainda." />
    </div>

    <BacklogTasksModal
      v-if="tasksBacklogId"
      :backlog-id="tasksBacklogId"
      :backlog-text="tasksBacklogEntry?.text ?? ''"
      :rows="tasksRows"
      :count-label="tasksCountLabel"
      @close="closeTasks"
    />
  </section>
</template>
