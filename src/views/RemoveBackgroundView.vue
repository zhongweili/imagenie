<template>
  <div class="remove-background-view">
    <FunctionSettings :mode="'remove-background'" />
    <ImageProcessor />
  </div>
</template>

<script setup lang="ts">
import FunctionSettings from '@/components/FunctionSettings.vue'
import ImageProcessor from '@/components/ImageProcessor.vue'
</script> 
