use once_cell::sync::OnceCell;
use ort::{GraphOptimizationLevel, Session};
use rayon::prelude::*;

use crate::image_backup::{ImageParams, ImageProcessingError};

pub struct UpscalingProcessor {
    session: Session,
}

impl UpscalingProcessor {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;
        Ok(Self { session })
    }
}

pub struct RestorationProcessor {
    session: Session,
}

impl RestorationProcessor {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;
        Ok(Self { session })
    }
}

static UPSCALE_PROCESSOR: OnceCell<UpscalingProcessor> = OnceCell::new();
static RESTORATION_PROCESSOR: OnceCell<RestorationProcessor> = OnceCell::new();

pub fn get_upscaling_processor() -> &'static UpscalingProcessor {
    UPSCALE_PROCESSOR
        .get_or_init(|| UpscalingProcessor::new("assets/RealESRGAN_x2_fp16.onnx").unwrap())
}

pub fn get_restoration_processor() -> &'static RestorationProcessor {
    RESTORATION_PROCESSOR
        .get_or_init(|| RestorationProcessor::new("assets/RealESRGAN_x2_fp16.onnx").unwrap())
}
