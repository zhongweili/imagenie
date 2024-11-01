<template>
  <div class="sidebar" :class="{ 'sidebar--collapsed': collapse }">
    <div class="sidebar__logo" @click="$emit('toggle-sidebar')">
      <el-icon class="sidebar__logo-icon"><MagicStick /></el-icon>
      <span class="sidebar__logo-text" v-show="!collapse">Imagenie</span>
    </div>

    <el-menu
      :default-active="activeRoute"
      :collapse="collapse"
      :collapse-transition="false"
      router
      class="sidebar__menu"
      background-color="#333"
      text-color="#fff"
      active-text-color="#ffd04b"
      @select="handleSelect"
    >
      <el-menu-item 
        v-for="item in menuItems" 
        :key="item.path"
        :index="item.path"
        :route="item.path"
      >
        <el-icon><component :is="item.icon" /></el-icon>
        <template #title>{{ item.title }}</template>
      </el-menu-item>
    </el-menu>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Fold, Picture, Scissor, MagicStick } from '@element-plus/icons-vue';
import { useStore } from '@/store';

interface MenuItem {
  path: string;
  title: string;
  icon: Component;
}

const props = defineProps<{
  collapse: boolean;
}>();

defineEmits<{
  (e: 'toggle-sidebar'): void;
}>();

const router = useRouter();
const route = useRoute();
const store = useStore();

const activeRoute = computed(() => route.path);

const menuItems: MenuItem[] = [
  {
    path: '/compress',
    title: '批量压缩',
    icon: Fold,
  },
  {
    path: '/repair',
    title: '老照片修复',
    icon: Picture,
  },
  {
    path: '/remove-bg',
    title: '背景移除',
    icon: Scissor,
  },
];

const handleSelect = (path: string) => {
  if (path) {
    router.push(path);
    const functionName = path.replace('/', '');
    store.setActiveFunction(functionName);
  }
};
</script>

<style scoped lang="scss">
.sidebar {
  $self: &;
  width: 200px;
  height: 100%;
  background-color: #333;
  transition: width 0.3s ease;
  
  &--collapsed {
    width: 64px;
    
    #{$self}__logo {
      padding: 0 20px;
    }
  }

  &__logo {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 64px;
    padding: 0 20px;
    cursor: pointer;
    transition: padding 0.3s ease;

    &-icon {
      font-size: 24px;
      color: #fff;
    }

    &-text {
      color: #fff;
      font-size: 18px;
      font-weight: 600;
      white-space: nowrap;
    }
  }

  &__menu {
    border-right: none;
    
    :deep(.el-menu-item) {
      &.is-active {
        background-color: #444;
      }
      
      &:hover {
        background-color: #444;
      }
    }
  }
}

// Override element-plus styles
:deep(.el-menu--collapse) {
  width: 64px;
}

:deep(.el-icon > svg) {
  fill: currentColor;
}
</style>
