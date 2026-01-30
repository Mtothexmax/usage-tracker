mod db;
mod config;
mod tracker;

use std::sync::{Arc, Mutex};
use tauri::{Manager, State, WindowEvent, WebviewWindowBuilder, WebviewUrl};
use chrono::{Utc, Duration as ChronoDuration};
use crate::db::{Db, UsageDay};
use crate::config::AppConfig;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

struct AppState {
    db: Arc<Mutex<Db>>,
    config: Arc<Mutex<AppConfig>>,
}

#[tauri::command]
async fn get_usage_data(state: State<'_, AppState>, days: i64) -> Result<Vec<UsageDay>, String> {
    let db = state.db.lock().unwrap();
    let config = state.config.lock().unwrap();
    let now = Utc::now();
    let end_date = now;
    let start_date = now - ChronoDuration::days(days + 1);
    db.get_usage_for_range(start_date, end_date, config.break_threshold_seconds).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
async fn update_config(app: tauri::AppHandle, state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    *state.config.lock().unwrap() = config.clone();
    
    let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
    let config_path = app_data_dir.join("config.json");
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
async fn center_window(window: tauri::Window) -> Result<(), String> {
    window.center().map_err(|e| e.to_string())
}

#[tauri::command]
async fn resize_window(window: tauri::Window, width: f64, height: f64) -> Result<(), String> {
    window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height))).map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_break_reminder(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("break-reminder") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let _window = WebviewWindowBuilder::new(&app, "break-reminder", WebviewUrl::App("index.html?window=break".into()))
        .title("Take a Break")
        .inner_size(400.0, 200.0)
        .always_on_top(true)
        .resizable(true)
        .minimizable(false)
        .decorations(false)
        .shadow(false)
        .transparent(true)
        .visible(true)
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;
    
    // Position at top center
    if let Some(monitor) = _window.current_monitor().ok().flatten() {
        let screen_size: tauri::PhysicalSize<u32> = *monitor.size();
        let scale_factor = monitor.scale_factor();
        let win_width = 400.0 * scale_factor;
        let x = (screen_size.width as f64 - win_width) / 2.0;
        let _ = _window.set_position(tauri::PhysicalPosition::new(x as i32, (20.0 * scale_factor) as i32));
    }

    Ok(())
}

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main").map(|w| {
                let _ = w.show();
                let _ = w.set_focus();
            });
        }))
        .setup(|app| {
            // ... same setup code ...
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => { app.exit(0); }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("usage.db");
            let config_path = app_data_dir.join("config.json");

            let config = if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).unwrap_or_default();
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                let default_config = AppConfig::default();
                let _ = std::fs::write(&config_path, serde_json::to_string_pretty(&default_config).unwrap());
                default_config
            };

            let db = Arc::new(Mutex::new(Db::init(db_path).expect("failed to init db")));
            let config = Arc::new(Mutex::new(config));
            
            app.manage(AppState { 
                db: db.clone(),
                config: config.clone(),
            });

            tracker::start_tracker(app.handle().clone(), db, config);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Don't hide the break reminder window, allow it to close
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_usage_data, get_config, update_config, show_break_reminder, close_window, center_window, resize_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}