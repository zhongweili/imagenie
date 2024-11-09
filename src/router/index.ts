import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/upscaling'
  },
  {
    path: '/upscaling',
    name: 'Upscaling',
    component: () => import('@/views/UpscalingView.vue')
  },
  {
    path: '/resizing',
    name: 'Resizing',
    component: () => import('@/views/ResizingView.vue')
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
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes: routes
})

export default router 
