<template>
  <div class="initialization-overlay">
    <div class="initialization-content">
      <h2>{{ t('initialization.title') }}</h2>
      <p>{{ t('initialization.downloading') }} {{ downloadProgress }}%</p>
      <div class="progress-bar">
        <div :style="{ width: `${downloadProgress}%` }" class="progress"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getLatestModelRelease } from '../utils/githubRelease';
import { FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL, BACKGROUND_REMOVAL_MODEL } from '../config/models';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const downloadProgress = ref(0);
const emit = defineEmits(['initializationComplete']);
const MODELS = [BACKGROUND_REMOVAL_MODEL, FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL];
const currentModelName = ref('');

onMounted(async () => {
  interface DownloadProgress {
    model_name: string;
    progress: number;
    total_progress: number;
  }

  await listen<DownloadProgress>('download-progress', (event) => {
    console.log('Progress event received:', event.payload);
    currentModelName.value = event.payload.model_name;
    if (!isNaN(event.payload.total_progress)) {
      downloadProgress.value = Math.round(event.payload.total_progress);
    }
  });

  await checkAndDownloadModels();
});

async function checkAndDownloadModels() {
  try {
    console.log('Starting model initialization...');
    const modelsToDownload = [];

    for (const model of MODELS) {
      const needsDownload = await invoke('check_model_exists', { modelName: model });

      if (needsDownload) {
        console.log(`Model ${model} needs to be downloaded`);
        const modelRelease = await getLatestModelRelease(model);
        modelsToDownload.push({
          name: model,
          url: modelRelease.url,
          version: modelRelease.version
        });
      } else {
        console.log(`Model ${model} already exists, skipping download`);
      }
    }

    if (modelsToDownload.length > 0) {
      await invoke('download_models', { models: modelsToDownload });
    }

    await invoke('init_background_removal');
    await invoke('init_face_restoration');
    await invoke('init_upscaling');

    console.log('All models processed successfully');
    downloadProgress.value = 100;
    emit('initializationComplete');
  } catch (error) {
    console.error('Initialization failed:', error);
    if (error instanceof Error) {
      console.error('Error details:', {
        message: error.message,
        stack: error.stack
      });
    }
  }
}
</script>

<style scoped>
.initialization-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.initialization-content {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  text-align: center;
}

.progress-bar {
  width: 300px;
  height: 20px;
  background: #eee;
  border-radius: 10px;
  overflow: hidden;
  margin-top: 1rem;
}

.progress {
  height: 100%;
  background: #4CAF50;
  transition: width 0.3s ease;
}
</style>
