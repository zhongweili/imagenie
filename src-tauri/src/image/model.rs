use image::DynamicImage;
use ort::{inputs, GraphOptimizationLevel, Session, Value};
use std::marker::PhantomData;

use crate::image::error::ImageProcessingError;
use crate::image::tensor::{image_to_tensor, tensor_to_image};
use crate::image::types::{
    FaceRestorationParams, NumericType, TensorInput, TensorOutput, UpscalingParams,
};

use super::types::BackgroundRemovalParams;

pub trait ImageModel {
    type Params: Sync + Clone;
    type InputType: NumericType;
    type OutputType: NumericType;

    fn load_session(model_path: &str) -> Result<Session, ImageProcessingError>;
    fn preprocess(
        image_path: &str,
        params: &mut Self::Params,
    ) -> Result<TensorInput<Self::InputType>, ImageProcessingError>;
    fn postprocess(
        output: &TensorOutput<Self::OutputType>,
        params: &Self::Params,
    ) -> Result<DynamicImage, ImageProcessingError>;
    fn process(
        session: &Session,
        input: &TensorInput<Self::InputType>,
    ) -> Result<TensorOutput<Self::OutputType>, ImageProcessingError>;
}

pub struct UpscalingModel<T = half::f16>(PhantomData<T>);

impl ImageModel for UpscalingModel<half::f16> {
    type Params = UpscalingParams;
    type InputType = half::f16;
    type OutputType = half::f16;

    fn load_session(model_path: &str) -> Result<Session, ImageProcessingError> {
        Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)
            .map_err(|e| ImageProcessingError::Ort(e.to_string()))
    }

    fn preprocess(
        image_path: &str,
        _params: &mut Self::Params,
    ) -> Result<TensorInput<Self::InputType>, ImageProcessingError> {
        let image = image::open(image_path)?;
        image_to_tensor(&image)
    }

    fn postprocess(
        output: &TensorOutput<Self::OutputType>,
        _params: &Self::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        tensor_to_image(output)
    }

    fn process(
        session: &Session,
        input: &TensorInput<Self::InputType>,
    ) -> Result<TensorOutput<Self::OutputType>, ImageProcessingError> {
        let input_array = input.clone().into_dyn();
        let input_value = Value::from_array(input_array)?;
        let inputs = inputs![input_value]?;

        let outputs = session.run(inputs)?;
        let output = outputs
            .get("output")
            .ok_or_else(|| ImageProcessingError::Processing("No output from model".to_string()))?;

        let output_tensor = output.try_extract_tensor::<Self::InputType>()?;
        let output_shape = output_tensor.shape();

        ndarray::Array4::from_shape_vec(
            (1, 3, output_shape[2], output_shape[3]),
            output_tensor.as_slice().unwrap().to_vec(),
        )
        .map_err(|e| ImageProcessingError::Processing(e.to_string()))
    }
}

pub struct FaceRestorationModel<T = f32>(PhantomData<T>);

impl ImageModel for FaceRestorationModel<f32> {
    type Params = FaceRestorationParams;
    type InputType = f32;
    type OutputType = f32;

    fn load_session(model_path: &str) -> Result<Session, ImageProcessingError> {
        Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)
            .map_err(|e| ImageProcessingError::Ort(e.to_string()))
    }

    fn preprocess(
        image_path: &str,
        params: &mut Self::Params,
    ) -> Result<TensorInput<Self::InputType>, ImageProcessingError> {
        let image = image::open(image_path)?;

        // Save the original size
        params.original_width = Some(image.width());
        params.original_height = Some(image.height());

        // Adjusting to the right size while maintaining the aspect ratio
        let mut resized = if image.width() > image.height() {
            let new_width =
                (params.face_size as f32 * image.width() as f32 / image.height() as f32) as u32;
            image.resize(
                new_width,
                params.face_size,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            let new_height =
                (params.face_size as f32 * image.height() as f32 / image.width() as f32) as u32;
            image.resize(
                params.face_size,
                new_height,
                image::imageops::FilterType::Lanczos3,
            )
        };

        // Center crop to 512x512
        let cropped = if resized.width() > params.face_size || resized.height() > params.face_size {
            let x = (resized.width() - params.face_size) / 2;
            let y = (resized.height() - params.face_size) / 2;
            resized.crop(x, y, params.face_size, params.face_size)
        } else {
            // If the picture is too small, fill in the edge.
            let mut buffer = image::RgbImage::new(params.face_size, params.face_size);
            let x = (params.face_size - resized.width()) / 2;
            let y = (params.face_size - resized.height()) / 2;

            image::imageops::replace(&mut buffer, &resized.to_rgb8(), x.into(), y.into());
            DynamicImage::ImageRgb8(buffer)
        };

        // Convert to RGB and normalize to [-1, 1]
        let rgb_image = cropped.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        let tensor = ndarray::Array::from_shape_fn(
            (1, 3, height as usize, width as usize),
            |(_, c, y, x)| {
                let pixel = rgb_image.get_pixel(x as u32, y as u32);
                (pixel[c] as f32 / 255.0 - 0.5) / 0.5
            },
        );

        Ok(tensor)
    }

    fn postprocess(
        output: &TensorOutput<Self::OutputType>,
        params: &Self::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        let (_, _, h, w) = output.dim();
        let mut img_buffer = image::RgbImage::new(w as u32, h as u32);

        // Convert the pixel value first
        for y in 0..h {
            for x in 0..w {
                let to_u8 = |v: f32| {
                    let normalized = v.clamp(-1.0, 1.0);
                    let denormalized = (normalized + 1.0) / 2.0;
                    (denormalized * 255.0).clamp(0.0, 255.0) as u8
                };

                let r = to_u8(output[[0, 0, y, x]]);
                let g = to_u8(output[[0, 1, y, x]]);
                let b = to_u8(output[[0, 2, y, x]]);
                img_buffer.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }

        let mut result = DynamicImage::ImageRgb8(img_buffer);

        // If there is original size information, adjust the picture back to the original size.
        if let (Some(original_width), Some(original_height)) =
            (params.original_width, params.original_height)
        {
            result = result.resize_exact(
                original_width,
                original_height,
                image::imageops::FilterType::Lanczos3,
            );
        }

        Ok(result)
    }

    fn process(
        session: &Session,
        input: &TensorInput<Self::InputType>,
    ) -> Result<TensorOutput<Self::OutputType>, ImageProcessingError> {
        let input_array = input.clone().into_dyn();
        let input_value = Value::from_array(input_array)?;
        let inputs = inputs![input_value]?;

        let outputs = session.run(inputs)?;
        let output = outputs
            .values()
            .next()
            .ok_or_else(|| ImageProcessingError::Processing("No output from model".to_string()))?;

        let output_tensor = output.try_extract_tensor::<Self::InputType>()?;
        let output_shape = output_tensor.shape();

        ndarray::Array4::from_shape_vec(
            (1, 3, output_shape[2], output_shape[3]),
            output_tensor.as_slice().unwrap().to_vec(),
        )
        .map_err(|e| ImageProcessingError::Processing(e.to_string()))
    }
}

pub struct BackgroundRemovalModel<T = f32>(PhantomData<T>);
impl ImageModel for BackgroundRemovalModel<f32> {
    type Params = BackgroundRemovalParams;
    type InputType = f32;
    type OutputType = f32;

    fn load_session(model_path: &str) -> Result<Session, ImageProcessingError> {
        Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)
            .map_err(|e| ImageProcessingError::Ort(e.to_string()))
    }

    fn preprocess(
        image_path: &str,
        params: &mut Self::Params,
    ) -> Result<TensorInput<Self::InputType>, ImageProcessingError> {
        let image = image::open(image_path)?;

        // Save original dimensions for postprocessing
        params.original_width = Some(image.width());
        params.original_height = Some(image.height());

        // Convert to RGB instead of RGBA
        let input_img = image.into_rgb8();

        // Calculate scaling factor based on model input requirements
        let input_shape = (params.model_height, params.model_width);
        let scaling_factor = f32::min(
            1.0,
            f32::min(
                input_shape.1 as f32 / input_img.width() as f32,
                input_shape.0 as f32 / input_img.height() as f32,
            ),
        );

        params.scaling_factor = Some(scaling_factor);

        // Resize image to model requirements
        let resized_img = image::imageops::resize(
            &input_img,
            input_shape.1 as u32,
            input_shape.0 as u32,
            image::imageops::FilterType::Triangle,
        );

        // Convert to tensor with normalization - now using 3 channels
        let tensor =
            ndarray::Array::from_shape_fn((1, 3, input_shape.0, input_shape.1), |(_, c, y, x)| {
                let pixel = resized_img.get_pixel(x as u32, y as u32);
                let mean = 128.0;
                let std = 256.0;
                (pixel[c] as f32 - mean) / std
            });

        Ok(tensor)
    }

    fn postprocess(
        output: &TensorOutput<Self::OutputType>,
        params: &Self::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        let (_, _, height, width) = output.dim();
        let mut img_buffer = image::RgbaImage::new(width as u32, height as u32);

        // Convert tensor values to alpha channel
        for y in 0..height {
            for x in 0..width {
                let alpha = (output[[0, 0, y, x]] * 255.0).clamp(0.0, 255.0) as u8;
                img_buffer.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, alpha]));
            }
        }

        let mut result = DynamicImage::ImageRgba8(img_buffer);

        // Resize back to original dimensions if they exist
        if let (Some(original_width), Some(original_height), Some(scaling_factor)) = (
            params.original_width,
            params.original_height,
            params.scaling_factor,
        ) {
            let scaled_width = (original_width as f32 * scaling_factor) as u32;
            let scaled_height = (original_height as f32 * scaling_factor) as u32;

            result = result.resize_exact(
                scaled_width,
                scaled_height,
                image::imageops::FilterType::Triangle,
            );
        }

        Ok(result)
    }

    fn process(
        session: &Session,
        input: &TensorInput<Self::InputType>,
    ) -> Result<TensorOutput<Self::OutputType>, ImageProcessingError> {
        let input_array = input.clone().into_dyn();
        let input_value = Value::from_array(input_array)?;
        let inputs = inputs![input_value]?;

        let outputs = session.run(inputs)?;
        let output = outputs
            .get("output")
            .ok_or_else(|| ImageProcessingError::Processing("No output from model".to_string()))?;

        let output_tensor = output.try_extract_tensor::<Self::InputType>()?;
        let output_shape = output_tensor.shape();

        // Create a new tensor with the alpha channel values
        let alpha_values = output_tensor.as_slice().unwrap().to_vec();

        // The output should match the input dimensions but with single channel
        ndarray::Array4::from_shape_vec((1, 1, output_shape[2], output_shape[3]), alpha_values)
            .map_err(|e| ImageProcessingError::Processing(e.to_string()))
    }
}
