import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/restoration'
  },
  {
    path: '/restoration',
    name: 'Restoration',
    component: () => import('@/views/RestorationView.vue')
  },
  {
    path: '/remove-background',
    name: 'RemoveBackground',
    component: () => import('@/views/RemoveBackgroundView.vue')
  },
  {
    path: '/upscaling',
    name: 'Upscaling',
    component: () => import('@/views/UpscalingView.vue')
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes: routes
})

export default router
