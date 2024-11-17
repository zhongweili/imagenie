use ndarray::ArrayBase;

#[derive(Default, Clone)]
pub struct UpscalingParams {}

#[derive(Clone)]
pub struct FaceRestorationParams {
    pub face_size: u32,
    pub original_width: Option<u32>,
    pub original_height: Option<u32>,
}

#[derive(Clone, Default)]
pub struct BackgroundRemovalParams {
    pub model_width: usize,
    pub model_height: usize,
    pub original_width: Option<u32>,
    pub original_height: Option<u32>,
    pub scaling_factor: Option<f32>,
}

impl Default for FaceRestorationParams {
    fn default() -> Self {
        Self {
            face_size: 512,
            original_width: None,
            original_height: None,
        }
    }
}

// Define concrete tensor types
pub type TensorInput<T> = ArrayBase<ndarray::OwnedRepr<T>, ndarray::Dim<[usize; 4]>>;
pub type TensorOutput<T> = ArrayBase<ndarray::OwnedRepr<T>, ndarray::Dim<[usize; 4]>>;

pub trait NumericType:
    Copy
    + Send
    + Sync
    + std::fmt::Debug
    + ort::IntoTensorElementType
    + ort::PrimitiveTensorElementType
    + 'static
{
    fn from_f32(v: f32) -> Self;
    fn to_f32(self) -> f32;
}

impl NumericType for f32 {
    fn from_f32(v: f32) -> Self {
        v
    }
    fn to_f32(self) -> f32 {
        self
    }
}

impl NumericType for half::f16 {
    fn from_f32(v: f32) -> Self {
        half::f16::from_f32(v)
    }
    fn to_f32(self) -> f32 {
        self.to_f32()
    }
}
