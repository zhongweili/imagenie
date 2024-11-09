<template>
  <div class="processing-control">
    <el-button
      type="primary"
      size="large"
      :loading="isProcessing"
      class="process-button"
      @click="startProcessing"
    >
      <el-icon><VideoPlay /></el-icon>
      <span>开始处理</span>
    </el-button>
    
    <transition name="fade">
      <div v-if="isProcessing" class="progress-overlay">
        <el-progress type="circle" :percentage="progress"></el-progress>
        <p class="current-file">{{ currentFileName }}</p>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.processing-control {
  position: relative;
}

.process-button {
  background: linear-gradient(45deg, #409EFF, #36D1DC);
  border: none;
  padding: 12px 24px;
  border-radius: 25px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
}

.process-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
}

.progress-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.current-file {
  margin-top: 16px;
  color: #606266;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

<script setup lang="ts">
import { ref } from 'vue'
import { VideoPlay } from '@element-plus/icons-vue'
import { useStore } from '../store'
import { invoke } from '@tauri-apps/api/core';
const isProcessing = ref(false)
const progress = ref(0)
const currentFileName = ref('')
const store = useStore()

const startProcessing = async () => {
  isProcessing.value = true
  console.log(store.input_path, store.output_dir)
  try {
    await invoke('upscale_image', {
      inputPath: store.input_path,
      outputDir: store.output_dir
    })
  } catch (error) {
    console.error('Upscaling failed:', error)
  } finally {
    isProcessing.value = false
  }
}
</script>
  