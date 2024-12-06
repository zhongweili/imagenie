<template>
  <div class="image-processor">
    <div class="image-container">
      <!-- Upload Area -->
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
          <span>{{ t('imageProcessor.dropHint') }}</span>
        </div>
      </div>

      <!-- Comparison Area -->
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
        {{ t('imageProcessor.selectInput') }}
      </button>
      <button
        class="control-button"
        @click="selectOutputDir"
      >
        {{ t('imageProcessor.selectOutput') }}
      </button>
      <button
        class="control-button primary"
        @click="startProcessing"
        :disabled="!canProcess || isProcessing"
      >
        {{ isProcessing ? t('imageProcessor.processing') : t('imageProcessor.startProcess') }}
      </button>
    </div>

    <div v-if="isProcessing" class="processing-status">
      {{ t('imageProcessor.processingStatus') }}
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
import { useI18n } from 'vue-i18n'

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
    processedImageUrl.value = ''
    store.setInputFile(selected);
    const assetUrl = convertFileSrc(selected);
    store.setInputImage(assetUrl);
    // set outputdir as the same directory as input file
    const outputDir = await join(selected, '..');
    store.setOutputDir(outputDir);
  }
}

// TODO: need to switch to tauri DRAG_DROP event
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

interface Props {
  mode: 'upscaling' | 'restoration' | 'remove-background' | 'resizing'
}
const props = defineProps<Props>()

const startProcessing = async () => {
  isProcessing.value = true
  console.log(store.inputPath, store.outputDir)
  try {
    const dimensions = await invoke('check_image_dimensions', {
      inputPath: store.inputPath
    }) as [number, number]

    console.log(dimensions)
    const maxDimension = 2048
    if (dimensions[0] > maxDimension || dimensions[1] > maxDimension) {
      enqueueNotification(
        t('imageProcessor.dimensionError'),
        t('imageProcessor.dimensionErrorDesc', { maxDimension }),
      )
      return
    }

    let command
    switch (props.mode) {
      case 'upscaling':
        command = 'upscale_image'
        break
      case 'remove-background':
        command = 'background_removal'
        break
      case 'restoration':
        command = 'face_restoration'
        break
      default:
        throw new Error(`Unsupported mode: ${props.mode}`)
    }
    const outputPath = await invoke(command, {
      inputPath: store.inputPath,
      outputDir: store.outputDir
    })
    processedImageUrl.value = convertFileSrc(outputPath as string)
    enqueueNotification(
      t('imageProcessor.processingCompleted'),
      t('imageProcessor.processingCompletedDesc')
    )
  } catch (error) {
    console.error('Processing failed:', error)
    enqueueNotification(
      t('imageProcessor.processingError'),
      String(error),
    )
  } finally {
    isProcessing.value = false
    store.setOutputDir(undefined)
  }
}

const originalImage = ref<HTMLImageElement | null>(null)
const processedImage = ref<HTMLImageElement | null>(null)
const comparisonWrapper = ref<HTMLElement | null>(null)

const previewImage = ref<HTMLImageElement | null>(null)

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
    displayHeight = containerHeight
    displayWidth = displayHeight * imgRatio
  } else {
    displayWidth = containerWidth
    displayHeight = displayWidth / imgRatio
  }

  img.style.width = `${displayWidth}px`
  img.style.height = `${displayHeight}px`
}

const handleImageLoad = () => {
  if (!originalImage.value || !processedImage.value || !comparisonWrapper.value) return

  const containerWidth = comparisonWrapper.value.clientWidth
  const containerHeight = comparisonWrapper.value.clientHeight

  const img1 = originalImage.value
  const img2 = processedImage.value

  // Make sure that both pictures have been loaded
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

// Listen for window size changes and recalculate image size
let resizeTimeout: number
onMounted(() => {
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout)
    resizeTimeout = window.setTimeout(handleImageLoad, 100)
  })
})

const { t } = useI18n()
</script>

<style scoped>
.image-processor {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  gap: 1.5rem;
  position: relative;
}

.image-container {
  flex: 1;
  min-height: 0;
  border-radius: 8px;
  overflow: hidden;
  background-color: #f5f5f5;
}

.upload-zone {
  width: 100%;
  height: 100%;
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
  --default-handle-color: #666;
  --default-handle-width: 40px;
  --default-divider-width: 2px;
  --default-divider-color: #666;
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
.control-panel {
  display: flex;
  gap: 1rem;
  padding: 0.5rem 0;
  align-items: center; /* Added to vertically align items */
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
  color: #666;
  font-size: 0.9rem;
  position: absolute;
  bottom: 1.5rem;
  right: 1.5rem;
  margin-left: 0;
  background-color: rgba(255, 255, 255, 0.9);
  padding: 0.5rem 1rem;
  border-radius: 4px;
  z-index: 10;
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
