use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ORT error: {0}")]
    Ort(String),
    #[error("Processing error: {0}")]
    Processing(String),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

impl From<ort::Error> for ImageProcessingError {
    fn from(error: ort::Error) -> Self {
        ImageProcessingError::Processing(error.to_string())
    }
}
