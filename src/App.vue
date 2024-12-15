<!-- App.vue -->
<template>
  <main class="container">
    <InitializationScreen v-if="needsInitialization" @initialization-complete="onInitializationComplete" />
    <MainLayout v-else />
  </main>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import MainLayout from '@/components/MainLayout.vue';
import InitializationScreen from '@/components/InitializationScreen.vue';
import { FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL, BACKGROUND_REMOVAL_MODEL } from './config/models';

const needsInitialization = ref(true);
const MODELS = [BACKGROUND_REMOVAL_MODEL, FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL];

onMounted(async () => {
  try {
    let requiresDownload = false;
    for (const model of MODELS) {
      const needsDownload = await invoke('check_model_exists', { modelName: model });
      if (needsDownload) {
        requiresDownload = true;
        break;
      }
    }

    if (!requiresDownload) {
      needsInitialization.value = false;
    }
  } catch (error) {
    console.error('Error checking models:', error);
  }
});

function onInitializationComplete() {
  needsInitialization.value = false;
}
</script>

<style>
@import './assets/styles.css';
</style>
