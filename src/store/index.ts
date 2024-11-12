// src/store/index.ts
import { defineStore } from 'pinia'

interface State {
  inputPath: string | null
  outputDir: string | null
  inputImage: string | null
  settings: {
    scale: number
    strength: number
  }
}

export const useStore = defineStore('main', {
  state: (): State => ({
    inputPath: null,
    outputDir: null,
    inputImage: null,
    settings: {
      scale: 2,
      strength: 50,
    }
  }),

  actions: {
    resetState() {
      this.inputPath = null
      this.outputDir = null
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
      this.outputDir = dir
    }
  }
})
