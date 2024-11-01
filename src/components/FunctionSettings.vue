<template>
    <div class="function-settings">
      <el-form :model="settings" label-width="120px">
        <template v-if="activeFunction === 'compress'">
          <el-form-item label="压缩质量">
            <el-slider v-model="settings.quality" :min="1" :max="100"></el-slider>
          </el-form-item>
        </template>
        <template v-else-if="activeFunction === 'repair'">
          <el-form-item label="修复强度">
            <el-slider v-model="settings.intensity" :min="1" :max="10"></el-slider>
          </el-form-item>
        </template>
        <template v-else-if="activeFunction === 'remove-bg'">
          <el-form-item label="细节保留">
            <el-switch v-model="settings.preserveDetails"></el-switch>
          </el-form-item>
        </template>
        <!-- 可根据需要添加更多功能的设置项 -->
      </el-form>
    </div>
  </template>
  
  <script>
  import { computed } from 'vue';
  import { useStore } from '../store';
  
  export default {
    setup() {
      const store = useStore();
      const activeFunction = computed(() => store.activeFunction);
  
      const settings = computed({
        get: () => store.functionSettings[activeFunction.value],
        set: (value) => {
          store.functionSettings[activeFunction.value] = value;
        },
      });
  
      return { activeFunction, settings };
    },
  };
  </script>
  
  <style scoped>
  .function-settings {
    margin-bottom: 20px;
  }
  </style>
  