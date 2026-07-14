use imagenie_core::ImageOperator;
use imagenie_core::background::BackgroundRemover;
use imagenie_core::model::{BackgroundRemovalModel, FaceRestorationModel, UpscalingModel};
use imagenie_core::processor::ModelProcessor;
use imagenie_core::registry;
use imagenie_core::types::{BackgroundRemovalParams, FaceRestorationParams, UpscalingParams};
use std::time::Instant;

fn main() {
    let input = std::env::args().nth(1).expect("usage: smoke <image>");
    let models = registry::models_dir();
    let out_dir = std::env::temp_dir().join("imagenie-smoke");
    std::fs::create_dir_all(&out_dir).unwrap();

    let t = Instant::now();
    let upscaler = ModelProcessor::<UpscalingModel>::new(
        models.join("image_upscaling.onnx").to_str().unwrap(),
    )
    .unwrap();
    let img = upscaler
        .process_single(&input, &UpscalingParams::default())
        .unwrap();
    img.save(out_dir.join("upscaled.png")).unwrap();
    println!(
        "upscale OK ({}x{}) in {:?}",
        img.width(),
        img.height(),
        t.elapsed()
    );

    let t = Instant::now();
    let rembg = BackgroundRemover::<BackgroundRemovalModel>::new(
        models.join("background_removal.onnx").to_str().unwrap(),
        BackgroundRemovalParams {
            model_width: 1024,
            model_height: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let removed = rembg.process(&input).unwrap().image;
    removed.save(out_dir.join("removed.png")).unwrap();
    println!(
        "rembg OK ({}x{}) in {:?}",
        removed.width(),
        removed.height(),
        t.elapsed()
    );

    let t = Instant::now();
    let restore = ModelProcessor::<FaceRestorationModel>::new(
        models.join("face_restoration.onnx").to_str().unwrap(),
    )
    .unwrap();
    let img = restore
        .process_single(&input, &FaceRestorationParams::default())
        .unwrap();
    img.save(out_dir.join("restored.png")).unwrap();
    println!(
        "restore OK ({}x{}) in {:?}",
        img.width(),
        img.height(),
        t.elapsed()
    );

    println!("outputs in {}", out_dir.display());
}
