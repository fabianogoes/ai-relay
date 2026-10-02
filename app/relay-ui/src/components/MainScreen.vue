<script setup lang="ts">
import { computed } from 'vue'
import type { ChecklistEntry, InconsistentState, OkState, UiPayload } from '../types'
import HandoffCard from './HandoffCard.vue'
import ChecklistList from './ChecklistList.vue'
import EmptyState from './EmptyState.vue'
import RepairScreen from './RepairScreen.vue'
import { allHarnesses, harnessById, selection } from '../lib/harness'
import { openPreflight } from '../lib/launch'
import type { Freshness } from '../lib/observer'

const props = defineProps<{ payload: UiPayload; freshness: Freshness }>()

const stale = computed(() => props.freshness === 'stale')

const ok = computed<OkState | null>(() =>
  props.payload.state.kind === 'ok' ? props.payload.state : null,
)

const inconsistent = computed<InconsistentState | null>(() =>
  props.payload.state.kind === 'inconsistent' ? props.payload.state : null,
)

const handoff = computed(() => ok.value?.handoff ?? null)
const status = computed(() => ok.value?.status)
const execEnabled = computed(() => props.payload.environment.execEnabled)

const sel = selection()

const activeHarnessName = computed(() => {
  const chosen = harnessById(sel.harnessId)
  if (chosen && chosen.state !== 'absent') return chosen.name
  const writer = handoff.value ? harnessById(handoff.value.harness) : null
  if (writer && writer.state !== 'absent') return writer.name
  return allHarnesses().find((h) => h.state !== 'absent')?.name ?? '…'
})

const availableEntries = computed<ChecklistEntry[]>(() => {
  if (!ok.value) return []
  const source = status.value === 'backlog' ? ok.value.backlog : ok.value.todo
  return source.filter((e) => e.available)
})

function comecar(id: string): void {
  openPreflight({
    title: `Iniciar sessão em ${id}`,
    skill: 'relay-session',
    intent: `Iniciar sessão em ${id}`,
  })
}

function entrevista(): void {
  openPreflight({ title: 'Especificar uma ideia', skill: 'relay-spec', intent: 'Especificar uma ideia' })
}
</script>

<template>
  <RepairScreen v-if="inconsistent" :violations="inconsistent.violations" />

  <div v-else-if="ok" class="agora">
    <HandoffCard
      v-if="handoff"
      :handoff="handoff"
      :blocked="status === 'blocked'"
      :exec-enabled="execEnabled"
      :stale="stale"
    />

    <ChecklistList
      v-if="handoff && ok.todo.length > 0"
      class="agora__subtasks"
      title="Subtarefas"
      :entries="[...ok.todo].reverse()"
      :completed="ok.completed"
      :total="ok.total"
    />

    <template v-if="!handoff && status !== 'done'">
      <ChecklistList
        v-if="status === 'ready' && ok.todo.length > 0"
        class="agora__subtasks"
        title="Subtarefas"
        :entries="[...ok.todo].reverse()"
        :completed="ok.completed"
        :total="ok.total"
      />
      <EmptyState v-else-if="status === 'idle'" message="Sem trabalho ativo." />

      <section v-if="availableEntries.length > 0" class="choose">
        <h2 class="panel__title">Escolher</h2>
        <p class="choose__reason">
          Estas tarefas são independentes entre si — a ordem da lista não é fila
          nem prioridade, e <code class="mono">needs</code> é a única dependência
          real. A escolha é sua.
        </p>
        <ul class="choose__list">
          <li v-for="entry in availableEntries" :key="entry.id" class="choose__item">
            <span class="choose__id mono">{{ entry.id }}</span>
            <span class="choose__text">{{ entry.text }}</span>
            <span v-if="entry.spec" class="choose__spec mono">{{ entry.spec }}</span>
            <span v-if="execEnabled && stale" class="choose__stale-note">
              Desatualizado — aguardando reconexão
            </span>
            <button
              v-if="execEnabled"
              class="button button--primary choose__action"
              :disabled="stale"
              @click="comecar(entry.id)"
            >
              ▶ Começar no {{ activeHarnessName }}
            </button>
          </li>
        </ul>

        <section v-if="execEnabled" class="choose__interview">
          <h2 class="choose__interview-title">Iniciar entrevista</h2>
          <p class="choose__interview-text">
            Nenhuma tarefa cobre o que você precisa? Especifique uma ideia nova.
          </p>
          <span v-if="stale" class="choose__stale-note">Desatualizado — aguardando reconexão</span>
          <button class="button button--secondary" :disabled="stale" @click="entrevista()">
            Iniciar entrevista
          </button>
        </section>
      </section>

      <EmptyState v-else-if="status === 'backlog'" message="Nenhuma tarefa disponível no backlog." />
    </template>

    <EmptyState v-if="!handoff && status === 'done'" message="Todo o trabalho concluído." />
  </div>
</template>
