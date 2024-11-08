<template>
  <div class="file-selection">
    <div 
      class="upload-area"
      @click="handleClick"
      @dragover.prevent
      @drop.prevent="handleDrop"
    >
      <el-icon class="upload-icon"><Upload /></el-icon>
      <div class="upload-text">
        <em>点击或拖拽图片到此处</em>
        <p class="output-path" v-if="outputPath">
          输出路径: {{ outputPath }}
          <el-button type="text" @click.stop="selectOutputPath">修改</el-button>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { Upload } from '@element-plus/icons-vue';
import { useStore } from '../store';
import { save, open } from '@tauri-apps/plugin-dialog';
import { dirname } from '@tauri-apps/api/path';

const store = useStore();
const outputPath = ref('');

const handleClick = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg']
      }]
    });

    if (selected) {
      outputPath.value = await dirname(selected);
      store.addFiles([{
        path: selected
      }]);
    }
  } catch (error) {
    console.error('文件选择错误:', error);
  }
};

// 处理拖拽文件
const handleDrop = async (e) => {
  e.preventDefault();
  const files = e.dataTransfer.files;
  if (files.length > 0) {
    // 这里需要处理拖拽的文件
    // 注意：由于安全限制，可能需要通过 Tauri API 来处理拖拽的文件
    console.log('Dropped files:', files);
  }
};

const selectOutputPath = async (e) => {
  e.preventDefault();
  const selected = await open({
    directory: true
  });
  
  if (selected) {
    outputPath.value = selected;
  }
};
</script>

<style scoped>
.file-selection {
  width: 100%;
  max-width: 400px;
}

.upload-area {
  border: 2px dashed #d9d9d9;
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s;
}

.upload-area:hover {
  border-color: #409EFF;
}

.upload-icon {
  font-size: 48px;
  color: #909399;
  margin-bottom: 16px;
}

.upload-text {
  color: #909399;
}

.output-path {
  margin-top: 8px;
  font-size: 12px;
  color: #606266;
}
</style>
  