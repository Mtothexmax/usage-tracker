use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, WebviewWindowBuilder, WebviewUrl, Manager};
use crate::db::Db;
use crate::config::AppConfig;

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows::Win32::System::SystemInformation::GetTickCount64;
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

    pub fn get_idle_time() -> Duration {
        let mut lii = LASTINPUTINFO::default();
        lii.cbSize = std::mem::size_of::<LASTINPUTINFO>() as u32;
        unsafe {
            if GetLastInputInfo(&mut lii).as_bool() {
                let now = GetTickCount64();
                let last_input_ms = lii.dwTime as u64;
                let now_32 = (now & 0xFFFFFFFF) as u32;
                let mut last_input_64 = (now & 0xFFFFFFFF00000000) | last_input_ms;
                
                if now_32 < lii.dwTime {
                    last_input_64 -= 0x100000000;
                }
                
                let idle_ms = now.saturating_sub(last_input_64);
                return Duration::from_millis(idle_ms);
            }
        }
        Duration::from_secs(0)
    }

    pub fn is_media_playing() -> bool {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync();
        if let Ok(manager) = manager.and_then(|m| m.get()) {
            if let Ok(session) = manager.GetCurrentSession() {
                if let Ok(playback_info) = session.GetPlaybackInfo() {
                    if let Ok(status) = playback_info.PlaybackStatus() {
                        return status.0 == 4;
                    }
                }
            }
        }
        false
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn get_idle_time() -> Duration {
        let output = Command::new("ioreg")
            .args(["-c", "IOHIDSystem"])
            .output()
            .ok();
        
        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("HIDIdleTime") {
                    if let Some(idle_str) = line.split('=').last() {
                        if let Ok(idle_nanos) = idle_str.trim().parse::<u64>() {
                            return Duration::from_nanos(idle_nanos);
                        }
                    }
                }
            }
        }
        Duration::from_secs(0)
    }

    pub fn is_media_playing() -> bool {
        let output = Command::new("pmset")
            .args(["-g", "assertions"])
            .output()
            .ok();
        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("PreventUserIdleDisplaySleep") && 
                   stdout.lines().any(|l| l.contains("PreventUserIdleDisplaySleep") && l.contains("1"));
        }
        false
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use super::*;
    pub fn get_idle_time() -> Duration { Duration::from_secs(0) }
    pub fn is_media_playing() -> bool { false }
}

pub fn start_tracker(handle: AppHandle, db: Arc<Mutex<Db>>, config: Arc<Mutex<AppConfig>>) {
    let mut active_seconds: u32 = 0;
    
    std::thread::spawn(move || {
        loop {
            let idle_time = platform::get_idle_time();
            let media_playing = platform::is_media_playing();
            let now = chrono::Utc::now().timestamp();
            
            let is_active = idle_time < Duration::from_secs(5) || media_playing;

            if is_active {
                {
                    let db_lock = db.lock().unwrap();
                    let _ = db_lock.record_ping();
                }
                active_seconds += 5;
                
                let (threshold_mins, enabled) = {
                    let config = config.lock().unwrap();
                    (config.break_reminder_threshold_minutes, config.break_reminder_enabled)
                };

                if enabled && active_seconds >= threshold_mins * 60 {
                    active_seconds = 0;
                    
                    let app_clone = handle.clone();
                    
                    let _ = handle.run_on_main_thread(move || {
                        if app_clone.get_webview_window("break-reminder").is_none() {
                            let w = WebviewWindowBuilder::new(&app_clone, "break-reminder", WebviewUrl::App("index.html?window=break".into()))
                                .title("Take a Break")
                                .inner_size(400.0, 200.0)
                                .always_on_top(true)
                                .resizable(true)
                                .minimizable(false)
                                .decorations(false)
                                .transparent(true)
                                .shadow(false)
                                .visible(true)
                                .build();
                        
                            if let Ok(win) = w {
                                if let Some(monitor) = win.current_monitor().ok().flatten() {
                                    let screen_size = monitor.size();
                                    let scale_factor = monitor.scale_factor();
                                    let win_width = 400.0 * scale_factor;
                                    let x = (screen_size.width as f64 - win_width) / 2.0;
                                    let _ = win.set_position(tauri::PhysicalPosition::new(x as i32, (20.0 * scale_factor) as i32));
                                }
                            }
                        }
                    });
                }
            } else {
                let break_threshold = config.lock().unwrap().break_threshold_seconds;
                if idle_time.as_secs() >= break_threshold as u64 {
                    active_seconds = 0;
                }
            }
            
            let _ = handle.emit("usage-update", (is_active, media_playing, now));
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}
