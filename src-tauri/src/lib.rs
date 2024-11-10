mod commands;
mod image;
mod utils;

use std::error::Error;

use commands::{
    face_restoration::face_restoration, upscaling::upscale_image, upscaling::upscale_images,
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadPayload,
    App, AppHandle, Builder, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tauri_plugin_log::{Target, TargetKind};
use tracing::{debug, info};
use utils::log_dir;

pub fn app() -> anyhow::Result<Builder<Wry>> {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(logger().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            face_restoration,
            upscale_image,
            upscale_images
        ])
        .setup(setup)
        .on_page_load(page_load_handler)
        .on_window_event(window_event_handler);
    Ok(builder)
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    info!("Setting up application");

    let handle = app.handle();

    #[cfg(desktop)]
    {
        handle.plugin(tauri_plugin_window_state::Builder::default().build())?;
    }

    setup_menu(app)?;

    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default());

    #[cfg(desktop)]
    {
        builder = builder
            .user_agent(&format!(
                "{} {} - {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            ))
            .title("Imagenie")
            .inner_size(1200., 800.)
            .min_inner_size(800., 600.)
            .center()
            .resizable(true)
            .content_protected(true);
    }

    let webview = builder.build()?;

    #[cfg(debug_assertions)]
    {
        webview.open_devtools();
    }

    Ok(())
}

fn page_load_handler(window: &tauri::Webview, _payload: &PageLoadPayload<'_>) {
    info!("Page loaded on {}", window.label());
}

fn window_event_handler(window: &tauri::Window, event: &tauri::WindowEvent) {
    debug!("Window event: {:?} on {}", event, window.label());

    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        info!("Window close requested on {}", window.label());
        if window.label() == "main" {
            api.prevent_close();
            window.hide().unwrap();
        }
    }
}

fn logger() -> tauri_plugin_log::Builder {
    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::Folder {
                path: log_dir(),
                file_name: None,
            }),
            Target::new(TargetKind::Webview),
        ])
        .level(tracing::log::LevelFilter::Debug)
}

fn setup_menu<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn Error>> {
    let icon = app.default_window_icon().unwrap().clone();
    let file_menu = SubmenuBuilder::with_id(app, "file", "File")
        .item(&MenuItem::with_id(
            app,
            "open",
            "Open",
            true,
            Some("CmdOrCtrl+O"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "save",
            "Save",
            true,
            Some("CmdOrCtrl+S"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "saveas",
            "Save As",
            true,
            Some("CmdOrCtrl+Shift+S"),
        )?)
        .separator()
        .quit()
        .build()?;
    let edit_menu = SubmenuBuilder::with_id(app, "edit", "Edit")
        .item(&MenuItem::with_id(
            app,
            "process",
            "Process",
            true,
            Some("CmdOrCtrl+P"),
        )?)
        .separator()
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .item(&CheckMenuItem::with_id(
            app,
            "checkme",
            "Check Me",
            true,
            true,
            None::<&str>,
        )?)
        .build()?;
    let tray_menu = SubmenuBuilder::with_id(app, "tray", "Tray")
        .item(&MenuItem::with_id(app, "open", "Open", true, None::<&str>)?)
        .item(&MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?)
        .separator()
        .quit()
        .build()?;

    TrayIconBuilder::with_id(format!("{}-tray", env!("CARGO_PKG_NAME")))
        .tooltip("Imagenie")
        .icon(icon)
        .menu(&tray_menu)
        .menu_on_left_click(true)
        .on_tray_icon_event(|tray, event| {
            // info!("Tray icon event: {:?}", event);
            if let TrayIconEvent::Click {
                button: MouseButton::Right,
                ..
            } = event
            {
                open_main(tray.app_handle()).unwrap();
            }
        })
        .build(app)?;

    let menu = Menu::with_items(app, &[&file_menu, &edit_menu])?;
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        info!("menu event: {:?}", event);
        match event.id.as_ref() {
            "open" => open_main(app).unwrap(),
            "save" => {}
            "saveas" => {}
            "process" => {}
            "checkme" => {
                // toggle checkme status and update config and runtime state
                // for runtime state - Arc<Mutex<State>> / ArcSwap
            }
            _ => {}
        }
    });
    Ok(())
}

fn open_main<R: Runtime>(handle: &AppHandle<R>) -> Result<(), tauri::Error> {
    handle
        .get_webview_window("main")
        .ok_or_else(|| tauri::Error::WindowNotFound)?
        .show()?;

    Ok(())
}
