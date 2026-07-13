use crate::events::{Emitter, Event, codes};
use crate::models::ensure;
use crate::output::{Collision, check_collision, prepare_output_dir, resolve};
use crate::{
    CommonArgs, EXIT_INPUT_UNREADABLE, EXIT_MODEL_MISSING, EXIT_OK, EXIT_PARTIAL, EXIT_USAGE,
    OutputFormat, RembgQuality,
};
use image::DynamicImage;
use imagenie_core::birefnet::BiRefNetModel;
use imagenie_core::erase::Eraser;
use imagenie_core::model::{BackgroundRemovalModel, FaceRestorationModel, UpscalingModel};
use imagenie_core::processor::ModelProcessor;
use imagenie_core::registry;
use imagenie_core::segment::{Prompt, Segmenter, dilate_mask};
use imagenie_core::types::{BackgroundRemovalParams, FaceRestorationParams, UpscalingParams};
use rayon::prelude::*;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn jobs(common: &CommonArgs) -> usize {
    common
        .jobs
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .min(4)
        })
        .max(1)
}

fn model_path(name: &str) -> String {
    let spec = registry::find(name).expect("model in registry");
    registry::model_paths(spec)[0]
        .to_string_lossy()
        .into_owned()
}

fn save(img: &DynamicImage, path: &Path, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Jpeg => img.to_rgb8().save(path)?,
        _ => img.save(path)?,
    }
    Ok(())
}

/// Read image from stdin for "-" pipe mode.
fn read_stdin_image() -> anyhow::Result<DynamicImage> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    Ok(image::load_from_memory(&buf)?)
}

fn write_stdout_image(img: &DynamicImage, format: OutputFormat) -> anyhow::Result<()> {
    let fmt = match format {
        OutputFormat::Png => image::ImageFormat::Png,
        OutputFormat::Jpeg => image::ImageFormat::Jpeg,
        OutputFormat::Webp => image::ImageFormat::WebP,
    };
    let mut buf = std::io::Cursor::new(Vec::new());
    if fmt == image::ImageFormat::Jpeg {
        img.to_rgb8().write_to(&mut buf, fmt)?;
    } else {
        img.write_to(&mut buf, fmt)?;
    }
    std::io::stdout().write_all(buf.get_ref())?;
    Ok(())
}

/// Generic batch runner: one model load, rayon-parallel processing,
/// per-image failures don't abort the batch.
fn run_batch<F>(
    op: &'static str,
    inputs: Vec<String>,
    suffix: &str,
    ext: &str,
    common: &CommonArgs,
    process: F,
) -> u8
where
    F: Fn(&str) -> Result<DynamicImage, String> + Sync,
{
    let emitter = Emitter::new(common.json);
    let multi = inputs.len() > 1;

    // Pipe mode ("-") is handled by the caller via process("-")
    if multi && inputs.iter().any(|i| i == "-") {
        eprintln!("error: \"-\" (stdin) can only be used as the sole input");
        return EXIT_USAGE;
    }

    if prepare_output_dir(common, multi).is_err() {
        eprintln!("error: cannot create output directory");
        return EXIT_USAGE;
    }

    let succeeded = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let unreadable = AtomicUsize::new(0);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs(common))
        .build()
        .expect("thread pool");

    pool.install(|| {
        inputs.par_iter().for_each(|input| {
            emitter.emit(&Event::Start { op, input });
            let t = Instant::now();

            if input == "-" {
                match process("-") {
                    Ok(_) => {
                        succeeded.fetch_add(1, Ordering::Relaxed);
                        emitter.emit(&Event::Done {
                            op,
                            input,
                            output: "-",
                            ms: t.elapsed().as_millis(),
                            iou: None,
                            mask_pixels: None,
                        });
                    }
                    Err(msg) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        emitter.emit(&Event::Error {
                            op,
                            input,
                            code: codes::PROCESSING_FAILED,
                            message: &msg,
                        });
                    }
                }
                return;
            }

            let out_path = resolve(input, suffix, ext, common, multi);
            match check_collision(&out_path, common) {
                Collision::Skip => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Collision::Fail => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    emitter.emit(&Event::Error {
                        op,
                        input,
                        code: codes::OUTPUT_EXISTS,
                        message: &format!(
                            "output {} exists (use --force or --skip-existing)",
                            out_path.display()
                        ),
                    });
                    return;
                }
                Collision::Write => {}
            }

            match process(input) {
                Ok(img) => {
                    let format = match ext {
                        "jpg" => OutputFormat::Jpeg,
                        "webp" => OutputFormat::Webp,
                        _ => OutputFormat::Png,
                    };
                    if let Err(e) = save(&img, &out_path, format) {
                        failed.fetch_add(1, Ordering::Relaxed);
                        emitter.emit(&Event::Error {
                            op,
                            input,
                            code: codes::OUTPUT_WRITE_FAILED,
                            message: &e.to_string(),
                        });
                    } else {
                        succeeded.fetch_add(1, Ordering::Relaxed);
                        emitter.emit(&Event::Done {
                            op,
                            input,
                            output: &out_path.to_string_lossy(),
                            ms: t.elapsed().as_millis(),
                            iou: None,
                            mask_pixels: None,
                        });
                    }
                }
                Err(msg) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    let code = if msg.contains("No such file") || msg.contains("os error 2") {
                        unreadable.fetch_add(1, Ordering::Relaxed);
                        codes::INPUT_UNREADABLE
                    } else {
                        codes::PROCESSING_FAILED
                    };
                    emitter.emit(&Event::Error {
                        op,
                        input,
                        code,
                        message: &msg,
                    });
                }
            }
        });
    });

    let (s, f, k) = (
        succeeded.load(Ordering::Relaxed),
        failed.load(Ordering::Relaxed),
        skipped.load(Ordering::Relaxed),
    );
    emitter.emit(&Event::Summary {
        op,
        total: inputs.len(),
        succeeded: s,
        failed: f,
        skipped: k,
    });

    let _ = s;
    if f == 0 {
        EXIT_OK
    } else if f == inputs.len() && unreadable.load(Ordering::Relaxed) == f {
        EXIT_INPUT_UNREADABLE
    } else {
        EXIT_PARTIAL
    }
}

pub fn upscale(inputs: Vec<String>, format: OutputFormat, common: &CommonArgs) -> u8 {
    let Some(spec) = registry::find("upscale") else {
        return EXIT_MODEL_MISSING;
    };
    if let Err(code) = ensure(spec, common.yes) {
        return code;
    }
    let processor = match ModelProcessor::<UpscalingModel>::new(&model_path("upscale")) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to load model: {e}");
            return EXIT_MODEL_MISSING;
        }
    };
    let params = UpscalingParams::default();
    let ext = format.extension();

    run_batch("upscale", inputs, "upscaled", ext, common, |input| {
        if input == "-" {
            let img = read_stdin_image().map_err(|e| e.to_string())?;
            let tmp = write_temp(&img).map_err(|e| e.to_string())?;
            let out = processor
                .process_single(tmp.to_str().unwrap(), &params)
                .map_err(|e| e.to_string())?;
            std::fs::remove_file(&tmp).ok();
            write_stdout_image(&out, format).map_err(|e| e.to_string())?;
            Ok(out)
        } else {
            processor
                .process_single(input, &params)
                .map_err(|e| e.to_string())
        }
    })
}

pub fn restore(inputs: Vec<String>, common: &CommonArgs) -> u8 {
    let Some(spec) = registry::find("restore") else {
        return EXIT_MODEL_MISSING;
    };
    if let Err(code) = ensure(spec, common.yes) {
        return code;
    }
    let processor = match ModelProcessor::<FaceRestorationModel>::new(&model_path("restore")) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to load model: {e}");
            return EXIT_MODEL_MISSING;
        }
    };
    let params = FaceRestorationParams::default();

    run_batch("restore", inputs, "restored", "png", common, |input| {
        if input == "-" {
            let img = read_stdin_image().map_err(|e| e.to_string())?;
            let tmp = write_temp(&img).map_err(|e| e.to_string())?;
            let out = processor
                .process_single(tmp.to_str().unwrap(), &params)
                .map_err(|e| e.to_string())?;
            std::fs::remove_file(&tmp).ok();
            write_stdout_image(&out, OutputFormat::Png).map_err(|e| e.to_string())?;
            Ok(out)
        } else {
            processor
                .process_single(input, &params)
                .map_err(|e| e.to_string())
        }
    })
}

/// Composite the alpha mask from the model onto the original image.
fn apply_mask(input: &str, mask: &DynamicImage) -> Result<DynamicImage, String> {
    let original = image::open(input).map_err(|e| e.to_string())?;
    let mask = mask.resize_exact(
        original.width(),
        original.height(),
        image::imageops::FilterType::Triangle,
    );
    let original_rgba = original.to_rgba8();
    let mask_rgba = mask.to_rgba8();
    let mut final_image = image::RgbaImage::new(original.width(), original.height());
    for (x, y, pixel) in final_image.enumerate_pixels_mut() {
        let o = original_rgba.get_pixel(x, y);
        let m = mask_rgba.get_pixel(x, y);
        *pixel = image::Rgba([o[0], o[1], o[2], m[3]]);
    }
    Ok(DynamicImage::ImageRgba8(final_image))
}

pub fn rembg(
    inputs: Vec<String>,
    quality: RembgQuality,
    format: OutputFormat,
    common: &CommonArgs,
) -> u8 {
    if format == OutputFormat::Jpeg {
        eprintln!("error: rembg output needs an alpha channel; use png or webp");
        return EXIT_USAGE;
    }
    let model_name = match quality {
        RembgQuality::Fast => "rembg",
        RembgQuality::High => "rembg-hq",
    };
    let Some(spec) = registry::find(model_name) else {
        return EXIT_MODEL_MISSING;
    };
    if let Err(code) = ensure(spec, common.yes) {
        return code;
    }

    let params = BackgroundRemovalParams {
        model_width: 1024,
        model_height: 1024,
        ..Default::default()
    };
    let ext = format.extension();

    enum P {
        Fast(ModelProcessor<BackgroundRemovalModel>),
        High(ModelProcessor<BiRefNetModel>),
    }
    let processor = match quality {
        RembgQuality::Fast => ModelProcessor::new(&model_path(model_name)).map(P::Fast),
        RembgQuality::High => ModelProcessor::new(&model_path(model_name)).map(P::High),
    };
    let processor = match processor {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to load model: {e}");
            return EXIT_MODEL_MISSING;
        }
    };

    run_batch("rembg", inputs, "rembg", ext, common, |input| {
        let (path, tmp) = if input == "-" {
            let img = read_stdin_image().map_err(|e| e.to_string())?;
            let tmp = write_temp(&img).map_err(|e| e.to_string())?;
            (tmp.to_string_lossy().into_owned(), Some(tmp))
        } else {
            (input.to_string(), None)
        };

        let mask = match &processor {
            P::Fast(p) => p.process_single(&path, &params),
            P::High(p) => p.process_single(&path, &params),
        }
        .map_err(|e| e.to_string())?;
        let result = apply_mask(&path, &mask)?;

        if let Some(tmp) = tmp {
            std::fs::remove_file(&tmp).ok();
            write_stdout_image(&result, format).map_err(|e| e.to_string())?;
        }
        Ok(result)
    })
}

fn parse_point(s: &str) -> Option<(f32, f32)> {
    let (x, y) = s.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

fn parse_box(s: &str) -> Option<(f32, f32, f32, f32)> {
    let parts: Vec<f32> = s.split(',').filter_map(|v| v.trim().parse().ok()).collect();
    match parts[..] {
        [x1, y1, x2, y2] => Some((x1, y1, x2, y2)),
        _ => None,
    }
}

pub fn segment(input: &str, point: Option<&str>, bbox: Option<&str>, common: &CommonArgs) -> u8 {
    let prompt = match (point, bbox) {
        (Some(p), None) => match parse_point(p) {
            Some((x, y)) => Prompt::Point { x, y },
            None => {
                eprintln!("error: --point expects \"x,y\" in pixels");
                return EXIT_USAGE;
            }
        },
        (None, Some(b)) => match parse_box(b) {
            Some((x1, y1, x2, y2)) => Prompt::Box { x1, y1, x2, y2 },
            None => {
                eprintln!("error: --box expects \"x1,y1,x2,y2\" in pixels");
                return EXIT_USAGE;
            }
        },
        _ => {
            eprintln!("error: segment requires exactly one of --point or --box");
            return EXIT_USAGE;
        }
    };

    let Some(spec) = registry::find("sam") else {
        return EXIT_MODEL_MISSING;
    };
    if let Err(code) = ensure(spec, common.yes) {
        return code;
    }

    let emitter = Emitter::new(common.json);
    emitter.emit(&Event::Start {
        op: "segment",
        input,
    });
    let t = Instant::now();

    let image = match image::open(input) {
        Ok(img) => img,
        Err(e) => {
            emitter.emit(&Event::Error {
                op: "segment",
                input,
                code: codes::INPUT_UNREADABLE,
                message: &e.to_string(),
            });
            return EXIT_INPUT_UNREADABLE;
        }
    };

    let paths = registry::model_paths(spec);
    let result = Segmenter::new(&paths[0].to_string_lossy(), &paths[1].to_string_lossy()).and_then(
        |mut seg| {
            seg.encode(&image)?;
            seg.decode(prompt)
        },
    );

    match result {
        Ok(res) => {
            let out_path = resolve(input, "mask", "png", common, false);
            if let Err(e) = res.mask.save(&out_path) {
                emitter.emit(&Event::Error {
                    op: "segment",
                    input,
                    code: codes::OUTPUT_WRITE_FAILED,
                    message: &e.to_string(),
                });
                return EXIT_PARTIAL;
            }
            emitter.emit(&Event::Done {
                op: "segment",
                input,
                output: &out_path.to_string_lossy(),
                ms: t.elapsed().as_millis(),
                iou: Some(res.iou),
                mask_pixels: Some(res.mask_pixels),
            });
            EXIT_OK
        }
        Err(e) => {
            emitter.emit(&Event::Error {
                op: "segment",
                input,
                code: codes::PROCESSING_FAILED,
                message: &e.to_string(),
            });
            EXIT_PARTIAL
        }
    }
}

pub fn erase(input: &str, mask_path: &Path, dilate: u32, common: &CommonArgs) -> u8 {
    let Some(spec) = registry::find("erase") else {
        return EXIT_MODEL_MISSING;
    };
    if let Err(code) = ensure(spec, common.yes) {
        return code;
    }

    let emitter = Emitter::new(common.json);
    emitter.emit(&Event::Start { op: "erase", input });
    let t = Instant::now();

    let image = match image::open(input) {
        Ok(img) => img,
        Err(e) => {
            emitter.emit(&Event::Error {
                op: "erase",
                input,
                code: codes::INPUT_UNREADABLE,
                message: &e.to_string(),
            });
            return EXIT_INPUT_UNREADABLE;
        }
    };
    let mask = match image::open(mask_path) {
        Ok(m) => m.to_luma8(),
        Err(e) => {
            emitter.emit(&Event::Error {
                op: "erase",
                input,
                code: codes::INPUT_UNREADABLE,
                message: &format!("mask: {e}"),
            });
            return EXIT_INPUT_UNREADABLE;
        }
    };

    let mask = if dilate > 0 {
        dilate_mask(&mask, dilate)
    } else {
        mask
    };

    let result = Eraser::new(&model_path("erase")).and_then(|mut e| e.erase(&image, &mask));

    match result {
        Ok(img) => {
            let out_path = resolve(input, "erased", "png", common, false);
            if let Err(e) = img.save(&out_path) {
                emitter.emit(&Event::Error {
                    op: "erase",
                    input,
                    code: codes::OUTPUT_WRITE_FAILED,
                    message: &e.to_string(),
                });
                return EXIT_PARTIAL;
            }
            emitter.emit(&Event::Done {
                op: "erase",
                input,
                output: &out_path.to_string_lossy(),
                ms: t.elapsed().as_millis(),
                iou: None,
                mask_pixels: None,
            });
            EXIT_OK
        }
        Err(e) => {
            emitter.emit(&Event::Error {
                op: "erase",
                input,
                code: codes::PROCESSING_FAILED,
                message: &e.to_string(),
            });
            EXIT_PARTIAL
        }
    }
}

fn write_temp(img: &DynamicImage) -> anyhow::Result<std::path::PathBuf> {
    let path = std::env::temp_dir().join(format!("imagenie-stdin-{}.png", std::process::id()));
    img.save(&path)?;
    Ok(path)
}
