<template>
  <div class="image-processor">
    <div class="image-display">
      <div 
        class="drop-zone"
        @drop.prevent="handleDrop"
        @dragover.prevent
        @dragenter.prevent
      >
        <img v-if="inputImage" :src="inputImage" alt="Input image" />
        <div v-else class="placeholder">
          拖拽图片到这里或点击选择
        </div>
      </div>
    </div>
    
    <div class="controls">
      <button @click="selectInputFile">选择输入图片</button>
      <button @click="selectOutputPath">选择输出路径</button>
      <button 
        @click="startProcessing"
        :disabled="!canProcess || isProcessing"
      >
        {{ isProcessing ? '处理中...' : '开始处理' }}
      </button>
    </div>
    
    <div v-if="isProcessing" class="processing-status">
      正在处理图片，请稍候...
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useStore } from '@/store/index.ts'
import {  open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { appDataDir, join } from '@tauri-apps/api/path';

const store = useStore()
const isProcessing = ref(false)

const inputImage = computed(() => store.$state.inputImage)
const canProcess = computed(() => store.$state.inputPath && store.$state.outputPath)

const selectInputFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'Image',
      extensions: ['png', 'jpg', 'jpeg']
    }]
  });
  
  if (selected && !Array.isArray(selected)) {
    store.setInputFile(selected);
    const assetUrl = convertFileSrc(selected);
    store.setInputImage(assetUrl);
  }
}

const handleDrop = async (event: DragEvent) => {
  event.preventDefault()
  const file = event.dataTransfer?.files[0]
  if (file) {
    const appDataDirPath = await appDataDir();
    const filePath = await join(appDataDirPath, file.name);
    const assetUrl = convertFileSrc(filePath);
    store.setInputFile(filePath);
    store.setInputImage(assetUrl);
  }
}

const selectOutputPath = async () => {
  const selected = await open({
    directory: true
  });

  if (selected) {
    store.setOutputDir(selected)
  }
}

const startProcessing = async () => {
  isProcessing.value = true
  console.log(store.inputPath, store.outputPath)
  try {
    await invoke('upscale_image', {
      inputPath: store.inputPath,
      outputDir: store.outputPath
    })
  } catch (error) {
    console.error('Upscaling failed:', error)
  } finally {
    isProcessing.value = false
  }
}
</script>

<style scoped>
.image-processor {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.image-display {
  flex: 1;
  min-height: 300px;
}

.drop-zone {
  width: 100%;
  height: 100%;
  border: 2px dashed #ccc;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.controls {
  display: flex;
  gap: 1rem;
}

.processing-status {
  margin-top: 1rem;
  text-align: center;
  color: #666;
}
</style> 
