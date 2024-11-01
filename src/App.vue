<!-- App.vue -->
<template>
  <main class="container">
    <MainLayout />
  </main>
</template>

<script>
import MainLayout from './components/MainLayout.vue';

export default {
  name: 'App',
  components: {
    MainLayout,
  },
};
</script>

<style>
/* 可在此处引入全局样式 */
@import './assets/styles.css';
</style>
