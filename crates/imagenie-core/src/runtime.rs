use ort::session::{Session, builder::GraphOptimizationLevel};

use crate::ImageProcessingError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExecutionProvider {
    #[default]
    Cpu,
    CoreMl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeOptions {
    pub execution_provider: ExecutionProvider,
    pub intra_threads: usize,
    pub memory_pattern: bool,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            execution_provider: ExecutionProvider::Cpu,
            intra_threads: 4,
            memory_pattern: true,
        }
    }
}

impl RuntimeOptions {
    /// Product default for the convolution-heavy 2x upscaler.
    pub fn upscale() -> Self {
        let logical_cpus = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(4);
        Self {
            intra_threads: logical_cpus.min(12),
            ..Self::default()
        }
    }

    /// Product default for BiRefNet. Disabling ORT's memory pattern avoids
    /// retaining a second multi-gigabyte activation allocation on CPU.
    pub fn rembg_high() -> Self {
        Self {
            memory_pattern: false,
            ..Self::default()
        }
    }
}

pub(crate) fn build_session(
    model_path: &str,
    options: &RuntimeOptions,
) -> Result<Session, ImageProcessingError> {
    if options.intra_threads == 0 {
        return Err(ImageProcessingError::Processing(
            "runtime intra_threads must be at least 1".to_string(),
        ));
    }

    let builder = Session::builder()
        .map_err(|e| ImageProcessingError::Ort(e.to_string()))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| ImageProcessingError::Ort(e.to_string()))?
        .with_intra_threads(options.intra_threads)
        .map_err(|e| ImageProcessingError::Ort(e.to_string()))?
        .with_memory_pattern(options.memory_pattern)
        .map_err(|e| ImageProcessingError::Ort(e.to_string()))?;

    #[cfg(target_os = "macos")]
    let mut builder = match options.execution_provider {
        ExecutionProvider::Cpu => builder,
        ExecutionProvider::CoreMl => builder
            .with_execution_providers([ort::ep::CoreML::default()
                .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
                .with_compute_units(ort::ep::coreml::ComputeUnits::All)
                .build()])
            .map_err(|e| ImageProcessingError::Ort(e.to_string()))?,
    };

    #[cfg(not(target_os = "macos"))]
    let mut builder = match options.execution_provider {
        ExecutionProvider::Cpu => builder,
        ExecutionProvider::CoreMl => {
            return Err(ImageProcessingError::Processing(
                "CoreML execution provider is only available on macOS".to_string(),
            ));
        }
    };

    builder
        .commit_from_file(model_path)
        .map_err(|e| ImageProcessingError::Ort(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_runtime_is_stable_cpu_configuration() {
        assert_eq!(
            RuntimeOptions::default().execution_provider,
            ExecutionProvider::Cpu
        );
        assert_eq!(RuntimeOptions::default().intra_threads, 4);
        assert!(RuntimeOptions::default().memory_pattern);
    }

    #[test]
    fn specialized_runtime_defaults_preserve_cpu_provider() {
        assert_eq!(
            RuntimeOptions::upscale().execution_provider,
            ExecutionProvider::Cpu
        );
        assert!((1..=12).contains(&RuntimeOptions::upscale().intra_threads));
        assert!(!RuntimeOptions::rembg_high().memory_pattern);
    }
}
