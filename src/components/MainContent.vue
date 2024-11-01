<!-- MainContent.vue -->
<template>
    <div class="main-content">
      <FileSelection />
      <FunctionSettings />
      <ProcessingControl />
    </div>
  </template>
  
  <script>
  import FileSelection from './FileSelection.vue';
  import FunctionSettings from './FunctionSettings.vue';
  import ProcessingControl from './ProcessingControl.vue';
  
  export default {
    name: 'MainContent',
    components: {
      FileSelection,
      FunctionSettings,
      ProcessingControl,
    },
  };
  </script>
  
  <style scoped>
  .main-content {
    padding: 20px;
    /* 其他样式 */
  }
  </style>
  