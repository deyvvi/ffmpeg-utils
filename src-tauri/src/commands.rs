use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, LogicalSize, Manager, State};

use crate::config::{Config, FfmpegTools, save_config};
use crate::errors::{AppError, AppResult};
use crate::ffmpeg::{check_amf, check_encoders, detect_tools, tools_status};
use crate::ffprobe::probe_file;
use crate::jobs;
use crate::models::{
    AmfStatus, AppInfo, CommandPlan, EncoderReport, JobRequest, MediaInfo, OutputRequest,
    OutputSuggestion, ToolsStatus,
};
use crate::state::AppState;

async fn tools_for(state: &AppState) -> AppResult<FfmpegTools> {
    if let Some(tools) = state.tools.lock().unwrap().clone() {
        return Ok(tools);
    }
    let config = state.config.lock().unwrap().clone();
    let tools = detect_tools(&config).await?;
    *state.tools.lock().unwrap() = Some(tools.clone());
    Ok(tools)
}

#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    let config = state.config.lock().unwrap().clone();
    let status = tools_status(&config).await;
    if status.found {
        *state.tools.lock().unwrap() = Some(FfmpegTools {
            ffmpeg: status.ffmpeg.clone(),
            ffprobe: status.ffprobe.clone(),
        });
    } else {
        *state.tools.lock().unwrap() = None;
    }
    Ok(AppInfo {
        config,
        tools: status,
        initial_file: std::env::var("FFMPEG_UTILS_OPEN").ok(),
    })
}

#[tauri::command]
pub async fn save_config_command(config: Config, state: State<'_, AppState>) -> AppResult<()> {
    let previous = state.config.lock().unwrap().clone();
    if previous.ffmpeg_dir != config.ffmpeg_dir {
        *state.tools.lock().unwrap() = None;
    }
    *state.config.lock().unwrap() = config.clone();
    save_config(&config).map_err(|error| AppError::new("config", error))
}

#[tauri::command]
pub async fn detect_tools_command(state: State<'_, AppState>) -> AppResult<ToolsStatus> {
    let config = state.config.lock().unwrap().clone();
    let status = tools_status(&config).await;
    if status.found {
        *state.tools.lock().unwrap() = Some(FfmpegTools {
            ffmpeg: status.ffmpeg.clone(),
            ffprobe: status.ffprobe.clone(),
        });
    } else {
        *state.tools.lock().unwrap() = None;
    }
    Ok(status)
}

#[tauri::command]
pub async fn check_amf_command(state: State<'_, AppState>) -> AppResult<AmfStatus> {
    let tools = tools_for(&state).await?;
    Ok(check_amf(&tools).await)
}

#[tauri::command]
pub async fn check_encoders_command(state: State<'_, AppState>) -> AppResult<EncoderReport> {
    let tools = tools_for(&state).await?;
    Ok(check_encoders(&tools).await)
}
#[tauri::command]
pub async fn probe_file_command(path: String, state: State<'_, AppState>) -> AppResult<MediaInfo> {
    let tools = tools_for(&state).await?;
    probe_file(&tools.ffprobe, &path, true).await
}

#[tauri::command]
pub async fn probe_track_file_command(
    path: String,
    state: State<'_, AppState>,
) -> AppResult<MediaInfo> {
    let tools = tools_for(&state).await?;
    probe_file(&tools.ffprobe, &path, false).await
}

#[tauri::command]
pub async fn preview_plan_command(
    request: JobRequest,
    state: State<'_, AppState>,
) -> AppResult<CommandPlan> {
    let tools = tools_for(&state).await?;
    let config = state.config.lock().unwrap().clone();
    jobs::preview_plan(&request, &config, &tools).await
}

#[tauri::command]
pub async fn start_job_command(
    request: JobRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let tools = tools_for(&state).await?;
    let config = state.config.lock().unwrap().clone();
    jobs::start_job(request, config, tools, app, &state).await
}

#[tauri::command]
pub fn cancel_job_command(job_id: String, state: State<'_, AppState>) {
    jobs::cancel_job(&job_id, &state);
}

#[tauri::command]
pub async fn suggest_outputs_command(
    request: OutputRequest,
    state: State<'_, AppState>,
) -> AppResult<Vec<OutputSuggestion>> {
    let tools = tools_for(&state).await?;
    let config = state.config.lock().unwrap().clone();
    jobs::suggest_outputs(&request, &config, &tools).await
}

#[tauri::command]
pub fn path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub fn set_window_size(width: f64, height: f64, app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_window_theme(theme: String, app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let tauri_theme = if theme == "light" {
            tauri::Theme::Light
        } else {
            tauri::Theme::Dark
        };
        window
            .set_theme(Some(tauri_theme))
            .map_err(|error| error.to_string())?;
        let effect = if theme == "light" {
            Effect::MicaLight
        } else {
            Effect::MicaDark
        };
        let _ = window.set_effects(EffectsBuilder::new().effect(effect).build());
    }
    Ok(())
}
