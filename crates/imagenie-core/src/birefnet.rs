use image::DynamicImage;
use ort::session::Session;
use std::marker::PhantomData;

use crate::error::ImageProcessingError;
use crate::model::ImageModel;
use crate::runtime::{RuntimeOptions, build_session};
use crate::types::{BackgroundRemovalParams, TensorInput, TensorOutput};

const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// BiRefNet-general-lite: higher-quality background removal.
/// ImageNet normalization on input; output is logits, so postprocess applies sigmoid.
pub struct BiRefNetModel<T = f32>(PhantomData<T>);

impl ImageModel for BiRefNetModel<f32> {
    type Params = BackgroundRemovalParams;
    type InputType = f32;
    type OutputType = f32;

    fn load_session(
        model_path: &str,
        runtime: &RuntimeOptions,
    ) -> Result<Session, ImageProcessingError> {
        build_session(model_path, runtime)
    }

    fn preprocess(
        image_path: &str,
        params: &mut Self::Params,
    ) -> Result<TensorInput<Self::InputType>, ImageProcessingError> {
        let image = image::open(image_path)?;

        params.original_width = Some(image.width());
        params.original_height = Some(image.height());

        let resized = image.resize_exact(
            params.model_width as u32,
            params.model_height as u32,
            image::imageops::FilterType::Lanczos3,
        );

        let rgb_image = resized.to_rgb8();
        let tensor = ndarray::Array::from_shape_fn(
            (1, 3, params.model_height, params.model_width),
            |(_, c, y, x)| {
                let pixel = rgb_image.get_pixel(x as u32, y as u32);
                (pixel[c] as f32 / 255.0 - IMAGENET_MEAN[c]) / IMAGENET_STD[c]
            },
        );

        Ok(tensor)
    }

    fn postprocess(
        output: &TensorOutput<Self::OutputType>,
        _params: &Self::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        let (_, _, height, width) = output.dim();
        let mut img_buffer = image::RgbaImage::new(width as u32, height as u32);

        for y in 0..height {
            for x in 0..width {
                let logit = output[[0, 0, y, x]];
                let alpha = (1.0 / (1.0 + (-logit).exp()) * 255.0).clamp(0.0, 255.0) as u8;
                img_buffer.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, alpha]));
            }
        }

        Ok(DynamicImage::ImageRgba8(img_buffer))
    }

    fn process(
        session: &mut Session,
        input: &TensorInput<Self::InputType>,
    ) -> Result<TensorOutput<Self::OutputType>, ImageProcessingError> {
        let input_value = ort::value::Value::from_array(input.clone())?;
        let outputs = session.run(ort::inputs![input_value])?;
        let output = outputs
            .values()
            .next()
            .ok_or_else(|| ImageProcessingError::Processing("No output from model".to_string()))?;

        let (shape, data) = output.try_extract_tensor::<f32>()?;
        let dims: Vec<usize> = shape.iter().map(|&d| d as usize).collect();

        ndarray::Array4::from_shape_vec((1, 1, dims[2], dims[3]), data.to_vec())
            .map_err(|e| ImageProcessingError::Processing(e.to_string()))
    }
}
