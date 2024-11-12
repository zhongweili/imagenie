<template>
  <div class="resizing-view">
    <div class="layout">
      <FunctionSettings :mode="'resizing'" class="settings" />
      <ImageProcessor :mode="'resizing'" class="processor" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeMount } from 'vue'
import { useStore } from '@/store/index.ts'
import FunctionSettings from '@/components/FunctionSettings.vue'
import ImageProcessor from '@/components/ImageProcessor.vue'

const store = useStore()

onBeforeMount(() => {
  // Reset the status before the view is loaded
  store.resetState()
})
</script>

<style scoped>
.layout {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 1rem;
  height: 100%;
}

.settings {
  background: #f5f5f5;
  padding: 1rem;
}

.processor {
  flex: 1;
}
</style>
