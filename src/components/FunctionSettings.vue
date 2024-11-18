<!-- not in use -->
<template>
    <div class="function-settings">
      <template v-if="mode === 'upscaling'">
        <div class="setting-item">
          <label>{{ t('function.scale') }}</label>
          <select v-model="scale">
            <option value="2">2x</option>
            <option value="4">4x</option>
          </select>
        </div>
      </template>

      <template v-else-if="mode === 'restoration'">
        <div class="setting-item">
          <label>{{ t('function.strength') }}</label>
          <input type="range" v-model="strength" min="0" max="100" />
        </div>
      </template>
    </div>
  </template>

  <script setup lang="ts">
  import { computed } from 'vue'
  import { useStore } from '@/store/index.ts'
  import { useI18n } from 'vue-i18n'

  const { t } = useI18n()

  const { mode } = defineProps<{
    mode: string
  }>()

  const store = useStore()

  const scale = computed({
    get: () => store.$state.settings.scale,
    set: (value) => store.updateSettings({ scale: value })
  })

  const strength = computed({
    get: () => store.$state.settings.strength,
    set: (value) => store.updateSettings({ strength: value })
  })
  </script>
