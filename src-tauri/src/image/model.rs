use image::DynamicImage;
use ort::{inputs, GraphOptimizationLevel, Session, Value};
use std::marker::PhantomData;

use crate::image::error::ImageProcessingError;
use crate::image::tensor::{image_to_tensor, tensor_to_image};
use crate::image::types::{NumericType, TensorInput, TensorOutput, UpscalingParams};

pub trait ImageModel {
    type Params: Sync;
    type InputType: NumericType;
    type OutputType: NumericType;

    fn load_session(model_path: &str) -> Result<Session, ImageProcessingError>;
    fn preprocess(
        image_path: &str,
        params: &Self::Params,
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
            .map_err(|e| ImageProcessingError::OrtError(e.to_string()))
    }

    fn preprocess(
        image_path: &str,
        _params: &Self::Params,
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
        let output = outputs.get("output").ok_or_else(|| {
            ImageProcessingError::ProcessingError("No output from model".to_string())
        })?;

        let output_tensor = output.try_extract_tensor::<Self::InputType>()?;
        let output_shape = output_tensor.shape();

        ndarray::Array4::from_shape_vec(
            (1, 3, output_shape[2], output_shape[3]),
            output_tensor.as_slice().unwrap().to_vec(),
        )
        .map_err(|e| ImageProcessingError::ProcessingError(e.to_string()))
    }
}
