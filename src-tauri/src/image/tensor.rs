use image::DynamicImage;

use crate::image::error::ImageProcessingError;
use crate::image::types::{NumericType, TensorInput, TensorOutput};

pub fn image_to_tensor<T: NumericType>(
    image: &DynamicImage,
) -> Result<TensorInput<T>, ImageProcessingError> {
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    let tensor =
        ndarray::Array::from_shape_fn((1, 3, height as usize, width as usize), |(_, c, y, x)| {
            let pixel = rgb_image.get_pixel(x as u32, y as u32);
            T::from_f32(pixel[c] as f32 / 255.0)
        });

    Ok(tensor)
}

pub fn tensor_to_image<T: NumericType>(
    tensor: &TensorOutput<T>,
) -> Result<DynamicImage, ImageProcessingError> {
    let (_, _, h, w) = tensor.dim();
    let mut img_buffer = image::RgbImage::new(w as u32, h as u32);

    for y in 0..h {
        for x in 0..w {
            let to_u8 = |v: T| (v.to_f32() * 255.0).clamp(0.0, 255.0) as u8;

            let r = to_u8(tensor[[0, 0, y, x]]);
            let g = to_u8(tensor[[0, 1, y, x]]);
            let b = to_u8(tensor[[0, 2, y, x]]);
            img_buffer.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
        }
    }

    Ok(DynamicImage::ImageRgb8(img_buffer))
}
