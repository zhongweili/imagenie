<!-- App.vue -->
<template>
  <main class="container">
    <InitializationScreen v-if="isInitializing" @initialization-complete="onInitializationComplete" />
    <MainLayout v-else />
  </main>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import MainLayout from '@/components/MainLayout.vue';
import InitializationScreen from '@/components/InitializationScreen.vue';

const isInitializing = ref(true);

function onInitializationComplete() {
  isInitializing.value = false;
}
</script>

<style>
@import './assets/styles.css';
</style>
