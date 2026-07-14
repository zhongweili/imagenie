use image::DynamicImage;
use ndarray::{Array1, Array3, Array4};
use ort::session::Session;
use ort::value::Value;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::error::ImageProcessingError;
use crate::operation::{ParameterizedImageOperator, ProcessMetrics, ProcessOutput};
use crate::runtime::{RuntimeOptions, build_session};

pub const SAM_SIZE: usize = 1024;

const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// SAM 2.1 segmenter. The image embedding is computed once (~585ms CPU) and
/// cached so each additional point prompt only costs a decoder run (~12ms).
pub struct Segmenter {
    encoder: Session,
    decoder: Session,
    embedding: Option<Embedding>,
}

struct Embedding {
    image_embed: Array4<f32>,
    high_res_feats_0: Array4<f32>,
    high_res_feats_1: Array4<f32>,
    original_width: u32,
    original_height: u32,
}

#[derive(Debug, Clone)]
pub struct SegmentResult {
    /// Binary mask at original image resolution (255 = selected).
    pub mask: image::GrayImage,
    /// Predicted IoU of the best mask — confidence signal. Low iou (< ~0.5)
    /// together with a tiny pixel count usually means the point missed the object.
    pub iou: f32,
    /// Number of selected pixels in the SAM-resolution (1024x1024) mask.
    pub mask_pixels: u64,
}

/// Point or box prompt in original-image pixel coordinates.
#[derive(Debug, Clone, Copy)]
pub enum Prompt {
    Point { x: f32, y: f32 },
    Box { x1: f32, y1: f32, x2: f32, y2: f32 },
}

#[derive(Debug, PartialEq, Eq)]
struct SourceKey {
    path: String,
    len: u64,
    modified: Option<SystemTime>,
}

impl SourceKey {
    fn read(path: &str) -> Result<Self, ImageProcessingError> {
        let metadata = std::fs::metadata(path)?;
        Ok(Self {
            path: path.to_string(),
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

struct SegmentState {
    segmenter: Segmenter,
    source: Option<SourceKey>,
}

/// Agent-facing segmentation operator that reuses model sessions and caches an
/// embedding while the input path, size, and modification time remain stable.
pub struct SegmentOperator {
    state: Mutex<SegmentState>,
}

impl SegmentOperator {
    pub fn new(encoder_path: &str, decoder_path: &str) -> Result<Self, ImageProcessingError> {
        Self::new_with_runtime(encoder_path, decoder_path, RuntimeOptions::default())
    }

    pub fn new_with_runtime(
        encoder_path: &str,
        decoder_path: &str,
        runtime: RuntimeOptions,
    ) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            state: Mutex::new(SegmentState {
                segmenter: Segmenter::new_with_runtime(encoder_path, decoder_path, runtime)?,
                source: None,
            }),
        })
    }

    pub fn process_image(
        &self,
        input_path: &str,
        image: &DynamicImage,
        prompt: Prompt,
    ) -> Result<ProcessOutput, ImageProcessingError> {
        let source = SourceKey::read(input_path)?;
        let mut state = self.state.lock().map_err(|_| {
            ImageProcessingError::Processing("segment operator lock poisoned".to_string())
        })?;

        if state.source.as_ref() != Some(&source) {
            state.segmenter.encode(image)?;
            state.source = Some(source);
        }
        let result = state.segmenter.decode(prompt)?;

        Ok(ProcessOutput {
            image: DynamicImage::ImageLuma8(result.mask),
            metrics: ProcessMetrics {
                iou: Some(result.iou),
                mask_pixels: Some(result.mask_pixels),
            },
            warnings: Vec::new(),
        })
    }
}

impl ParameterizedImageOperator<Prompt> for SegmentOperator {
    fn process_with(
        &self,
        input_path: &str,
        prompt: &Prompt,
    ) -> Result<ProcessOutput, ImageProcessingError> {
        let image = image::open(input_path)?;
        self.process_image(input_path, &image, *prompt)
    }
}

impl Segmenter {
    pub fn new(encoder_path: &str, decoder_path: &str) -> Result<Self, ImageProcessingError> {
        Self::new_with_runtime(encoder_path, decoder_path, RuntimeOptions::default())
    }

    pub fn new_with_runtime(
        encoder_path: &str,
        decoder_path: &str,
        runtime: RuntimeOptions,
    ) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            encoder: build_session(encoder_path, &runtime)?,
            decoder: build_session(decoder_path, &runtime)?,
            embedding: None,
        })
    }

    /// Encode an image, caching the embedding for subsequent prompts.
    pub fn encode(&mut self, image: &DynamicImage) -> Result<(), ImageProcessingError> {
        let resized = image.resize_exact(
            SAM_SIZE as u32,
            SAM_SIZE as u32,
            image::imageops::FilterType::Lanczos3,
        );
        let rgb = resized.to_rgb8();
        let tensor = ndarray::Array::from_shape_fn((1, 3, SAM_SIZE, SAM_SIZE), |(_, c, y, x)| {
            let pixel = rgb.get_pixel(x as u32, y as u32);
            (pixel[c] as f32 / 255.0 - IMAGENET_MEAN[c]) / IMAGENET_STD[c]
        });

        let outputs = self
            .encoder
            .run(ort::inputs!["image" => Value::from_array(tensor)?])?;

        let extract = |name: &str| -> Result<Array4<f32>, ImageProcessingError> {
            let output = outputs.get(name).ok_or_else(|| {
                ImageProcessingError::Processing(format!("missing encoder output {name}"))
            })?;
            let (shape, data) = output.try_extract_tensor::<f32>()?;
            let dims: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
            Array4::from_shape_vec((dims[0], dims[1], dims[2], dims[3]), data.to_vec())
                .map_err(|e| ImageProcessingError::Processing(e.to_string()))
        };

        self.embedding = Some(Embedding {
            image_embed: extract("image_embed")?,
            high_res_feats_0: extract("high_res_feats_0")?,
            high_res_feats_1: extract("high_res_feats_1")?,
            original_width: image.width(),
            original_height: image.height(),
        });
        Ok(())
    }

    /// Decode a prompt against the cached embedding. Picks the best of the
    /// candidate masks by predicted IoU.
    pub fn decode(&mut self, prompt: Prompt) -> Result<SegmentResult, ImageProcessingError> {
        let emb = self.embedding.as_ref().ok_or_else(|| {
            ImageProcessingError::Processing("call encode() before decode()".to_string())
        })?;

        let sx = SAM_SIZE as f32 / emb.original_width as f32;
        let sy = SAM_SIZE as f32 / emb.original_height as f32;

        let (coords, labels): (Vec<[f32; 2]>, Vec<f32>) = match prompt {
            Prompt::Point { x, y } => (vec![[x * sx, y * sy]], vec![1.0]),
            // Box prompt: label 2 = top-left corner, 3 = bottom-right corner
            Prompt::Box { x1, y1, x2, y2 } => {
                (vec![[x1 * sx, y1 * sy], [x2 * sx, y2 * sy]], vec![2.0, 3.0])
            }
        };

        let n = coords.len();
        let point_coords =
            Array3::from_shape_vec((1, n, 2), coords.into_iter().flatten().collect())
                .map_err(|e| ImageProcessingError::Processing(e.to_string()))?;
        let point_labels = ndarray::Array2::from_shape_vec((1, n), labels)
            .map_err(|e| ImageProcessingError::Processing(e.to_string()))?;

        let outputs = self.decoder.run(ort::inputs![
            "image_embed" => Value::from_array(emb.image_embed.clone())?,
            "high_res_feats_0" => Value::from_array(emb.high_res_feats_0.clone())?,
            "high_res_feats_1" => Value::from_array(emb.high_res_feats_1.clone())?,
            "point_coords" => Value::from_array(point_coords)?,
            "point_labels" => Value::from_array(point_labels)?,
            "mask_input" => Value::from_array(Array4::<f32>::zeros((1, 1, 256, 256)))?,
            "has_mask_input" => Value::from_array(Array1::<f32>::zeros(1))?,
        ])?;

        let (mask_shape, masks) = outputs[0].try_extract_tensor::<f32>()?;
        let (_, ious) = outputs[1].try_extract_tensor::<f32>()?;

        let best = ious
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let iou = ious[best];

        let mh = mask_shape[2] as usize;
        let mw = mask_shape[3] as usize;
        let mask_data = &masks[best * mh * mw..(best + 1) * mh * mw];

        let mut mask_pixels = 0u64;
        let mut low_res = image::GrayImage::new(mw as u32, mh as u32);
        for y in 0..mh {
            for x in 0..mw {
                if mask_data[y * mw + x] > 0.0 {
                    low_res.put_pixel(x as u32, y as u32, image::Luma([255]));
                    mask_pixels += 1;
                }
            }
        }

        // Resize to original resolution with nearest neighbor to keep it binary
        let mask = image::imageops::resize(
            &low_res,
            emb.original_width,
            emb.original_height,
            image::imageops::FilterType::Nearest,
        );

        Ok(SegmentResult {
            mask,
            iou,
            mask_pixels,
        })
    }
}

/// Dilate a binary mask by radius `r` (used before LaMa to avoid halo edges).
pub fn dilate_mask(mask: &image::GrayImage, r: u32) -> image::GrayImage {
    let (w, h) = mask.dimensions();
    let mut out = image::GrayImage::new(w, h);
    let r = r as i64;

    // Two-pass separable max filter
    let mut tmp = image::GrayImage::new(w, h);
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let mut v = 0u8;
            for dx in -r..=r {
                let nx = x + dx;
                if nx >= 0 && nx < w as i64 {
                    v = v.max(mask.get_pixel(nx as u32, y as u32)[0]);
                    if v == 255 {
                        break;
                    }
                }
            }
            tmp.put_pixel(x as u32, y as u32, image::Luma([v]));
        }
    }
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let mut v = 0u8;
            for dy in -r..=r {
                let ny = y + dy;
                if ny >= 0 && ny < h as i64 {
                    v = v.max(tmp.get_pixel(x as u32, ny as u32)[0]);
                    if v == 255 {
                        break;
                    }
                }
            }
            out.put_pixel(x as u32, y as u32, image::Luma([v]));
        }
    }
    out
}
