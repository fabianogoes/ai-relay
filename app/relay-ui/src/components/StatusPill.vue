<script setup lang="ts">
import { statusLabel, type DisplayStatus } from '../lib/presentation'

defineProps<{ status: DisplayStatus }>()
</script>

<template>
  <span class="status-pill" :class="`status-pill--${status}`" role="status" aria-live="polite">
    <span class="status-pill__dot" aria-hidden="true"></span>
    <span class="status-pill__label">{{ statusLabel(status) }}</span>
  </span>
</template>
