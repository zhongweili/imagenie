use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("ORT error: {0}")]
    OrtError(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

impl From<ort::Error> for ImageProcessingError {
    fn from(error: ort::Error) -> Self {
        ImageProcessingError::ProcessingError(error.to_string())
    }
}
