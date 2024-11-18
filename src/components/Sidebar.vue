<template>
  <div class="sidebar" :class="{ 'sidebar--collapsed': collapse }">
    <div class="sidebar__logo" @click="$emit('toggle-sidebar')">
      <span class="sidebar__logo-text" v-show="!collapse">{{ t('sidebar.title') }}</span>
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
        <template #title>{{ t(item.titleKey) }}</template>
      </el-menu-item>
    </el-menu>

    <div class="sidebar__bottom">
      <button @click="toggleLanguage" class="lang-switch">
        {{ locale === 'en' ? '中文' : 'English' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n'
import * as ElementPlusIconsVue from '@element-plus/icons-vue';
import { useStore } from '@/store/index.ts'

const { t } = useI18n()
const router = useRouter();
const route = useRoute();
const store = useStore();

// Register all icons globally
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  const app = getCurrentInstance()
  if (app && typeof component === 'object') {
    app.appContext.app.component(key, component)
  }
}

const { collapse } = defineProps<{
  collapse: boolean
}>()

const { locale } = useI18n()
const toggleLanguage = () => {
  locale.value = locale.value === 'en' ? 'zh' : 'en'
}

defineEmits(['toggle-sidebar'])

const activeRoute = computed(() => route.path);

const menuItems = [
  {
    path: '/upscaling',
    titleKey: 'sidebar.menu.upscaling',
    icon: 'Fold',
  },
  {
    path: '/restoration',
    titleKey: 'sidebar.menu.restoration',
    icon: 'Picture',
  },
  {
    path: '/remove-background',
    titleKey: 'sidebar.menu.removeBackground',
    icon: 'Scissor',
  },
];

const handleSelect = (path: string) => {
  if (path) {
    router.push(path);
    store.resetState();
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
  position: relative;
  display: flex;
  flex-direction: column;

  &--collapsed {
    width: 64px;

    #{$self}__logo {
      padding: 0;
      justify-content: center;

      .collapse-icon {
        margin: 0;
      }
    }

    .lang-switch {
      display: none;
    }

    #{$self}__logo-text {
      display: none;
    }

    .sidebar__bottom {
      padding: 8px 0;
      display: flex;
      justify-content: center;

      .lang-switch {
        width: auto;
        padding: 8px 0;
      }
    }
  }

  &__logo {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 64px;
    padding: 0 20px;
    cursor: pointer;
    transition: all 0.3s ease;

    &:hover {
      background-color: #444;
    }

    &-text {
      color: #fff;
      font-size: 18px;
      font-weight: 600;
      white-space: nowrap;
      text-align: center;
    }
  }

  &__menu {
    flex: 1;
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

  &__bottom {
    padding: 16px;
    border-top: 1px solid #444;

    .lang-switch {
      width: 100%;
      background: none;
      border: none;
      color: #fff;
      cursor: pointer;
      padding: 8px;
      border-radius: 4px;
      transition: background-color 0.3s;

      &:hover {
        background-color: #444;
      }
    }
  }

  .collapse-icon {
    cursor: pointer;
    font-size: 20px;
    color: #fff;
    transition: transform 0.3s;
    position: absolute;
    right: 12px;

    &:hover {
      color: #ffd04b;
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

.lang-switch {
  background: none;
  border: none;
  color: #fff;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;

  &:hover {
    background-color: #444;
  }
}
</style>
