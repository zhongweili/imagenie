use crate::ImageProcessingError;
use crate::model::ImageModel;
use crate::operation::{ImageOperator, ProcessOutput};
use crate::processor::ModelProcessor;
use crate::runtime::RuntimeOptions;
use crate::types::BackgroundRemovalParams;
use image::DynamicImage;

/// A complete background-removal operator: inference plus alpha compositing.
pub struct BackgroundRemover<M>
where
    M: ImageModel<Params = BackgroundRemovalParams> + Send + Sync,
{
    processor: ModelProcessor<M>,
    params: BackgroundRemovalParams,
}

impl<M> BackgroundRemover<M>
where
    M: ImageModel<Params = BackgroundRemovalParams> + Send + Sync,
{
    pub fn new(
        model_path: &str,
        params: BackgroundRemovalParams,
    ) -> Result<Self, ImageProcessingError> {
        Self::new_with_runtime(model_path, params, RuntimeOptions::default())
    }

    pub fn new_with_runtime(
        model_path: &str,
        params: BackgroundRemovalParams,
        runtime: RuntimeOptions,
    ) -> Result<Self, ImageProcessingError> {
        Ok(Self {
            processor: ModelProcessor::new_with_runtime(model_path, runtime)?,
            params,
        })
    }

    pub fn mask(&self, input_path: &str) -> Result<DynamicImage, ImageProcessingError> {
        self.processor.process_single(input_path, &self.params)
    }
}

impl<M> ImageOperator for BackgroundRemover<M>
where
    M: ImageModel<Params = BackgroundRemovalParams> + Send + Sync,
{
    fn process(&self, input_path: &str) -> Result<ProcessOutput, ImageProcessingError> {
        let original = image::open(input_path)?;
        let mask = self.mask(input_path)?;
        Ok(ProcessOutput::image(composite_alpha(&original, &mask)))
    }
}

/// Apply a model-produced alpha mask while preserving the original RGB pixels.
pub fn composite_alpha(original: &DynamicImage, mask: &DynamicImage) -> DynamicImage {
    let mask = mask.resize_exact(
        original.width(),
        original.height(),
        image::imageops::FilterType::Triangle,
    );
    let original_rgba = original.to_rgba8();
    let mask_rgba = mask.to_rgba8();
    let mut output = image::RgbaImage::new(original.width(), original.height());

    for (x, y, pixel) in output.enumerate_pixels_mut() {
        let original = original_rgba.get_pixel(x, y);
        let mask = mask_rgba.get_pixel(x, y);
        *pixel = image::Rgba([original[0], original[1], original[2], mask[3]]);
    }

    DynamicImage::ImageRgba8(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_preserves_rgb_and_replaces_alpha() {
        let original = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([10, 20, 30, 99]),
        ));
        let mask = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([0, 0, 0, 123]),
        ));

        let output = composite_alpha(&original, &mask).to_rgba8();

        assert_eq!(output.dimensions(), (2, 2));
        for pixel in output.pixels() {
            assert_eq!(*pixel, image::Rgba([10, 20, 30, 123]));
        }
    }

    #[test]
    fn composite_uses_mask_alpha_not_mask_color() {
        let original =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, image::Rgb([1, 2, 3])));
        let mask = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([200, 100, 50, 7]),
        ));

        assert_eq!(
            composite_alpha(&original, &mask).to_rgba8().get_pixel(0, 0),
            &image::Rgba([1, 2, 3, 7])
        );
    }
}
