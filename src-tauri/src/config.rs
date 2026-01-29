use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub break_threshold_seconds: u32,
    pub show_debug_view: bool,
    pub break_reminder_threshold_minutes: u32,
    pub break_reminder_enabled: bool,
    pub show_fullscreen_svg: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { 
            break_threshold_seconds: 120, // 2 minutes default
            show_debug_view: true,
            break_reminder_threshold_minutes: 55,
            break_reminder_enabled: true,
            show_fullscreen_svg: false,
        }
    }
}
