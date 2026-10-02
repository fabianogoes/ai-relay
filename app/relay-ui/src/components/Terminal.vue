<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { sendInput, subscribeData } from '../lib/execution'
import { readTerminalTheme } from '../lib/terminal-theme'

const props = defineProps<{ runId: string }>()

const container = ref<HTMLDivElement | null>(null)

let term: Terminal | null = null
let fit: FitAddon | null = null
let unsubscribe: (() => void) | null = null
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (!container.value) return
  term = new Terminal({
    cursorBlink: true,
    convertEol: false,
    fontFamily: 'var(--font-mono)',
    fontSize: 13,
    theme: readTerminalTheme(getComputedStyle(container.value)),
  })
  fit = new FitAddon()
  term.loadAddon(fit)
  term.open(container.value)
  fit.fit()

  term.onData((text) => sendInput(text))

  unsubscribe = subscribeData((data) => term?.write(data))

  resizeObserver = new ResizeObserver(() => fit?.fit())
  resizeObserver.observe(container.value)

  watch(
    () => props.runId,
    () => {
      term?.reset()
      fit?.fit()
    },
  )
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  unsubscribe?.()
  term?.dispose()
  term = null
  fit = null
})
</script>

<template>
  <div ref="container" class="terminal" aria-label="Terminal embutido"></div>
</template>
