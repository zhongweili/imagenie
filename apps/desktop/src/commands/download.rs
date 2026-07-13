use futures::future::join_all;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::task;
use tracing::info;

use crate::utils::models_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    name: String,
    url: String,
    version: String,
}

#[derive(Debug, Serialize, Clone)]
struct DownloadProgress {
    model_name: String,
    progress: f64,
    total_progress: f64,
}

#[tauri::command]
pub async fn check_model_exists(model_name: String) -> Result<bool, String> {
    let models_dir = models_dir();
    let model_path = models_dir.join(&model_name);

    if !models_dir.exists() {
        info!("Creating models directory");
        std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }

    info!("Checking if model exists: {}", model_name);
    Ok(!model_path.exists())
}

#[tauri::command]
pub async fn download_models(app: AppHandle, models: Vec<ModelInfo>) -> Result<(), String> {
    let client = Client::new();

    let models_dir = models_dir();
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let total_models = models.len();

    // Create download tasks for each model
    let download_tasks: Vec<_> = models
        .into_iter()
        .enumerate()
        .map(|(index, model_info)| {
            let client = client.clone();
            let models_dir = models_dir.clone();
            let app_handle = app.clone();

            task::spawn(async move {
                download_single_model(
                    &client,
                    model_info,
                    models_dir,
                    app_handle,
                    total_models,
                    index,
                )
                .await
            })
        })
        .collect();

    // Wait for all downloads to complete
    join_all(download_tasks)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    Ok(())
}

async fn download_single_model(
    client: &Client,
    model_info: ModelInfo,
    models_dir: PathBuf,
    app: AppHandle,
    total_models: usize,
    model_index: usize,
) -> Result<(), String> {
    info!("Starting download for model: {}", model_info.name);

    let file_path = models_dir.join(&model_info.name);
    let mut file = File::create(file_path).map_err(|e| e.to_string())?;

    let res = client
        .get(&model_info.url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let total_size = res.content_length().unwrap_or(0);
    let mut downloaded = 0u64;

    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;

        downloaded += chunk.len() as u64;
        let model_progress = (downloaded as f64 / total_size as f64) * 100.0;

        // Calculate total progress considering all models
        let total_progress = ((model_index as f64 * 100.0) + model_progress) / total_models as f64;

        // Emit progress event
        let progress = DownloadProgress {
            model_name: model_info.name.clone(),
            progress: model_progress,
            total_progress,
        };
        app.emit("download-progress", progress).unwrap();
    }

    // Save version info
    let version_info = serde_json::json!({
        "version": model_info.version,
        "lastUpdated": chrono::Utc::now().to_rfc3339()
    });

    std::fs::write(
        models_dir.join("version.json"),
        serde_json::to_string_pretty(&version_info).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    println!("Download completed for model: {}", model_info.name);
    Ok(())
}
