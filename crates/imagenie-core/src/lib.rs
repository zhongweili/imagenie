pub mod background;
pub mod birefnet;
pub mod erase;
mod error;
pub mod model;
pub mod operation;
pub mod processor;
pub mod registry;
pub mod runtime;
pub mod segment;
mod tensor;
pub mod types;

pub use error::ImageProcessingError;
pub use model::ImageModel;
pub use operation::{
    ImageOperator, ModelOperator, ParameterizedImageOperator, ProcessMetrics, ProcessOutput,
};
