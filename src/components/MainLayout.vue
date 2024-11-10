<!-- MainLayout.vue -->
<template>
  <el-container style="height: 100vh;">
    <el-aside
      :width="isCollapse ? '64px' : '200px'"
      class="main-layout__aside"
    >
      <Sidebar :collapse="isCollapse" @toggle-sidebar="toggleSidebar" />
    </el-aside>

    <el-container>
      <Header @toggle-sidebar="toggleSidebar" />
      <el-main class="main-container">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Sidebar from '@/components/Sidebar.vue';
import Header from '@/components/Header.vue';
const isCollapse = ref(false);
const toggleSidebar = () => {
  isCollapse.value = !isCollapse.value;
};
</script>

<style>
.sidebar {
  width: 200px;
  transition: transform 0.3s ease;
}
.sidebar.collapsed {
  transform: translateX(-136px);
}
.main-container {
  padding: 0;
  background: #f5f7fa;
  position: relative;
  overflow: hidden;
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
