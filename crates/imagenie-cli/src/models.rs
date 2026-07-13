use crate::{EXIT_OK, EXIT_PARTIAL, EXIT_USAGE, ModelsAction};
use imagenie_core::registry::{self, ModelSpec};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;

pub fn run(action: ModelsAction) -> u8 {
    match action {
        ModelsAction::List => list(),
        ModelsAction::Pull { name } => pull(&name),
        ModelsAction::Rm { name } => rm(&name),
        ModelsAction::Dir => {
            println!("{}", registry::models_dir().display());
            EXIT_OK
        }
    }
}

fn list() -> u8 {
    for spec in registry::MODELS {
        let status = if registry::is_downloaded(spec) {
            "downloaded"
        } else {
            "not downloaded"
        };
        println!(
            "{:<10} {:>7} MB  {status}",
            spec.name,
            spec.size_bytes / 1_000_000
        );
    }
    EXIT_OK
}

fn pull(name: &str) -> u8 {
    let specs: Vec<&ModelSpec> = if name == "all" {
        registry::MODELS.iter().collect()
    } else {
        match registry::find(name) {
            Some(spec) => vec![spec],
            None => {
                eprintln!(
                    "unknown model '{name}'. available: {}",
                    registry::MODELS
                        .iter()
                        .map(|m| m.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return EXIT_USAGE;
            }
        }
    };

    let mut failed = 0;
    for spec in specs {
        if registry::is_downloaded(spec) {
            eprintln!("{} already downloaded", spec.name);
            continue;
        }
        if let Err(e) = pull_one(spec) {
            eprintln!("failed to pull {}: {e}", spec.name);
            failed += 1;
        }
    }
    if failed > 0 { EXIT_PARTIAL } else { EXIT_OK }
}

pub fn pull_one(spec: &ModelSpec) -> Result<(), imagenie_core::ImageProcessingError> {
    let bar = if std::io::stderr().is_terminal() {
        let bar = ProgressBar::new(spec.size_bytes);
        bar.set_style(
            ProgressStyle::with_template(
                "{msg:<10} [{bar:30}] {bytes}/{total_bytes} ({bytes_per_sec})",
            )
            .unwrap(),
        );
        bar.set_message(spec.name.to_string());
        Some(bar)
    } else {
        eprintln!(
            "downloading {} ({} MB)...",
            spec.name,
            spec.size_bytes / 1_000_000
        );
        None
    };

    registry::download(spec, |p| {
        if let Some(bar) = &bar {
            bar.set_length(p.total);
            bar.set_position(p.downloaded);
        }
    })?;

    if let Some(bar) = bar {
        bar.finish_with_message(format!("{} done", spec.name));
    } else {
        eprintln!("{} downloaded", spec.name);
    }
    Ok(())
}

fn rm(name: &str) -> u8 {
    let Some(spec) = registry::find(name) else {
        eprintln!("unknown model '{name}'");
        return EXIT_USAGE;
    };
    if !registry::is_downloaded(spec) {
        eprintln!("{name} is not downloaded");
        return EXIT_OK;
    }
    for path in registry::model_paths(spec) {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!("failed to remove {}: {e}", path.display());
            return EXIT_PARTIAL;
        }
    }
    eprintln!("{name} removed");
    EXIT_OK
}

/// Ensure a model is available; on missing, either auto-download (--yes) or
/// print the exact pull command and fail with exit code 3.
pub fn ensure(spec: &ModelSpec, yes: bool) -> Result<(), u8> {
    if registry::is_downloaded(spec) {
        return Ok(());
    }
    if yes {
        pull_one(spec).map_err(|e| {
            eprintln!("failed to download {}: {e}", spec.name);
            crate::EXIT_MODEL_MISSING
        })
    } else {
        eprintln!(
            "model '{}' is not downloaded. run: imagenie models pull {}",
            spec.name, spec.name
        );
        Err(crate::EXIT_MODEL_MISSING)
    }
}
