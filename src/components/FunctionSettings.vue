<template>
    <div class="function-settings">
      <template v-if="mode === 'upscaling'">
        <div class="setting-item">
          <label>放大倍数</label>
          <select v-model="scale">
            <option value="2">2x</option>
            <option value="4">4x</option>
          </select>
        </div>
        <!-- 其他放大相关设置 -->
      </template>
  
      <template v-else-if="mode === 'restoration'">
        <div class="setting-item">
          <label>修复强度</label>
          <input type="range" v-model="strength" min="0" max="100" />
        </div>
        <!-- 其他修复相关设置 -->
      </template>
    </div>
  </template>
  
  <script setup lang="ts">
  import { computed } from 'vue'
  import { useStore } from '@/store/index.ts'
  
  const props = defineProps<{
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
