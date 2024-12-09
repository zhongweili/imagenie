export interface ModelRelease {
  version: string;
  url: string;
  fileName: string;
}

export const GITHUB_REPO = {
  owner: 'zhongweili',
  repo: 'imagenie',
};

export const FACE_RESTORATION_MODEL = 'face_restoration.onnx';
export const IMAGE_UPSCALING_MODEL = 'image_upscaling.onnx';
export const BACKGROUND_REMOVAL_MODEL = 'background_removal.onnx';
