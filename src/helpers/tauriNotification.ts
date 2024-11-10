import {
    isPermissionGranted,
    requestPermission,
    sendNotification
  } from '@tauri-apps/plugin-notification'
  
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
