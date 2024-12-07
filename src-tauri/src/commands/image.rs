#[tauri::command]
pub async fn check_image_dimensions(input_path: &str) -> Result<(u32, u32), String> {
    let dimensions = image::image_dimensions(input_path).unwrap();
    Ok(dimensions)
}
