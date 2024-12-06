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
import { ref } from 'vue';
import { exists, writeFile, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
import { fetch } from '@tauri-apps/plugin-http';
import { getLatestModelRelease } from '../utils/githubRelease';
import { FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL, BACKGROUND_REMOVAL_MODEL } from '../config/models';
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const downloadProgress = ref(0);
const emit = defineEmits(['initializationComplete']);
const MODELS = [BACKGROUND_REMOVAL_MODEL,FACE_RESTORATION_MODEL, IMAGE_UPSCALING_MODEL];

async function checkAndDownloadModels() {
  try {
    console.log('Starting model initialization...');
    await mkdir('models', {
      baseDir: BaseDirectory.AppLocalData,
      recursive: true
    });

    let completedModels = 0;
    for (const model of MODELS) {
      console.log(`Checking model: ${model}`);
      const modelPath = `models/${model}`;
      const needsDownload = !(await exists(modelPath, {
        baseDir: BaseDirectory.AppLocalData
      }));

      if (needsDownload) {
        console.log(`Model ${model} needs to be downloaded`);
        await downloadModel(model, completedModels);
        completedModels++;
        console.log(`Model ${model} download completed`);
      } else {
        console.log(`Model ${model} already exists, skipping download`);
        downloadProgress.value = Math.round(((completedModels + 1) / MODELS.length) * 100);
        completedModels++;
      }
    }

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

async function downloadModel(modelName: string, completedModels: number): Promise<void> {
  try {
    console.log(`Starting download for model: ${modelName}`);
    const modelRelease = await getLatestModelRelease(modelName);
    console.log(`Got release info for ${modelName}:`, modelRelease);

    console.log(`Fetching model from URL: ${modelRelease.url}`);
    const response = await fetch(modelRelease.url, {
      method: 'GET',
    });

    if (!response.ok) {
      throw new Error(`Failed to download model ${modelName}, status: ${response.status}`);
    }

    const contentLength = Number(response.headers.get('content-length'));
    console.log(`Total size for ${modelName}: ${(contentLength / 1024 / 1024).toFixed(2)}MB`);
    let receivedLength = 0;

    const reader = response.body!.getReader();

    console.log(`Starting to write ${modelName} to disk...`);
    let fileContent = new Uint8Array(0);

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      if (value) {
        await writeFile(`models/${modelName}`, value, {
          baseDir: BaseDirectory.AppLocalData,
          append: true
        });

        receivedLength += value.length;
        const currentModelProgress = (receivedLength / contentLength) * 100;

        if (Math.floor(currentModelProgress) % 10 === 0) {
          console.log(`${modelName} download progress: ${Math.floor(currentModelProgress)}%`);
          console.log(`Wrote ${(receivedLength / 1024 / 1024).toFixed(2)}MB to disk`);
        }

        downloadProgress.value = Math.round(
          ((completedModels * 100) + currentModelProgress) / MODELS.length
        );
      }
    }

    console.log(`${modelName} file write completed`);

    console.log(`Updating version information for ${modelName}`);
    await writeFile('models/version.json', new TextEncoder().encode(JSON.stringify({
      version: modelRelease.version,
      lastUpdated: new Date().toISOString()
    })), {
      baseDir: BaseDirectory.AppLocalData
    });

    console.log(`${modelName} processing completed successfully`);
  } catch (error) {
    console.error(`Error processing ${modelName}:`, error);
    throw error;
  }
}

checkAndDownloadModels();
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
