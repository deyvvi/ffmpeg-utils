mod command;
mod commands;
mod config;
mod errors;
mod extract;
mod ffmpeg;
mod ffprobe;
mod jobs;
mod languages;
mod logging;
mod models;
mod naming;
mod process;
mod resolution;
mod state;
mod track_edit;
mod tracks;
mod trim;
mod verify;

use config::load_config;
use state::AppState;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{LogicalSize, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = load_config();
    let boot_theme = match config.theme.as_str() {
        "light" => "light",
        "system" => "system",
        _ => "dark",
    };
    let boot_density = if config.density == "compact" {
        "compact"
    } else {
        "comfortable"
    };
    let boot_script = format!(
        "window.__FFMPEG_BOOT__ = Object.freeze({{ theme: \"{boot_theme}\", density: \"{boot_density}\" }});"
    );

    tauri::Builder::default()
        .append_invoke_initialization_script(boot_script)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(config.clone(), None))
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(LogicalSize::new(
                    config.window_width as f64,
                    config.window_height as f64,
                ));
                let theme = if config.theme == "light" {
                    tauri::Theme::Light
                } else {
                    tauri::Theme::Dark
                };
                let _ = window.set_theme(Some(theme));
                if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                }
                let _ = window.set_title("FFmpeg Utils");
                let effect = if config.theme == "light" {
                    Effect::MicaLight
                } else {
                    Effect::MicaDark
                };
                let _ = window.set_effects(EffectsBuilder::new().effect(effect).build());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::save_config_command,
            commands::detect_tools_command,
            commands::check_amf_command,
            commands::check_encoders_command,
            commands::probe_file_command,
            commands::probe_track_file_command,
            commands::preview_plan_command,
            commands::start_job_command,
            commands::cancel_job_command,
            commands::suggest_outputs_command,
            commands::path_exists,
            commands::set_window_size,
            commands::set_window_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FFmpeg Utils");
}
