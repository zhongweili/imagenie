use imagenie_core::erase::Eraser;
use imagenie_core::segment::{Prompt, Segmenter, dilate_mask};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args.get(1).expect("usage: click_erase <image> <x,y 0-1>");
    let coords = args.get(2).map(String::as_str).unwrap_or("0.8,0.5");
    let (px, py) = coords.split_once(',').expect("bad coords");
    let (px, py): (f32, f32) = (px.parse().unwrap(), py.parse().unwrap());

    let models = "experiments/models/candidates";
    let image = image::open(input).unwrap();
    let out_dir = std::env::temp_dir().join("imagenie-click-erase");
    std::fs::create_dir_all(&out_dir).unwrap();

    let t = Instant::now();
    let mut seg = Segmenter::new(
        &format!("{models}/sam2.1_tiny/sam2.1_hiera_tiny.encoder.onnx"),
        &format!("{models}/sam2.1_tiny/sam2.1_hiera_tiny.decoder.onnx"),
    )
    .unwrap();
    println!("SAM sessions loaded in {:?}", t.elapsed());

    let t = Instant::now();
    seg.encode(&image).unwrap();
    println!("encode: {:?}", t.elapsed());

    let t = Instant::now();
    let result = seg
        .decode(Prompt::Point {
            x: px * image.width() as f32,
            y: py * image.height() as f32,
        })
        .unwrap();
    println!(
        "decode: {:?}, iou {:.3}, mask px {}",
        t.elapsed(),
        result.iou,
        result.mask_pixels
    );
    result.mask.save(out_dir.join("mask.png")).unwrap();

    let t = Instant::now();
    let dilated = dilate_mask(&result.mask, 12 * image.width().max(image.height()) / 512);
    println!("dilate: {:?}", t.elapsed());

    let t = Instant::now();
    let mut eraser = Eraser::new(&format!("{models}/lama_fp32.onnx")).unwrap();
    let erased = eraser.erase(&image, &dilated).unwrap();
    println!("erase: {:?}", t.elapsed());

    erased.save(out_dir.join("erased.png")).unwrap();
    println!("outputs in {}", out_dir.display());
}
