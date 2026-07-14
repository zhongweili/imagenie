use imagenie_core::ImageOperator;
use imagenie_core::background::BackgroundRemover;
use imagenie_core::birefnet::BiRefNetModel;
use imagenie_core::types::BackgroundRemovalParams;
use std::time::Instant;

fn main() {
    let input = std::env::args()
        .nth(1)
        .expect("usage: birefnet_smoke <image>");
    let out_dir = std::env::temp_dir().join("imagenie-birefnet");
    std::fs::create_dir_all(&out_dir).unwrap();

    let t = Instant::now();
    let processor = BackgroundRemover::<BiRefNetModel>::new(
        "experiments/models/candidates/birefnet-general-lite.onnx",
        BackgroundRemovalParams {
            model_width: 1024,
            model_height: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    println!("loaded in {:?}", t.elapsed());

    let t = Instant::now();
    let removed = processor.process(&input).unwrap().image;
    println!("inference in {:?}", t.elapsed());

    removed.save(out_dir.join("removed.png")).unwrap();
    println!("output saved to {}", out_dir.display());
}
