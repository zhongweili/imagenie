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
    input_path: '',
    output_dir: '',
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
    setInputPath(path: string) {
      this.input_path = path;
    },
    setOutputDir(dir: string) {
      this.output_dir = dir;
    }
  },
});
