use tracing::info;

#[tauri::command]
pub fn backend_add(number: i32) -> i32 {
    // Note: these commands block the main thread and hang the UI until they return.
    // If you need to run a long-running task, use async command instead.
    info!("Backend was called with an argument: {}", number);
    number + 2
}
