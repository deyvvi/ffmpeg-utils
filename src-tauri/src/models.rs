use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub index: i64,
    #[serde(rename = "type")]
    pub kind: String,
    pub codec: String,
    pub codec_long: String,
    pub profile: String,
    pub language: String,
    pub raw_language: String,
    pub title: String,
    pub channels: Option<u32>,
    pub channel_layout: String,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub sample_aspect_ratio: String,
    pub display_aspect_ratio: String,
    pub rotation: i32,
    pub disposition: HashMap<String, i32>,
    pub duration: Option<f64>,
}

impl StreamInfo {
    pub fn is_default(&self) -> bool {
        self.disposition.get("default").copied().unwrap_or(0) != 0
    }

    pub fn is_attached_pic(&self) -> bool {
        self.disposition.get("attached_pic").copied().unwrap_or(0) != 0
    }

    pub fn is_forced(&self) -> bool {
        self.disposition.get("forced").copied().unwrap_or(0) != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub path: String,
    pub size: u64,
    pub duration: f64,
    pub format_name: String,
    pub format_long_name: String,
    pub tags: HashMap<String, String>,
    pub chapter_count: usize,
    pub streams: Vec<StreamInfo>,
    pub is_full_hd_canvas: bool,
}

impl MediaInfo {
    pub fn of_type(&self, kind: &str) -> Vec<&StreamInfo> {
        self.streams
            .iter()
            .filter(|stream| stream.kind == kind)
            .collect()
    }

    pub fn video(&self) -> Option<&StreamInfo> {
        let videos: Vec<&StreamInfo> = self.of_type("video");
        videos
            .iter()
            .copied()
            .find(|stream| !stream.is_attached_pic())
            .or_else(|| videos.first().copied())
    }

    pub fn audios(&self) -> Vec<&StreamInfo> {
        self.of_type("audio")
    }

    pub fn subtitles(&self) -> Vec<&StreamInfo> {
        self.of_type("subtitle")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobKind {
    Normalize,
    SingleAudio,
    Trim,
    EditTracks,
    Extract,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub ok: bool,
    pub code: String,
    pub params: Value,
}

impl Check {
    pub fn new(ok: bool, code: impl Into<String>, params: Value) -> Self {
        Self {
            ok,
            code: code.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobFinished {
    pub job_id: String,
    pub ok: bool,
    pub kind: JobKind,
    pub message: String,
    pub code: String,
    pub detail: String,
    pub outputs: Vec<String>,
    pub checks: Vec<Check>,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: String,
    pub percent: i32,
    pub detail: String,
    pub speed: String,
    pub out_time: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub job_id: String,
    pub line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSuggestion {
    pub path: String,
    pub exists: bool,
    pub alternate: String,
    pub stream_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPlan {
    pub commands: Vec<Vec<String>>,
    pub display: Vec<String>,
    pub notes: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsStatus {
    pub found: bool,
    pub ffmpeg: String,
    pub ffprobe: String,
    pub version: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderStatus {
    pub id: String,
    pub label: String,
    pub codec: String,
    pub vendor: String,
    pub hardware: bool,
    pub available: bool,
    pub message: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderReport {
    pub encoders: Vec<EncoderStatus>,
    pub amf: AmfStatus,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub config: crate::config::Config,
    pub tools: ToolsStatus,
    pub initial_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfStatus {
    pub available: bool,
    pub message: String,
    pub detail: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SubtitlePlan {
    pub indices: Vec<i64>,
    pub conversions: HashMap<i64, String>,
    pub excluded: Vec<String>,
    pub warnings: Vec<String>,
}

fn default_extract_format() -> String {
    "copy".to_string()
}

fn default_target_width() -> u32 {
    1920
}

fn default_target_height() -> u32 {
    1080
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractItemRequest {
    pub stream_index: i64,
    pub output: String,
    #[serde(default)]
    pub replace: bool,
    #[serde(default = "default_extract_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractChoice {
    pub stream_index: i64,
    #[serde(default = "default_extract_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTrackChoice {
    pub stream_index: i64,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddedTrackRequest {
    pub path: String,
    pub stream_index: i64,
    pub kind: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub forced: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum JobRequest {
    Normalize {
        path: String,
        output: String,
        video_mode: String,
        #[serde(default = "default_target_width")]
        width: u32,
        #[serde(default = "default_target_height")]
        height: u32,
        #[serde(default)]
        replace: bool,
    },
    Trim {
        path: String,
        output: String,
        start: f64,
        end: f64,
        mode: String,
        #[serde(default)]
        replace: bool,
    },
    SingleAudio {
        path: String,
        output: String,
        language: String,
        audio_index: Option<i64>,
        #[serde(default)]
        replace: bool,
    },
    EditTracks {
        path: String,
        output: String,
        audio: Vec<SourceTrackChoice>,
        subtitles: Vec<SourceTrackChoice>,
        added: Vec<AddedTrackRequest>,
        #[serde(default)]
        replace: bool,
    },
    Extract {
        path: String,
        items: Vec<ExtractItemRequest>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OutputRequest {
    Normalize {
        path: String,
        #[serde(default = "default_target_width")]
        width: u32,
        #[serde(default = "default_target_height")]
        height: u32,
    },
    Trim {
        path: String,
        start: f64,
        end: f64,
    },
    SingleAudio {
        path: String,
        language: String,
    },
    EditTracks {
        path: String,
    },
    Extract {
        path: String,
        output_dir: String,
        items: Vec<ExtractChoice>,
    },
}
