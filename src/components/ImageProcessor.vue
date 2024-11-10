<template>
  <div class="image-processor">
    <div class="image-container">
      <!-- 上传区域 -->
      <div 
        v-if="!processedImageUrl"
        class="upload-zone"
        @drop.prevent="handleDrop"
        @dragover.prevent
        @dragenter.prevent
        @click="selectInputFile"
      >
        <div class="image-preview-wrapper" v-if="inputImage">
          <img 
            :src="inputImage" 
            alt="Input image" 
            ref="previewImage"
            @load="handlePreviewImageLoad"
          />
        </div>
        <div v-else class="upload-placeholder">
          <span>拖拽图片到这里或点击选择</span>
        </div>
      </div>

      <!-- 对比区域 -->
      <div v-else class="comparison-container">
        <div class="comparison-wrapper" ref="comparisonWrapper">
          <div class="comparison-content">
            <img-comparison-slider class="comparison-slider">
              <div slot="first" class="image-wrapper">
                <img 
                  :src="inputImage || undefined" 
                  alt="Original image"
                  class="comparison-image"
                  ref="originalImage"
                  @load="handleImageLoad"
                />
              </div>
              <div slot="second" class="image-wrapper">
                <img 
                  :src="processedImageUrl" 
                  alt="Processed image"
                  class="comparison-image"
                  ref="processedImage"
                  @load="handleImageLoad"
                />
              </div>
            </img-comparison-slider>
          </div>
        </div>
      </div>
    </div>
    
    <div class="control-panel">
      <button 
        class="control-button"
        @click="selectInputFile"
      >
        选择输入图片
      </button>
      <button 
        class="control-button"
        @click="selectOutputDir"
      >
        选择输出路径
      </button>
      <button 
        class="control-button primary"
        @click="startProcessing"
        :disabled="!canProcess || isProcessing"
      >
        {{ isProcessing ? '处理中...' : '开始处理' }}
      </button>
    </div>
    
    <div v-if="isProcessing" class="processing-status">
      正在处理图片，请稍候...
    </div>
  </div>
</template>

<script setup lang="ts">
import { ImgComparisonSlider } from '@img-comparison-slider/vue'
import { computed, ref, onMounted } from 'vue'
import { useStore } from '@/store/index.ts'
import {  open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { appDataDir, join } from '@tauri-apps/api/path';
import { enqueueNotification } from '@/helpers/tauriNotification';

const store = useStore()
const isProcessing = ref(false)

const inputImage = computed(() => store.$state.inputImage)
const canProcess = computed(() => store.$state.inputPath && store.$state.outputDir)

const processedImageUrl = ref<string>('')

const selectInputFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'Image',
      extensions: ['png', 'jpg', 'jpeg']
    }]
  });
  
  if (selected && !Array.isArray(selected)) {
    store.setInputFile(selected);
    const assetUrl = convertFileSrc(selected);
    store.setInputImage(assetUrl);
  }
}

// TODO need to switch to tauri DRAG_DROP event
const handleDrop = async (event: DragEvent) => {
  event.preventDefault()
  const file = event.dataTransfer?.files[0]
  if (file) {
    const appDataDirPath = await appDataDir();
    const filePath = await join(appDataDirPath, file.name);
    const assetUrl = convertFileSrc(filePath);
    store.setInputFile(filePath);
    store.setInputImage(assetUrl);
  }
}

const selectOutputDir = async () => {
  const selected = await open({
    directory: true
  });

  if (selected) {
    store.setOutputDir(selected)
  }
}

const startProcessing = async () => {
  isProcessing.value = true
  console.log(store.inputPath, store.outputDir)
  try {
    const outputPath = await invoke('upscale_image', {
      inputPath: store.inputPath,
      outputDir: store.outputDir
    })
    processedImageUrl.value = convertFileSrc(outputPath as string)
    enqueueNotification(
      '图片处理完成',
      '图片处理完成'
    )
  } catch (error) {
    console.error('Upscaling failed:', error)
  } finally {
    isProcessing.value = false
  }
}

// 新增的图片尺寸处理逻辑
const originalImage = ref<HTMLImageElement | null>(null)
const processedImage = ref<HTMLImageElement | null>(null)
const comparisonWrapper = ref<HTMLElement | null>(null)

// 添加预览图片的引用
const previewImage = ref<HTMLImageElement | null>(null)

// 处理预览图片加载
const handlePreviewImageLoad = () => {
  if (!previewImage.value) return
  
  const img = previewImage.value
  const container = img.parentElement
  if (!container) return

  const containerWidth = container.clientWidth
  const containerHeight = container.clientHeight
  const imgRatio = img.naturalWidth / img.naturalHeight
  const containerRatio = containerWidth / containerHeight

  let displayWidth, displayHeight
  if (containerRatio > imgRatio) {
    // 容器更宽，以高度为基准
    displayHeight = containerHeight
    displayWidth = displayHeight * imgRatio
  } else {
    // 容器更高，以宽度为基准
    displayWidth = containerWidth
    displayHeight = displayWidth / imgRatio
  }

  img.style.width = `${displayWidth}px`
  img.style.height = `${displayHeight}px`
}

// 修改原有的图片加载处理函数
const handleImageLoad = () => {
  if (!originalImage.value || !processedImage.value || !comparisonWrapper.value) return

  const containerWidth = comparisonWrapper.value.clientWidth
  const containerHeight = comparisonWrapper.value.clientHeight

  const img1 = originalImage.value
  const img2 = processedImage.value

  // 确保两张图片都已加载完成
  if (!img1.complete || !img2.complete) {
    setTimeout(handleImageLoad, 100)
    return
  }

  const ratio1 = img1.naturalWidth / img1.naturalHeight
  const ratio2 = img2.naturalWidth / img2.naturalHeight
  const targetRatio = Math.max(ratio1, ratio2)

  let displayWidth, displayHeight
  if (containerWidth / containerHeight > targetRatio) {
    displayHeight = containerHeight
    displayWidth = displayHeight * targetRatio
  } else {
    displayWidth = containerWidth
    displayHeight = displayWidth / targetRatio
  }

  // 更新comparison-content的尺寸，添加类型断言
  const contentElement = comparisonWrapper.value.querySelector('.comparison-content') as HTMLElement
  if (contentElement) {
    contentElement.style.width = `${displayWidth}px`
    contentElement.style.height = `${displayHeight}px`
  }

  const images = [img1, img2]
  images.forEach(img => {
    img.style.width = `${displayWidth}px`
    img.style.height = `${displayHeight}px`
  })
}

// 监听窗口大小变化，重新计算图片尺寸
let resizeTimeout: number
onMounted(() => {
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout)
    resizeTimeout = window.setTimeout(handleImageLoad, 100)
  })
})
</script>

<style scoped>
.image-processor {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  gap: 1.5rem;
}

.image-container {
  flex: 1;
  min-height: 0;
  border-radius: 8px;
  overflow: hidden;
  background-color: #f5f5f5;
}

/* 上传区域样式 */
.upload-zone {
  width: 100%;
  height: 100%;
  border: 2px dashed #ccc;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
}

.upload-zone:hover {
  border-color: #666;
  background-color: #eee;
}

.upload-placeholder {
  color: #666;
  font-size: 1.1rem;
}

.upload-zone img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

/* 对比区域样式 */
.comparison-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
}

.comparison-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.comparison-content {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.comparison-slider {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  --divider-width: 2px;
  --divider-color: #666;
  --handle-color: #666;
}

.image-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
  overflow: hidden;
}

.comparison-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  transition: width 0.3s ease, height 0.3s ease;
}

/* 控制面板样式 */
.control-panel {
  display: flex;
  gap: 1rem;
  padding: 0.5rem 0;
}

.control-button {
  padding: 0.5rem 1rem;
  border: 1px solid #ccc;
  border-radius: 4px;
  background-color: white;
  cursor: pointer;
  transition: all 0.3s ease;
}

.control-button:hover:not(:disabled) {
  background-color: #f5f5f5;
  border-color: #666;
}

.control-button.primary {
  background-color: #4CAF50;
  color: white;
  border-color: #4CAF50;
}

.control-button.primary:hover:not(:disabled) {
  background-color: #45a049;
}

.control-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.processing-status {
  text-align: center;
  color: #666;
  font-size: 0.9rem;
}

.image-preview-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.image-preview-wrapper img {
  transition: width 0.3s ease, height 0.3s ease;
  object-fit: contain;
}
</style> 
