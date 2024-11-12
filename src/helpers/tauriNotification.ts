import {
    isPermissionGranted,
    requestPermission,
    sendNotification
  } from '@tauri-apps/plugin-notification'
  import { useI18n } from 'vue-i18n'
  import { i18n } from '@/i18n'
  
  async function checkPermission() {
    if (!(await isPermissionGranted())) {
      return (await requestPermission()) === 'granted'
    }
    return true
  }
  
  export async function enqueueNotification(title: string, body: string) {
    if (!(await checkPermission())) {
      return
    }
    sendNotification({ title, body })
  }
  
  export const showNotification = (type: 'success' | 'error' | 'processing', message: string) => {
    const t = i18n.global.t
    const title = t(`notification.${type}`)
    enqueueNotification(title, message)
  }
