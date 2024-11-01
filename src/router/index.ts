import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/compress'
  },
  {
    path: '/compress',
    name: 'Compress',
    component: () => import('../views/CompressView.vue')
  },
  {
    path: '/repair',
    name: 'Repair',
    component: () => import('../views/RepairView.vue')
  },
  {
    path: '/remove-bg',
    name: 'RemoveBg',
    component: () => import('../views/RemoveBgView.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes: routes
})

export default router 
