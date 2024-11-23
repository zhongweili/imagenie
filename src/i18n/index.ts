import { createI18n } from 'vue-i18n'

const messages = {
  en: {
    common: {
      upload: 'Upload Image',
      process: 'Process',
      download: 'Download',
      settings: 'Settings',
      cancel: 'Cancel',
      confirm: 'Confirm'
    },
    nav: {
      removeBackground: 'Background Removal',
      upscaling: 'Image Upscaling',
      resizing: 'Image Resizing'
    },
    settings: {
      modelPath: 'Model Path',
      selectModel: 'Select Model',
      processingMode: 'Processing Mode',
      quality: 'Quality',
      speed: 'Speed'
    },
    notification: {
      success: 'Success',
      error: 'Error',
      processing: 'Processing'
    },
    function: {
      scale: 'Scale',
      strength: 'Strength',
      removeBackground: 'Background Removal',
      upscaling: 'Image Upscaling',
      restoration: 'Face Restoration',
      resizing: 'Image Resizing'
    },
    layout: {
      sidebar: 'Sidebar',
      header: 'Header'
    },
    imageProcessor: {
      // dropHint: 'Drag image here or click to select',
      dropHint: 'Click to select',
      selectInput: 'Select Input Image',
      selectOutput: 'Select Output Path',
      startProcess: 'Start Processing',
      processing: 'Processing...',
      processingStatus: 'Processing image, please wait...',
      processingCompleted: 'Processing Completed',
      processingCompletedDesc: 'Processing completed, please compare the results'
    },
    sidebar: {
      title: 'Imagenie',
      menu: {
        upscaling: 'Image Upscaling',
        resizing: 'Image Resizing',
        restoration: 'Face Restoration',
        removeBackground: 'Background Removal'
      }
    }
  },
  zh: {
    common: {
      upload: '上传图片',
      process: '处理',
      download: '下载',
      settings: '设置',
      cancel: '取消',
      confirm: '确认'
    },
    nav: {
      removeBackground: '移除背景',
      upscaling: '图片放大',
      resizing: '调整尺寸'
    },
    settings: {
      modelPath: '模型路径',
      selectModel: '选择模型',
      processingMode: '处理模式',
      quality: '质量优先',
      speed: '速度优先'
    },
    notification: {
      success: '成功',
      error: '错误',
      processing: '处理中'
    },
    function: {
      scale: '放大倍数',
      strength: '修复强度',
      removeBackground: '移除背景',
      upscaling: '图片放大',
      restoration: '图片修复',
      resizing: '调整尺寸'
    },
    layout: {
      sidebar: '侧边栏',
      header: '顶部栏'
    },
    imageProcessor: {
      // dropHint: '拖拽图片到这里或点击选择',
      dropHint: '点击选择',
      selectInput: '选择输入图片',
      selectOutput: '选择输出路径',
      startProcess: '开始处理',
      processing: '处理中...',
      processingStatus: '正在处理图片，请稍候...',
      processingCompleted: '处理完成',
      processingCompletedDesc: '处理完成，请对比查看结果'
    },
    sidebar: {
      title: 'Imagenie',
      menu: {
        upscaling: '图像放大',
        resizing: '图像缩放',
        restoration: '人像修复',
        removeBackground: '背景移除'
      }
    }
  }
}

export const i18n = createI18n({
  legacy: false, // Set to false to use Composition API
  locale: 'en', // Default language
  fallbackLocale: 'en',
  messages
})
