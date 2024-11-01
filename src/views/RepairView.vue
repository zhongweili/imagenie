<template>
  <div class="repair-view">
    <FileSelection />
    <FunctionSettings :mode="'repair'" />
    <ProcessingControl />
  </div>
</template>

<script setup lang="ts">
import FileSelection from '@/components/FileSelection.vue';
import FunctionSettings from '@/components/FunctionSettings.vue';
import ProcessingControl from '@/components/ProcessingControl.vue';
</script> 
