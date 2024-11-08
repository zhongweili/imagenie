use ndarray::ArrayBase;

#[derive(Default)]
pub struct UpscalingParams {}

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
