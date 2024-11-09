// src/store/index.ts
import { defineStore } from 'pinia'

interface State {
  inputPath: string | null
  outputPath: string | null
  inputImage: string | null
  settings: {
    scale: number
    strength: number
  }
}

export const useStore = defineStore('main', {
  state: (): State => ({
    inputPath: null,
    outputPath: null,
    inputImage: null,
    settings: {
      scale: 2,
      strength: 50,
    }
  }),

  actions: {
    resetState() {
      this.inputPath = null
      this.outputPath = null
      this.inputImage = null
      this.settings = {
        scale: 2,
        strength: 50,
      }
    },

    updateSettings(payload: Partial<State['settings']>) {
      this.settings = { ...this.settings, ...payload }
    },

    async setInputFile(file: string) {
      this.inputPath = file
    },

    async setInputImage(image: string) {
      this.inputImage = image
    },

    async setOutputDir(dir: string) {
      this.outputPath = dir
    },

    async selectOutputPath() {
      // 选择输出路径...
    },

    async startProcessing() {
      // 开始处理图片...
    }
  }
})
