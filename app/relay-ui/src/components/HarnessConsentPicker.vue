<script setup lang="ts">
import { computed } from 'vue'
import { CONSENT_OPTIONS, allHarnesses, type ConsentLevel } from '../lib/harness'
import HarnessIcon from './HarnessIcon.vue'

withDefaults(
  defineProps<{
    selectedHarnessId: string | null
    selectedConsent: ConsentLevel
    radioName?: string
  }>(),
  { radioName: 'relay-consent' },
)

const emit = defineEmits<{
  (event: 'update:selectedHarnessId', id: string): void
  (event: 'update:selectedConsent', level: ConsentLevel): void
}>()

const harnesses = computed(() => allHarnesses())

function stateLabel(state: string): string {
  if (state === 'installed') return 'Instalado'
  if (state === 'unauthenticated') return 'Não autenticado'
  return 'Ausente'
}
</script>

<template>
  <section class="selector__section">
    <h3 class="selector__section-title">Harness</h3>
    <ul class="selector__harnesses">
      <li v-for="harness in harnesses" :key="harness.id">
        <button
          class="harness-option"
          :class="{
            'is-selected': selectedHarnessId === harness.id,
            'is-disabled': harness.state === 'absent',
          }"
          :disabled="harness.state === 'absent'"
          @click="emit('update:selectedHarnessId', harness.id)"
        >
          <span class="harness-avatar" aria-hidden="true">
            <HarnessIcon :id="harness.id" />
          </span>
          <span class="harness-option__name">{{ harness.name }}</span>
          <span class="harness-option__version mono">{{ harness.version }}</span>
          <span class="harness-option__state">{{ stateLabel(harness.state) }}</span>
        </button>
      </li>
    </ul>
  </section>

  <section class="selector__section">
    <h3 class="selector__section-title">Consentimento</h3>
    <div class="selector__consents" role="radiogroup" aria-label="Nível de consentimento">
      <label
        v-for="option in CONSENT_OPTIONS"
        :key="option.id"
        class="consent-option"
        :class="{ 'is-selected': selectedConsent === option.id }"
      >
        <input
          class="consent-option__input"
          type="radio"
          :name="radioName"
          :value="option.id"
          :checked="selectedConsent === option.id"
          @change="emit('update:selectedConsent', option.id)"
        />
        <span class="consent-option__label">{{ option.label }}</span>
      </label>
    </div>
  </section>
</template>
