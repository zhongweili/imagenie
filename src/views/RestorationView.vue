<template>
  <div class="restoration-view">
    <FunctionSettings :mode="'restoration'" />
    <ImageProcessor />
  </div>
</template>

<script setup lang="ts">
import FunctionSettings from '@/components/FunctionSettings.vue'
import ImageProcessor from '@/components/ImageProcessor.vue'
</script> 
