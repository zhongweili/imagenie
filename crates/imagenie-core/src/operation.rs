use crate::ImageProcessingError;
use crate::model::ImageModel;
use crate::processor::ModelProcessor;
use image::DynamicImage;
use rayon::prelude::*;
use serde::Serialize;

/// Task-specific signals that help an agent judge whether an output is useful.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct ProcessMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iou: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_pixels: Option<u64>,
}

/// Common result returned by every configured image operator.
pub struct ProcessOutput {
    pub image: DynamicImage,
    pub metrics: ProcessMetrics,
    pub warnings: Vec<String>,
}

impl ProcessOutput {
    pub fn image(image: DynamicImage) -> Self {
        Self {
            image,
            metrics: ProcessMetrics::default(),
            warnings: Vec::new(),
        }
    }
}

/// Uniform core entry point used by the CLI, desktop app, and future adapters.
///
/// Operators are configured when constructed (model, quality tier, and other
/// reusable parameters), then process one or more input paths through the same
/// result contract. Batch processing preserves input order and reports an
/// independent result for every input.
pub trait ImageOperator: Send + Sync {
    fn process(&self, input_path: &str) -> Result<ProcessOutput, ImageProcessingError>;

    fn process_batch(
        &self,
        input_paths: &[String],
    ) -> Vec<Result<ProcessOutput, ImageProcessingError>> {
        input_paths
            .par_iter()
            .map(|path| self.process(path))
            .collect()
    }
}

/// Uniform entry point for operators with request-specific parameters.
///
/// The operator owns reusable model sessions while the parameter value carries
/// per-call state such as a segmentation prompt or erase mask path.
pub trait ParameterizedImageOperator<P>: Send + Sync {
    fn process_with(
        &self,
        input_path: &str,
        params: &P,
    ) -> Result<ProcessOutput, ImageProcessingError>;
}

/// Adapter for models whose postprocessed image is already the final artifact.
///
/// Operators that need composition or task-specific metrics should provide a
/// dedicated `ImageOperator` implementation instead.
pub struct ModelOperator<M>
where
    M: ImageModel + Send + Sync,
{
    processor: ModelProcessor<M>,
    params: M::Params,
}

impl<M> ModelOperator<M>
where
    M: ImageModel + Send + Sync,
{
    pub fn new(model_path: &str, params: M::Params) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            processor: ModelProcessor::new(model_path)?,
            params,
        })
    }
}

impl<M> ImageOperator for ModelOperator<M>
where
    M: ImageModel + Send + Sync,
{
    fn process(&self, input_path: &str) -> Result<ProcessOutput, ImageProcessingError> {
        self.processor
            .process_single(input_path, &self.params)
            .map(ProcessOutput::image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureOperator;

    impl ImageOperator for FixtureOperator {
        fn process(&self, input_path: &str) -> Result<ProcessOutput, ImageProcessingError> {
            if input_path == "bad" {
                return Err(ImageProcessingError::Processing("fixture failure".into()));
            }
            Ok(ProcessOutput::image(DynamicImage::new_rgba8(1, 1)))
        }
    }

    #[test]
    fn batch_preserves_input_order_and_independent_failures() {
        let results = FixtureOperator.process_batch(&[
            "first".to_string(),
            "bad".to_string(),
            "last".to_string(),
        ]);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
}
