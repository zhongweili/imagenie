<template>
  <div class="restoration-view">
    <ImageProcessor :mode="'restoration'" />
  </div>
</template>

<script setup lang="ts">
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
