<template>
  <div class="upscaling-view">
    <FunctionSettings :mode="'upscaling'" />
    <ImageProcessor />
  </div>
</template>

<script setup lang="ts">
import FunctionSettings from '@/components/FunctionSettings.vue'
import ImageProcessor from '@/components/ImageProcessor.vue'
</script>
