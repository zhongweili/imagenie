#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use imagenie_lib::app;

fn main() -> anyhow::Result<()> {
    let ctx = tauri::generate_context!();
    app()?
        .run(ctx)
        .expect("error while running tauri application");

    Ok(())
}
