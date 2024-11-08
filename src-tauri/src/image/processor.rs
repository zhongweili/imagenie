use image::DynamicImage;
use rayon::prelude::*;
use std::marker::PhantomData;

use super::{ImageModel, ImageProcessingError};

pub struct ModelProcessor<M: ImageModel + Send + Sync> {
    session: ort::Session,
    _phantom: PhantomData<M>,
}

impl<M: ImageModel + Send + Sync> ModelProcessor<M> {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        let session = M::load_session(model_path)?;
        Ok(Self {
            session,
            _phantom: PhantomData,
        })
    }

    pub fn process_single(
        &self,
        image_path: &str,
        params: &M::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        let input = M::preprocess(image_path, params)?;
        let output = M::process(&self.session, &input)?;
        M::postprocess(&output, params)
    }

    pub fn process_batch<I>(
        &self,
        image_paths: I,
        params: &M::Params,
    ) -> Result<Vec<DynamicImage>, ImageProcessingError>
    where
        I: IntoParallelIterator<Item = String>,
    {
        image_paths
            .into_par_iter()
            .map(|path| self.process_single(&path, params))
            .collect()
    }
}
