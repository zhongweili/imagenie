// src/store/index.js
import { defineStore } from 'pinia';

type FunctionSettings = {
  upscaling: { quality: number };
  resizing: { quality: number };
  restoration: { intensity: number };
  'remove-background': { preserveDetails: boolean };
}

export const useStore = defineStore('main', {
  state: () => ({
    activeFunction: 'upscaling' as keyof FunctionSettings,
    files: [] as File[],
    functionSettings: {
      upscaling: { quality: 80 },
      resizing: { quality: 80 },
      restoration: { intensity: 5 },
      'remove-background': { preserveDetails: true },
    } as FunctionSettings,
  }),
  actions: {
    setActiveFunction(name: keyof FunctionSettings) {
      this.activeFunction = name;
    },
    addFiles(newFiles: any) {
      this.files = [...this.files, ...newFiles];
    },
    clearFiles() {
      this.files = [];
    },
  },
});
