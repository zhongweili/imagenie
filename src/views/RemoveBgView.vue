<template>
  <div class="remove-bg-view">
    <FileSelection />
    <FunctionSettings :mode="'remove-bg'" />
    <ProcessingControl />
  </div>
</template>

<script setup lang="ts">
import FileSelection from '@/components/FileSelection.vue';
import FunctionSettings from '@/components/FunctionSettings.vue';
import ProcessingControl from '@/components/ProcessingControl.vue';
</script> 
