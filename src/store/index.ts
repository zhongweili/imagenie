// src/store/index.js
import { defineStore } from 'pinia';

type FunctionSettings = {
  compress: { quality: number };
  repair: { intensity: number };
  'removebg': { preserveDetails: boolean };
}

export const useStore = defineStore('main', {
  state: () => ({
    activeFunction: 'compress' as keyof FunctionSettings,
    files: [] as File[],
    functionSettings: {
      compress: { quality: 80 },
      repair: { intensity: 5 },
      'removebg': { preserveDetails: true },
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
