use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const APP_DIR_NAME: &str = "FFmpegUtils";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub language: String,
    pub theme: String,
    pub accent: String,
    pub density: String,
    pub ffmpeg_dir: String,
    pub output_dir: String,
    pub quality: String,
    pub qp_i: i32,
    pub qp_p: i32,
    pub software_crf: i32,
    pub window_width: u32,
    pub window_height: u32,
    pub log_visible: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "dark".to_string(),
            accent: "#77756f".to_string(),
            density: "comfortable".to_string(),
            ffmpeg_dir: String::new(),
            output_dir: String::new(),
            quality: "speed".to_string(),
            qp_i: 20,
            qp_p: 22,
            software_crf: 20,
            window_width: 1180,
            window_height: 780,
            log_visible: false,
        }
    }
}

fn value_string(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(*key).and_then(Value::as_str) {
            return Some(text.to_string());
        }
    }
    None
}

fn value_i32(value: &Value, keys: &[&str]) -> Option<i32> {
    for key in keys {
        if let Some(number) = value.get(*key).and_then(Value::as_i64) {
            return Some(number as i32);
        }
    }
    None
}

fn value_u32(value: &Value, keys: &[&str]) -> Option<u32> {
    value_i32(value, keys).and_then(|number| u32::try_from(number).ok())
}

impl Config {
    pub fn from_value(value: &Value) -> Self {
        let mut config = Config::default();
        if let Some(text) = value_string(value, &["language"]) {
            config.language = text;
        }
        if let Some(text) = value_string(value, &["theme"]) {
            config.theme = text;
        }
        if let Some(text) = value_string(value, &["accent"]) {
            config.accent = text;
        }
        if let Some(text) = value_string(value, &["density"]) {
            config.density = text;
        }
        if let Some(text) = value_string(value, &["ffmpegDir", "ffmpeg_dir"]) {
            config.ffmpeg_dir = text;
        }
        if let Some(text) = value_string(value, &["outputDir", "output_dir"]) {
            config.output_dir = text;
        }
        if let Some(text) = value_string(value, &["quality"]) {
            config.quality = text;
        }
        if let Some(number) = value_i32(value, &["qpI", "qp_i"]) {
            config.qp_i = number;
        }
        if let Some(number) = value_i32(value, &["qpP", "qp_p"]) {
            config.qp_p = number;
        }
        if let Some(number) = value_i32(value, &["softwareCrf", "software_crf"]) {
            config.software_crf = number;
        }
        if let Some(number) = value_u32(value, &["windowWidth", "window_width"]) {
            config.window_width = number;
        }
        if let Some(number) = value_u32(value, &["windowHeight", "window_height"]) {
            config.window_height = number;
        }
        if let Some(visible) = value
            .get("logVisible")
            .and_then(Value::as_bool)
            .or_else(|| value.get("log_visible").and_then(Value::as_bool))
        {
            config.log_visible = visible;
        }
        config
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn logs_dir() -> PathBuf {
    config_dir().join("logs")
}

fn legacy_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(config) = dirs::config_dir() {
        paths.push(config.join("MVK_AMD").join("config.json"));
    }
    paths
}

pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(text) = fs::read_to_string(&path)
        && let Ok(value) = serde_json::from_str::<Value>(&text)
    {
        return Config::from_value(&value);
    }
    for legacy in legacy_paths() {
        if let Ok(text) = fs::read_to_string(legacy)
            && let Ok(value) = serde_json::from_str::<Value>(&text)
        {
            return Config::from_value(&value);
        }
    }
    Config::default()
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let directory = config_dir();
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(directory.join("config.json"), text).map_err(|error| error.to_string())
}

#[derive(Debug, Clone)]
pub struct FfmpegTools {
    pub ffmpeg: String,
    pub ffprobe: String,
}
