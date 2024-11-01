<template>
    <div class="file-selection">
      <el-button type="primary" @click="selectFiles">添加文件/文件夹</el-button>
      <div
        class="drop-area"
        @dragover.prevent
        @drop.prevent="handleDrop"
      >
        <p>拖拽文件或文件夹到此处</p>
        <div class="thumbnails">
          <div v-for="file in files" :key="file.path" class="thumbnail">
            <img :src="file.thumbnail" alt="thumbnail" />
            <p class="filename" :title="file.name">{{ truncateFilename(file.name) }}</p>
          </div>
        </div>
      </div>
    </div>
  </template>
  
  <script>
  import { ref } from 'vue';
  import { open } from '@tauri-apps/plugin-dialog';
  import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
  import { useStore } from '@/store';
  export default {
    setup() {
      const files = ref([]);
  
      const store = useStore();

      const selectFiles = async () => {
        const selected = await open({
          multiple: true,
          directory: false,
          filters: [
            {
              name: 'Images',
              extensions: ['png', 'jpg', 'jpeg', 'gif'],
            },
          ],
        });
        if (selected) {
          // 处理选中的文件
          addFiles(selected);
        }
      };
  
      const handleDrop = async (event) => {
        const droppedFiles = event.dataTransfer.files;
        for (let i = 0; i < droppedFiles.length; i++) {
          const file = droppedFiles[i];
          // 处理文件
          addFile(file.path);
        }
      };
  
      const addFiles = async (paths) => {
        if (!Array.isArray(paths)) {
          paths = [paths];
        }
        for (const path of paths) {
          await addFile(path);
        }
      };
  
      const addFile = async (path) => {
        // 读取文件，生成缩略图
        const data = await readFile(path);
        const blob = new Blob([data]);
        const url = URL.createObjectURL(blob);
        files.value.push({
          path,
          name: path.split('/').pop(),
          thumbnail: url,
        });
        store.addFiles(files.value);
      };
  
      const truncateFilename = (filename) => {
        return filename.length > 15 ? filename.slice(0, 12) + '...' : filename;
      };
      
      return { files, selectFiles, handleDrop, truncateFilename };
    },
  };
  </script>
  
  <style scoped>
  .file-selection {
    margin-bottom: 20px;
  }
  .drop-area {
    border: 2px dashed #d9d9d9;
    padding: 20px;
    text-align: center;
    margin-top: 10px;
  }
  .thumbnails {
    display: flex;
    flex-wrap: wrap;
    margin-top: 10px;
  }
  .thumbnail {
    width: 100px;
    margin: 8px;
    text-align: center;
  }
  .filename {
    margin: 4px 0;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .thumbnail img {
    width: 100%;
    height: 100px;
    object-fit: cover;
    border-radius: 4px;
  }
  </style>
  