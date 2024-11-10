<template>
  <div class="restoration-view">
    <FunctionSettings :mode="'restoration'" />
    <ImageProcessor :mode="'restoration'" />
  </div>
</template>

<script setup lang="ts">
import FunctionSettings from '@/components/FunctionSettings.vue'
import ImageProcessor from '@/components/ImageProcessor.vue'
</script> 

<style scoped>
.restoration-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.image-processor) {
  flex: 1;
  min-height: 0;
}
</style>
