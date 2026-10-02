<script setup lang="ts">
import { computed } from 'vue'
import type { DiskEntry } from '../lib/execution'
import { formatRelative } from '../lib/presentation'

const props = defineProps<{ entries: ReadonlyArray<DiskEntry> }>()

const reversed = computed(() => [...props.entries].reverse())

function timeShort(at: string): string {
  return formatRelative(at)
}
</script>

<template>
  <aside class="disk-log" aria-label="Gravado em disco">
    <header class="disk-log__header">
      <h2 class="disk-log__title">Gravado em disco</h2>
      <span class="disk-log__count mono">{{ entries.length }} arquivos alterados</span>
    </header>
    <ul class="disk-log__list">
      <li v-for="entry in reversed" :key="entry.id" class="disk-log__entry">
        <div class="disk-log__row">
          <span class="disk-log__badge" :class="`disk-log__badge--${entry.type}`">
            {{ entry.type === 'updated' ? 'ATUALIZADO' : 'LIMPO' }}
          </span>
          <span class="disk-log__path mono">{{ entry.path }}</span>
          <span class="disk-log__time mono">{{ timeShort(entry.at) }}</span>
        </div>
        <p class="disk-log__meaning">{{ entry.meaning }}</p>
        <div class="disk-log__diff">
          <div class="disk-log__col">
            <span class="disk-log__col-label mono">Antes</span>
            <pre class="disk-log__code mono">{{ entry.before }}</pre>
          </div>
          <div class="disk-log__col">
            <span class="disk-log__col-label mono">Depois</span>
            <pre class="disk-log__code mono">{{ entry.after }}</pre>
          </div>
        </div>
      </li>
    </ul>
    <p v-if="entries.length === 0" class="disk-log__empty">
      Nenhuma escrita ainda — o app não grava nada; quem grava é a skill no harness.
    </p>
  </aside>
</template>
