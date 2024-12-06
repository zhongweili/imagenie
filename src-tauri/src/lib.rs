mod commands;
mod image;
mod utils;

use std::error::Error;

use commands::{
    background_removal::background_removal,
    face_restoration::face_restoration,
    upscaling::{upscale_image, upscale_images},
};
use tauri::{
    menu::{Menu, MenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadPayload,
    App, AppHandle, Builder, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_shell::ShellExt;
use tracing::info;
use utils::log_dir;

pub fn app() -> anyhow::Result<Builder<Wry>> {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(logger().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            face_restoration,
            upscale_image,
            upscale_images,
            background_removal
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

    let _webview = builder.build()?;

    #[cfg(debug_assertions)]
    {
        _webview.open_devtools();
    }

    Ok(())
}

fn page_load_handler(window: &tauri::Webview, _payload: &PageLoadPayload<'_>) {
    info!("Page loaded on {}", window.label());
}

fn window_event_handler(window: &tauri::Window, event: &tauri::WindowEvent) {
    // debug!("Window event: {:?} on {}", event, window.label());

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
            "about",
            "About",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "open",
            "Open",
            true,
            Some("CmdOrCtrl+O"),
        )?)
        .separator()
        .quit()
        .build()?;

    let help_menu = SubmenuBuilder::with_id(app, "help", "Help")
        .item(&MenuItem::with_id(
            app,
            "report-issues",
            "Report Issues",
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "view-license",
            "View License",
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "toggle-devtools",
            "Toggle Developer Tools",
            true,
            Some("CmdOrCtrl+Shift+I"),
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

    let menu = Menu::with_items(app, &[&file_menu, &help_menu])?;
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        info!("menu event: {:?}", event);
        match event.id.as_ref() {
            "open" => open_main(app).unwrap(),
            "about" => {
                app.dialog()
                    .message(format!(
                        "Imagenie v{}\n\nAn AI-powered image processing tool.\n\nBuilt with Rust and Tauri.",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .kind(MessageDialogKind::Info)
                    .title("About Imagenie")
                    .show(|_| {})
            }
            "report-issues" => {
                app.shell()
                    .open("https://github.com/zhongweili/imagenie/issues/new", None)
                    .unwrap();
            }
            "view-license" => {
                app.shell()
                    .open("https://github.com/zhongweili/imagenie/blob/main/LICENSE", None)
                    .unwrap();
            }
            "toggle-devtools" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
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
