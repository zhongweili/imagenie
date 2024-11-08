<!-- MainContent.vue -->
<template>
  <div class="main-content">
    <div class="content-area">
      <div class="image-preview" v-if="store.files[0]">
        <img :src="getImageUrl(store.files[0].path)" alt="Preview" />
      </div>
      <div class="upload-area" v-else>
        <FileSelection />
      </div>
    </div>
    
    <div class="control-panel">
      <el-button 
        class="settings-toggle"
        type="text"
        @click="showSettings = !showSettings"
      >
        <el-icon><Setting /></el-icon>
      </el-button>
      
      <transition name="slide">
        <FunctionSettings v-if="showSettings" class="settings-drawer" />
      </transition>
      
      <ProcessingControl class="processing-controls" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { useStore } from '../store';
import FileSelection from './FileSelection.vue';
import FunctionSettings from './FunctionSettings.vue';
import ProcessingControl from './ProcessingControl.vue';
import { convertFileSrc } from '@tauri-apps/api/core';

const store = useStore();
const showSettings = ref(false);

const getImageUrl = (path) => {
  return convertFileSrc(path);
};
</script>

<style scoped>
.main-content {
  height: 100%;
  position: relative;
  display: flex;
}

.content-area {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  background: #fff;
}

.image-preview {
  max-width: 90%;
  max-height: 90%;
  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
}

.control-panel {
  position: absolute;
  right: 0;
  top: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.settings-toggle {
  margin: 16px;
  z-index: 2;
}

.settings-drawer {
  position: absolute;
  right: 0;
  top: 0;
  width: 300px;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.1);
  padding: 60px 20px 20px;
}

.processing-controls {
  position: absolute;
  bottom: 20px;
  right: 20px;
  z-index: 2;
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
  