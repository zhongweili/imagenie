use image::DynamicImage;
use std::marker::PhantomData;
use std::sync::Mutex;

use super::{ImageModel, ImageProcessingError};

pub struct ModelProcessor<M: ImageModel + Send + Sync> {
    // ort 2.0 requires `&mut Session` to run; inference is serialized here while
    // pre/post-processing stays parallel. The session itself uses 4 intra-op threads.
    session: Mutex<ort::session::Session>,
    _phantom: PhantomData<M>,
}

impl<M: ImageModel + Send + Sync> ModelProcessor<M> {
    pub fn new(model_path: &str) -> Result<Self, ImageProcessingError> {
        let session = M::load_session(model_path)?;
        Ok(Self {
            session: Mutex::new(session),
            _phantom: PhantomData,
        })
    }

    pub fn process_single(
        &self,
        image_path: &str,
        params: &M::Params,
    ) -> Result<DynamicImage, ImageProcessingError> {
        let mut params = params.clone();
        let input = M::preprocess(image_path, &mut params)?;
        let output = {
            let mut session = self.session.lock().unwrap();
            M::process(&mut session, &input)?
        };
        M::postprocess(&output, &params)
    }
}
