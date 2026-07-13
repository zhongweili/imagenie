mod capabilities;
mod events;
mod info;
mod models;
mod output;
mod run;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

pub const EXIT_OK: u8 = 0;
pub const EXIT_PARTIAL: u8 = 1;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_MODEL_MISSING: u8 = 3;
pub const EXIT_INPUT_UNREADABLE: u8 = 4;

#[derive(Parser)]
#[command(
    name = "imagenie",
    version,
    about = "Privacy-first image processing for your terminal and your agents"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RembgQuality {
    Fast,
    High,
}

#[derive(clap::Args)]
pub struct CommonArgs {
    /// Output file (single input) or directory (multiple inputs).
    /// Defaults to a suffixed file next to each input (photo.upscaled.png).
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Overwrite existing outputs.
    #[arg(long)]
    pub force: bool,
    /// Skip inputs whose output already exists.
    #[arg(long)]
    pub skip_existing: bool,
    /// Parallel jobs for batch processing (default: min(4, cores)).
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Emit machine-readable JSONL events on stdout.
    #[arg(long)]
    pub json: bool,
    /// Auto-download missing models without prompting.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Upscale images 2x with Real-ESRGAN
    Upscale {
        /// Input images ("-" for stdin)
        #[arg(required = true)]
        inputs: Vec<String>,
        #[arg(long, value_enum, default_value = "png")]
        format: OutputFormat,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Remove image backgrounds
    Rembg {
        #[arg(required = true)]
        inputs: Vec<String>,
        /// fast = ISNet (default), high = BiRefNet (finer edges, slower)
        #[arg(long, value_enum, default_value = "fast")]
        quality: RembgQuality,
        #[arg(long, value_enum, default_value = "png")]
        format: OutputFormat,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Restore faces with GFPGAN
    Restore {
        #[arg(required = true)]
        inputs: Vec<String>,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Segment an object with SAM 2.1 (point or box prompt)
    Segment {
        input: String,
        /// Point prompt "x,y" in pixels
        #[arg(long, conflicts_with = "r#box")]
        point: Option<String>,
        /// Box prompt "x1,y1,x2,y2" in pixels
        #[arg(long = "box")]
        r#box: Option<String>,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Erase a masked region with LaMa inpainting
    Erase {
        input: String,
        /// Binary mask image (white = erase)
        #[arg(long)]
        mask: PathBuf,
        /// Dilate the mask by N pixels before inpainting (default 12)
        #[arg(long, default_value = "12")]
        dilate: u32,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Manage local models
    Models {
        #[command(subcommand)]
        action: ModelsAction,
    },
    /// Show image metadata
    Info {
        #[arg(required = true)]
        inputs: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Print machine-readable operator/parameter schema
    Capabilities {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum ModelsAction {
    /// List models and their download status
    List,
    /// Download a model (or "all")
    Pull { name: String },
    /// Remove a downloaded model
    Rm { name: String },
    /// Print the models directory
    Dir,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Upscale {
            inputs,
            format,
            common,
        } => run::upscale(inputs, format, &common),
        Command::Rembg {
            inputs,
            quality,
            format,
            common,
        } => run::rembg(inputs, quality, format, &common),
        Command::Restore { inputs, common } => run::restore(inputs, &common),
        Command::Segment {
            input,
            point,
            r#box,
            common,
        } => run::segment(&input, point.as_deref(), r#box.as_deref(), &common),
        Command::Erase {
            input,
            mask,
            dilate,
            common,
        } => run::erase(&input, &mask, dilate, &common),
        Command::Models { action } => models::run(action),
        Command::Info { inputs, json } => info::run(&inputs, json),
        Command::Capabilities { json: _ } => capabilities::run(),
    };
    ExitCode::from(code)
}
