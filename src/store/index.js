// src/store/index.js
import { defineStore } from 'pinia';

export const useStore = defineStore('main', {
  state: () => ({
    activeFunction: 'compress',
    files: [],
    functionSettings: {
      compress: { quality: 80 },
      repair: { intensity: 5 },
      'remove-bg': { preserveDetails: true },
    },
  }),
  actions: {
    setActiveFunction(name) {
      this.activeFunction = name;
    },
    addFiles(newFiles) {
      this.files.push(...newFiles);
    },
    clearFiles() {
      this.files = [];
    },
  },
});
