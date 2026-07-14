use imagenie_core::erase::EraseOperator;
use imagenie_core::registry;
use imagenie_core::segment::{Prompt, SegmentOperator};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args.get(1).expect("usage: click_erase <image> <x,y 0-1>");
    let coords = args.get(2).map(String::as_str).unwrap_or("0.8,0.5");
    let (px, py) = coords.split_once(',').expect("bad coords");
    let (px, py): (f32, f32) = (px.parse().unwrap(), py.parse().unwrap());

    let image = image::open(input).unwrap();
    let out_dir = std::env::temp_dir().join("imagenie-click-erase");
    std::fs::create_dir_all(&out_dir).unwrap();

    let sam = registry::find("sam").unwrap();
    let sam_paths = registry::model_paths(sam);
    let t = Instant::now();
    let segment = SegmentOperator::new(
        &sam_paths[0].to_string_lossy(),
        &sam_paths[1].to_string_lossy(),
    )
    .unwrap();
    println!("SAM sessions loaded in {:?}", t.elapsed());

    let prompt = Prompt::Point {
        x: px * image.width() as f32,
        y: py * image.height() as f32,
    };
    let t = Instant::now();
    let segmented = segment.process_image(input, &image, prompt).unwrap();
    println!(
        "segment: {:?}, iou {:.3}, mask px {}",
        t.elapsed(),
        segmented.metrics.iou.unwrap(),
        segmented.metrics.mask_pixels.unwrap()
    );
    let t = Instant::now();
    let cached = segment.process_image(input, &image, prompt).unwrap();
    assert_eq!(cached.metrics, segmented.metrics);
    assert_eq!(cached.image.to_luma8(), segmented.image.to_luma8());
    println!("cached decode: {:?}", t.elapsed());

    let mask = segmented.image.to_luma8();
    mask.save(out_dir.join("mask.png")).unwrap();

    let erase = registry::find("erase").unwrap();
    let erase_path = registry::model_paths(erase);
    let t = Instant::now();
    let eraser = EraseOperator::new(&erase_path[0].to_string_lossy()).unwrap();
    let dilate = 12 * image.width().max(image.height()) / 512;
    let erased = eraser.process_image(&image, &mask, dilate).unwrap().image;
    println!("erase: {:?}", t.elapsed());

    erased.save(out_dir.join("erased.png")).unwrap();
    println!("outputs in {}", out_dir.display());
}
