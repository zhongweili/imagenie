use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use imagenie_core::background::BackgroundRemover;
use imagenie_core::birefnet::BiRefNetModel;
use imagenie_core::erase::{EraseOperator, EraseParams};
use imagenie_core::model::{BackgroundRemovalModel, FaceRestorationModel, UpscalingModel};
use imagenie_core::registry;
use imagenie_core::runtime::{ExecutionProvider, RuntimeOptions};
use imagenie_core::segment::{Prompt, SegmentOperator};
use imagenie_core::types::{BackgroundRemovalParams, FaceRestorationParams, UpscalingParams};
use imagenie_core::{
    ImageOperator, ModelOperator, ParameterizedImageOperator, ProcessMetrics, ProcessOutput,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const REPORT_SCHEMA_VERSION: &str = "1.0";

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OperatorKind {
    Upscale,
    Rembg,
    RembgHigh,
    Restore,
    Segment,
    Erase,
    ClickErase,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RuntimeProvider {
    Cpu,
    CoreMl,
}

impl RuntimeProvider {
    fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::CoreMl => "coreml",
        }
    }
}

impl OperatorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Upscale => "upscale",
            Self::Rembg => "rembg",
            Self::RembgHigh => "rembg-high",
            Self::Restore => "restore",
            Self::Segment => "segment",
            Self::Erase => "erase",
            Self::ClickErase => "click-erase",
        }
    }
}

#[derive(Parser)]
#[command(about = "Measure one Imagenie operator in an isolated process")]
struct Args {
    #[arg(long, value_enum)]
    operator: OperatorKind,
    #[arg(long)]
    input: PathBuf,
    /// Relative point prompt in 0-1 coordinates, used by segment/click-erase.
    #[arg(long, default_value = "0.5,0.5")]
    point: String,
    /// Mask path used by erase.
    #[arg(long)]
    mask: Option<PathBuf>,
    #[arg(long, default_value = "12")]
    dilate: u32,
    /// Unmeasured runs after the cold run and before recorded warm runs.
    #[arg(long, default_value = "1")]
    warmups: usize,
    #[arg(long, default_value = "3")]
    iterations: usize,
    #[arg(long, value_enum, default_value = "cpu")]
    execution_provider: RuntimeProvider,
    #[arg(long, default_value = "4")]
    intra_threads: usize,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    memory_pattern: bool,
    /// Optionally save the final image outside the measured region.
    #[arg(long)]
    output: Option<PathBuf>,
}

enum LoadedOperator {
    Upscale(ModelOperator<UpscalingModel>),
    Rembg(BackgroundRemover<BackgroundRemovalModel>),
    RembgHigh(BackgroundRemover<BiRefNetModel>),
    Restore(ModelOperator<FaceRestorationModel>),
    Segment {
        operator: SegmentOperator,
        prompt: Prompt,
    },
    Erase {
        operator: EraseOperator,
        params: EraseParams,
    },
    ClickErase {
        segment: SegmentOperator,
        erase: EraseOperator,
        prompt: Prompt,
        dilate: u32,
    },
}

impl LoadedOperator {
    fn load(args: &Args, width: u32, height: u32) -> Result<Self> {
        let runtime = runtime_options(args);
        let params = BackgroundRemovalParams {
            model_width: 1024,
            model_height: 1024,
            ..Default::default()
        };
        match args.operator {
            OperatorKind::Upscale => Ok(Self::Upscale(ModelOperator::new_with_runtime(
                &model_path("upscale")?,
                UpscalingParams::default(),
                runtime,
            )?)),
            OperatorKind::Rembg => Ok(Self::Rembg(BackgroundRemover::new_with_runtime(
                &model_path("rembg")?,
                params,
                runtime,
            )?)),
            OperatorKind::RembgHigh => Ok(Self::RembgHigh(BackgroundRemover::new_with_runtime(
                &model_path("rembg-hq")?,
                params,
                runtime,
            )?)),
            OperatorKind::Restore => Ok(Self::Restore(ModelOperator::new_with_runtime(
                &model_path("restore")?,
                FaceRestorationParams::default(),
                runtime,
            )?)),
            OperatorKind::Segment => {
                let paths = model_paths("sam")?;
                Ok(Self::Segment {
                    operator: SegmentOperator::new_with_runtime(
                        &paths[0].to_string_lossy(),
                        &paths[1].to_string_lossy(),
                        runtime,
                    )?,
                    prompt: parse_relative_point(&args.point, width, height)?,
                })
            }
            OperatorKind::Erase => {
                let mask_path = args.mask.clone().context("--mask is required for erase")?;
                Ok(Self::Erase {
                    operator: EraseOperator::new_with_runtime(&model_path("erase")?, runtime)?,
                    params: EraseParams {
                        mask_path,
                        dilate: args.dilate,
                    },
                })
            }
            OperatorKind::ClickErase => {
                let sam = model_paths("sam")?;
                Ok(Self::ClickErase {
                    segment: SegmentOperator::new_with_runtime(
                        &sam[0].to_string_lossy(),
                        &sam[1].to_string_lossy(),
                        runtime,
                    )?,
                    erase: EraseOperator::new_with_runtime(&model_path("erase")?, runtime)?,
                    prompt: parse_relative_point(&args.point, width, height)?,
                    dilate: args.dilate,
                })
            }
        }
    }

    fn process(&self, input: &str) -> Result<ProcessOutput> {
        match self {
            Self::Upscale(operator) => Ok(operator.process(input)?),
            Self::Rembg(operator) => Ok(operator.process(input)?),
            Self::RembgHigh(operator) => Ok(operator.process(input)?),
            Self::Restore(operator) => Ok(operator.process(input)?),
            Self::Segment { operator, prompt } => Ok(operator.process_with(input, prompt)?),
            Self::Erase { operator, params } => Ok(operator.process_with(input, params)?),
            Self::ClickErase {
                segment,
                erase,
                prompt,
                dilate,
            } => {
                let image = image::open(input)?;
                let segmented = segment.process_image(input, &image, *prompt)?;
                let mask = segmented.image.to_luma8();
                let mut erased = erase.process_image(&image, &mask, *dilate)?;
                erased.metrics = segmented.metrics;
                erased.warnings.extend(segmented.warnings);
                Ok(erased)
            }
        }
    }
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    generated_at_unix_ms: u128,
    environment: Environment,
    git: GitState,
    case: CaseReport,
    measurements: Measurements,
    result: ResultReport,
}

#[derive(Serialize)]
struct Environment {
    os: &'static str,
    arch: &'static str,
    logical_cpus: usize,
    cpu_model: Option<String>,
    memory_bytes: Option<u64>,
    execution_provider: &'static str,
    intra_threads: usize,
    memory_pattern: bool,
}

#[derive(Serialize)]
struct GitState {
    commit: Option<String>,
    dirty: Option<bool>,
}

#[derive(Serialize)]
struct CaseReport {
    operator: &'static str,
    input: String,
    input_sha256: String,
    input_width: u32,
    input_height: u32,
    point: Option<String>,
    mask: Option<String>,
    dilate: Option<u32>,
    models: Vec<ModelReport>,
}

#[derive(Serialize)]
struct ModelReport {
    name: &'static str,
    artifact_sha256: &'static str,
    size_bytes: u64,
}

#[derive(Serialize)]
struct Measurements {
    model_load_ms: f64,
    cold_ms: f64,
    warmups: usize,
    warm_iterations: usize,
    warm_ms: Vec<f64>,
    warm_stats: Statistics,
    peak_rss_bytes: Option<u64>,
}

#[derive(Default, Serialize)]
struct Statistics {
    min_ms: Option<f64>,
    mean_ms: Option<f64>,
    p50_ms: Option<f64>,
    p95_ms: Option<f64>,
    max_ms: Option<f64>,
}

#[derive(Serialize)]
struct ResultReport {
    output_width: u32,
    output_height: u32,
    metrics: ProcessMetrics,
    warnings: Vec<String>,
    saved_to: Option<String>,
    quality_status: &'static str,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.iterations == 0 {
        bail!("--iterations must be at least 1");
    }
    let input = args
        .input
        .to_str()
        .context("input path is not valid UTF-8")?;
    let dimensions = image::ImageReader::open(&args.input)?
        .with_guessed_format()?
        .into_dimensions()?;

    let load_started = Instant::now();
    let operator = LoadedOperator::load(&args, dimensions.0, dimensions.1)?;
    let model_load_ms = elapsed_ms(load_started);

    let cold_started = Instant::now();
    let mut output = operator.process(input)?;
    let cold_ms = elapsed_ms(cold_started);

    for _ in 0..args.warmups {
        output = operator.process(input)?;
    }

    let mut warm_ms = Vec::with_capacity(args.iterations);
    for _ in 0..args.iterations {
        let started = Instant::now();
        output = operator.process(input)?;
        warm_ms.push(elapsed_ms(started));
    }

    let saved_to = if let Some(path) = &args.output {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        output.image.save(path)?;
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    };

    let report = Report {
        schema_version: REPORT_SCHEMA_VERSION,
        generated_at_unix_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        environment: environment(&args),
        git: git_state(),
        case: CaseReport {
            operator: args.operator.as_str(),
            input: input.to_string(),
            input_sha256: sha256_file(&args.input)?,
            input_width: dimensions.0,
            input_height: dimensions.1,
            point: matches!(
                args.operator,
                OperatorKind::Segment | OperatorKind::ClickErase
            )
            .then(|| args.point.clone()),
            mask: args
                .mask
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            dilate: matches!(
                args.operator,
                OperatorKind::Erase | OperatorKind::ClickErase
            )
            .then_some(args.dilate),
            models: model_reports(args.operator),
        },
        measurements: Measurements {
            model_load_ms,
            cold_ms,
            warmups: args.warmups,
            warm_iterations: args.iterations,
            warm_stats: statistics(&warm_ms),
            warm_ms,
            peak_rss_bytes: peak_rss_bytes(),
        },
        result: ResultReport {
            output_width: output.image.width(),
            output_height: output.image.height(),
            metrics: output.metrics,
            warnings: output.warnings,
            saved_to,
            quality_status: "not_evaluated",
        },
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn model_reports(operator: OperatorKind) -> Vec<ModelReport> {
    let names: &[&str] = match operator {
        OperatorKind::Upscale => &["upscale"],
        OperatorKind::Rembg => &["rembg"],
        OperatorKind::RembgHigh => &["rembg-hq"],
        OperatorKind::Restore => &["restore"],
        OperatorKind::Segment => &["sam"],
        OperatorKind::Erase => &["erase"],
        OperatorKind::ClickErase => &["sam", "erase"],
    };
    names
        .iter()
        .filter_map(|name| registry::find(name))
        .map(|spec| ModelReport {
            name: spec.name,
            artifact_sha256: spec.sha256,
            size_bytes: spec.size_bytes,
        })
        .collect()
}

fn model_path(name: &str) -> Result<String> {
    Ok(model_paths(name)?[0].to_string_lossy().into_owned())
}

fn model_paths(name: &str) -> Result<Vec<PathBuf>> {
    let spec = registry::find(name).with_context(|| format!("unknown model {name}"))?;
    let paths = registry::model_paths(spec);
    for path in &paths {
        if !path.exists() {
            bail!(
                "model {name} is missing at {}; run `imagenie models pull {name}`",
                path.display()
            );
        }
    }
    Ok(paths)
}

fn parse_relative_point(value: &str, width: u32, height: u32) -> Result<Prompt> {
    let (x, y) = value
        .split_once(',')
        .context("--point expects relative coordinates x,y")?;
    let x: f32 = x.trim().parse()?;
    let y: f32 = y.trim().parse()?;
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        bail!("--point coordinates must be between 0 and 1");
    }
    Ok(Prompt::Point {
        x: x * width as f32,
        y: y * height as f32,
    })
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

fn statistics(values: &[f64]) -> Statistics {
    if values.is_empty() {
        return Statistics::default();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        let index = ((sorted.len() as f64 * p).ceil() as usize)
            .saturating_sub(1)
            .min(sorted.len() - 1);
        sorted[index]
    };
    Statistics {
        min_ms: sorted.first().copied(),
        mean_ms: Some(sorted.iter().sum::<f64>() / sorted.len() as f64),
        p50_ms: Some(percentile(0.50)),
        p95_ms: Some(percentile(0.95)),
        max_ms: sorted.last().copied(),
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn runtime_options(args: &Args) -> RuntimeOptions {
    RuntimeOptions {
        execution_provider: match args.execution_provider {
            RuntimeProvider::Cpu => ExecutionProvider::Cpu,
            RuntimeProvider::CoreMl => ExecutionProvider::CoreMl,
        },
        intra_threads: args.intra_threads,
        memory_pattern: args.memory_pattern,
    }
}

fn environment(args: &Args) -> Environment {
    Environment {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        logical_cpus: std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1),
        cpu_model: cpu_model(),
        memory_bytes: memory_bytes(),
        execution_provider: args.execution_provider.as_str(),
        intra_threads: args.intra_threads,
        memory_pattern: args.memory_pattern,
    }
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn cpu_model() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        command_output("sysctl", &["-n", "machdep.cpu.brand_string"])
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()?
            .lines()
            .find_map(|line| line.strip_prefix("model name\t: ").map(str::to_string))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        command_output("sysctl", &["-n", "hw.memsize"])?
            .parse()
            .ok()
    }
    #[cfg(target_os = "linux")]
    {
        let value = std::fs::read_to_string("/proc/meminfo")?
            .lines()
            .find_map(|line| line.strip_prefix("MemTotal:"))?
            .split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()?;
        Some(value * 1024)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn git_state() -> GitState {
    GitState {
        commit: command_output("git", &["rev-parse", "HEAD"]),
        dirty: command_output("git", &["status", "--porcelain"]).map(|value| !value.is_empty()),
    }
}

#[cfg(target_os = "macos")]
fn peak_rss_bytes() -> Option<u64> {
    peak_rss_raw().map(|value| value as u64)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn peak_rss_bytes() -> Option<u64> {
    peak_rss_raw().map(|value| value as u64 * 1024)
}

#[cfg(unix)]
fn peak_rss_raw() -> Option<libc::c_long> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: getrusage initializes the provided rusage on a successful return.
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    (result == 0).then(|| unsafe { usage.assume_init().ru_maxrss })
}

#[cfg(not(unix))]
fn peak_rss_bytes() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_are_stable_for_small_samples() {
        let stats = statistics(&[30.0, 10.0, 20.0]);
        assert_eq!(stats.min_ms, Some(10.0));
        assert_eq!(stats.mean_ms, Some(20.0));
        assert_eq!(stats.p50_ms, Some(20.0));
        assert_eq!(stats.p95_ms, Some(30.0));
        assert_eq!(stats.max_ms, Some(30.0));
    }

    #[test]
    fn relative_point_is_converted_to_pixels() {
        let prompt = parse_relative_point("0.25,0.75", 400, 200).unwrap();
        match prompt {
            Prompt::Point { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 150.0);
            }
            Prompt::Box { .. } => panic!("expected point"),
        }
    }
}
