use imagenie_core::birefnet::BiRefNetModel;
use imagenie_core::processor::ModelProcessor;
use imagenie_core::types::BackgroundRemovalParams;
use std::time::Instant;

fn main() {
    let input = std::env::args()
        .nth(1)
        .expect("usage: birefnet_smoke <image>");
    let out_dir = std::env::temp_dir().join("imagenie-birefnet");
    std::fs::create_dir_all(&out_dir).unwrap();

    let t = Instant::now();
    let processor = ModelProcessor::<BiRefNetModel>::new(
        "experiments/models/candidates/birefnet-general-lite.onnx",
    )
    .unwrap();
    println!("loaded in {:?}", t.elapsed());

    let params = BackgroundRemovalParams {
        model_width: 1024,
        model_height: 1024,
        ..Default::default()
    };
    let t = Instant::now();
    let mask = processor.process_single(&input, &params).unwrap();
    println!("inference in {:?}", t.elapsed());

    mask.save(out_dir.join("mask.png")).unwrap();
    println!("mask saved to {}", out_dir.display());
}
