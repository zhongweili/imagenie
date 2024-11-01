<template>
    <div class="processing-control">
      <el-button type="primary" @click="startProcessing" :loading="isProcessing">
        开始处理
      </el-button>
      <div v-if="isProcessing" class="progress">
        <el-progress :percentage="progress"></el-progress>
        <p>当前处理文件：{{ currentFileName }}</p>
      </div>
      <el-alert v-if="message" :title="message" type="info" show-icon></el-alert>
    </div>
  </template>
  
  <script>
  import { ref, computed } from 'vue';
  import { useStore } from '../store';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  export default {
    setup() {
      const store = useStore();
      const isProcessing = ref(false);
      const progress = ref(0);
      const currentFileName = ref('');
      const message = ref('');
  
      const startProcessing = async () => {
        if (store.files.length === 0) {
          message.value = '请先添加文件';
          return;
        }
        isProcessing.value = true;
        progress.value = 0;
        currentFileName.value = '';
  
        // 监听进度更新事件
        const unlisten = await listen('processing-progress', (event) => {
          progress.value = event.payload.progress;
          currentFileName.value = event.payload.fileName;
        });
  
        // 调用后端命令
        try {
          await invoke('process_images', {
            files: store.files.map((f) => f.path),
            options: store.functionSettings[store.activeFunction],
            functionName: store.activeFunction,
          });
          message.value = '处理完成';
        } catch (error) {
          message.value = `处理失败：${error}`;
        } finally {
          isProcessing.value = false;
          unlisten();
        }
      };
  
      return { isProcessing, progress, currentFileName, message, startProcessing };
    },
  };
  </script>
  
  <style scoped>
  .processing-control {
    margin-top: 20px;
  }
  .progress {
    margin-top: 10px;
  }
  </style>
  