use image::DynamicImage;
use ndarray::Array4;
use ort::session::Session;
use ort::value::Value;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::ImageProcessingError;
use crate::operation::{ParameterizedImageOperator, ProcessOutput};
use crate::runtime::{RuntimeOptions, build_session};
use crate::segment::dilate_mask;

pub const LAMA_SIZE: usize = 512;

/// LaMa inpainting. Model input is fixed 512x512; the image is processed at
/// that resolution and the inpainted region is blended back into the original
/// at full resolution using the mask (crop-around-mask strategy would be a
/// future improvement for very large images).
pub struct Eraser {
    session: Session,
}

#[derive(Clone, Debug)]
pub struct EraseParams {
    pub mask_path: PathBuf,
    pub dilate: u32,
}

/// Agent-facing erase operator that reuses the LaMa session across requests.
pub struct EraseOperator {
    eraser: Mutex<Eraser>,
}

impl EraseOperator {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        Self::new_with_runtime(model_path, RuntimeOptions::default())
    }

    pub fn new_with_runtime(
        model_path: &str,
        runtime: RuntimeOptions,
    ) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            eraser: Mutex::new(Eraser::new_with_runtime(model_path, runtime)?),
        })
    }

    pub fn process_image(
        &self,
        image: &DynamicImage,
        mask: &image::GrayImage,
        dilate: u32,
    ) -> Result<ProcessOutput, ImageProcessingError> {
        let mask = if dilate > 0 {
            dilate_mask(mask, dilate)
        } else {
            mask.clone()
        };
        let mut eraser = self.eraser.lock().map_err(|_| {
            ImageProcessingError::Processing("erase operator lock poisoned".to_string())
        })?;
        eraser.erase(image, &mask).map(ProcessOutput::image)
    }
}

impl ParameterizedImageOperator<EraseParams> for EraseOperator {
    fn process_with(
        &self,
        input_path: &str,
        params: &EraseParams,
    ) -> Result<ProcessOutput, ImageProcessingError> {
        let image = image::open(input_path)?;
        let mask = image::open(&params.mask_path)?.to_luma8();
        self.process_image(&image, &mask, params.dilate)
    }
}

impl Eraser {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        Self::new_with_runtime(model_path, RuntimeOptions::default())
    }

    pub fn new_with_runtime(
        model_path: &str,
        runtime: RuntimeOptions,
    ) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            session: build_session(model_path, &runtime)?,
        })
    }

    /// Erase the masked region (mask: 255 = erase) and return the full-resolution result.
    pub fn erase(
        &mut self,
        image: &DynamicImage,
        mask: &image::GrayImage,
    ) -> Result<DynamicImage, ImageProcessingError> {
        if mask.dimensions() != (image.width(), image.height()) {
            return Err(ImageProcessingError::Processing(
                "mask dimensions must match image dimensions".to_string(),
            ));
        }

        let size = LAMA_SIZE as u32;
        let resized = image
            .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
            .to_rgb8();
        let mask_small =
            image::imageops::resize(mask, size, size, image::imageops::FilterType::Nearest);

        let img_tensor =
            ndarray::Array::from_shape_fn((1, 3, LAMA_SIZE, LAMA_SIZE), |(_, c, y, x)| {
                resized.get_pixel(x as u32, y as u32)[c] as f32 / 255.0
            });
        let mask_tensor =
            ndarray::Array::from_shape_fn((1, 1, LAMA_SIZE, LAMA_SIZE), |(_, _, y, x)| {
                if mask_small.get_pixel(x as u32, y as u32)[0] > 127 {
                    1.0f32
                } else {
                    0.0
                }
            });

        let outputs = self.session.run(ort::inputs![
            "image" => Value::from_array(img_tensor)?,
            "mask" => Value::from_array(mask_tensor)?,
        ])?;

        let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
        let h = shape[2] as usize;
        let w = shape[3] as usize;
        let output = Array4::from_shape_vec((1, 3, h, w), data.to_vec())
            .map_err(|e| ImageProcessingError::Processing(e.to_string()))?;

        // Carve export outputs 0-255 range already
        let mut inpainted = image::RgbImage::new(w as u32, h as u32);
        for y in 0..h {
            for x in 0..w {
                let r = output[[0, 0, y, x]].clamp(0.0, 255.0) as u8;
                let g = output[[0, 1, y, x]].clamp(0.0, 255.0) as u8;
                let b = output[[0, 2, y, x]].clamp(0.0, 255.0) as u8;
                inpainted.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }

        // Blend the inpainted (512x512) region back into the full-resolution
        // original: outside the mask keep original pixels, inside use the
        // upscaled inpainted result.
        let inpainted_full = image::imageops::resize(
            &inpainted,
            image.width(),
            image.height(),
            image::imageops::FilterType::Lanczos3,
        );
        let mut result = image.to_rgb8();
        for (x, y, pixel) in result.enumerate_pixels_mut() {
            if mask.get_pixel(x, y)[0] > 127 {
                *pixel = *inpainted_full.get_pixel(x, y);
            }
        }

        Ok(DynamicImage::ImageRgb8(result))
    }
}
