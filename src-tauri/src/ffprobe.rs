use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use crate::errors::{AppError, AppResult};
use crate::languages::normalize_language;
use crate::models::{MediaInfo, StreamInfo};
use crate::resolution::{TARGET_HEIGHT, TARGET_WIDTH, compute_canvas};

fn as_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn as_u64(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn as_u32(value: Option<&Value>) -> Option<u32> {
    as_u64(value).and_then(|value| u32::try_from(value).ok())
}

fn as_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn tag(tags: Option<&Value>, name: &str) -> String {
    let Some(Value::Object(map)) = tags else {
        return String::new();
    };
    for (key, value) in map {
        if key.eq_ignore_ascii_case(name) {
            if let Value::String(text) = value {
                return text.clone();
            }
            return value.to_string();
        }
    }
    String::new()
}

fn duration_from_tag(value: &str) -> Option<f64> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn parse_fps(value: &str) -> Option<f64> {
    let (numerator, denominator) = value.split_once('/')?;
    let numerator: f64 = numerator.parse().ok()?;
    let denominator: f64 = denominator.parse().ok()?;
    if numerator <= 0.0 || denominator <= 0.0 {
        return None;
    }
    Some((numerator / denominator * 1000.0).round() / 1000.0)
}

fn rotation(stream: &Value) -> i32 {
    if let Some(items) = stream.get("side_data_list").and_then(Value::as_array) {
        for item in items {
            if let Some(value) = item.get("rotation").and_then(Value::as_f64) {
                return (value.round() as i32).rem_euclid(360);
            }
        }
    }
    let tags = stream.get("tags");
    let value = tag(tags, "rotate");
    let value = if value.is_empty() {
        tag(tags, "rotation")
    } else {
        value
    };
    if let Ok(value) = value.parse::<f64>() {
        return (value.round() as i32).rem_euclid(360);
    }
    0
}

pub fn parse_probe(data: &Value, path: Option<&str>, require_video: bool) -> AppResult<MediaInfo> {
    let format = data.get("format").cloned().unwrap_or(Value::Null);
    let raw_streams = data
        .get("streams")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if raw_streams.is_empty() {
        return Err(AppError::new(
            "no_streams",
            "FFprobe did not detect any streams in the file.",
        ));
    }

    let path_string = path
        .map(ToOwned::to_owned)
        .or_else(|| {
            format
                .get("filename")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unknown_file".to_string());

    let mut streams = Vec::with_capacity(raw_streams.len());
    for raw in &raw_streams {
        let tags = raw.get("tags");
        let disposition_value = raw.get("disposition").cloned().unwrap_or(Value::Null);
        let mut disposition = HashMap::new();
        if let Value::Object(map) = disposition_value {
            for (key, value) in map {
                if let Some(number) = value.as_i64() {
                    disposition.insert(key, number as i32);
                }
            }
        }
        let raw_language = tag(tags, "language");
        let bit_rate = as_u64(raw.get("bit_rate")).or_else(|| {
            let value = tag(tags, "BPS");
            value.parse().ok()
        });
        let duration = as_f64(raw.get("duration")).or_else(|| {
            let value = tag(tags, "DURATION");
            duration_from_tag(&value)
        });
        let fps = raw
            .get("avg_frame_rate")
            .and_then(Value::as_str)
            .and_then(parse_fps)
            .or_else(|| {
                raw.get("r_frame_rate")
                    .and_then(Value::as_str)
                    .and_then(parse_fps)
            });
        streams.push(StreamInfo {
            index: as_i64(raw.get("index")).unwrap_or(0),
            kind: raw
                .get("codec_type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            codec: raw
                .get("codec_name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            codec_long: raw
                .get("codec_long_name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            profile: raw
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            language: normalize_language(&raw_language),
            raw_language,
            title: {
                let title = tag(tags, "title");
                if title.is_empty() {
                    tag(tags, "filename")
                } else {
                    title
                }
            },
            channels: as_u32(raw.get("channels")).filter(|value| *value > 0),
            channel_layout: raw
                .get("channel_layout")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            bit_rate: bit_rate.filter(|value| *value > 0),
            width: as_u32(raw.get("width")).filter(|value| *value > 0),
            height: as_u32(raw.get("height")).filter(|value| *value > 0),
            fps,
            sample_aspect_ratio: raw
                .get("sample_aspect_ratio")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            display_aspect_ratio: raw
                .get("display_aspect_ratio")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            rotation: rotation(raw),
            disposition,
            duration,
        });
    }

    if require_video && !streams.iter().any(|stream| stream.kind == "video") {
        return Err(AppError::new(
            "no_video",
            "The file does not contain a video track.",
        ));
    }

    let mut duration = as_f64(format.get("duration")).unwrap_or(0.0);
    if duration <= 0.0 {
        duration = streams
            .iter()
            .filter_map(|stream| stream.duration)
            .fold(0.0, f64::max);
    }

    let mut size = as_u64(format.get("size")).unwrap_or(0);
    if size == 0
        && let Ok(metadata) = std::fs::metadata(&path_string)
    {
        size = metadata.len();
    }

    let tags = {
        let mut map = HashMap::new();
        if let Some(Value::Object(object)) = format.get("tags") {
            for (key, value) in object {
                if let Value::String(text) = value {
                    map.insert(key.clone(), text.clone());
                }
            }
        }
        map
    };

    let mut media = MediaInfo {
        path: path_string,
        size,
        duration,
        format_name: format
            .get("format_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        format_long_name: format
            .get("format_long_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        tags,
        chapter_count: data
            .get("chapters")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        streams,
        is_full_hd_canvas: false,
    };
    if let Some(video) = media.video()
        && let (Some(width), Some(height)) = (video.width, video.height)
    {
        let plan = compute_canvas(
            width,
            height,
            &video.sample_aspect_ratio,
            &video.display_aspect_ratio,
            video.rotation,
            TARGET_WIDTH,
            TARGET_HEIGHT,
        );
        media.is_full_hd_canvas = plan.filter_chain.is_none();
    }
    Ok(media)
}

pub async fn probe_file(ffprobe: &str, path: &str, require_video: bool) -> AppResult<MediaInfo> {
    if !Path::new(path).is_file() {
        return Err(AppError::new(
            "missing_file",
            format!("The file does not exist:\n{path}"),
        ));
    }
    let mut command = tokio::process::Command::new(ffprobe);
    command.args([
        "-v",
        "error",
        "-show_format",
        "-show_streams",
        "-show_chapters",
        "-of",
        "json",
        path,
    ]);
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let output = command.output().await.map_err(|error| {
        AppError::new("ffprobe_start", format!("Could not run FFprobe: {error}"))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let lowered = stderr.to_lowercase();
        let code = if lowered.contains("invalid data") || lowered.contains("moov atom") {
            "corrupt"
        } else if lowered.contains("permission denied") {
            "permission"
        } else {
            "probe"
        };
        return Err(AppError::new(code, "FFprobe could not read the file.")
            .with_detail(stderr.trim().to_string()));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let data: Value = serde_json::from_str(&text)?;
    parse_probe(&data, Some(path), require_video)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Value {
        let text = match name {
            "los_tipos" => include_str!("../tests/fixtures/los_tipos_malos_probe.json"),
            "coyote" => include_str!("../tests/fixtures/coyote_probe.json"),
            _ => unreachable!(),
        };
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn parses_los_tipos() {
        let media = parse_probe(&fixture("los_tipos"), None, true).unwrap();
        assert_eq!(media.video().unwrap().width, Some(1920));
        assert_eq!(media.video().unwrap().height, Some(804));
        assert_eq!(media.audios().len(), 2);
        assert_eq!(media.subtitles().len(), 3);
        assert_eq!(media.audios()[1].language, "eng");
        assert!(!media.is_full_hd_canvas);
    }

    #[test]
    fn parses_coyote_english_51() {
        let media = parse_probe(&fixture("coyote"), None, true).unwrap();
        let english = media
            .audios()
            .into_iter()
            .find(|stream| stream.language == "eng")
            .unwrap();
        assert_eq!(english.index, 2);
        assert_eq!(english.channels, Some(6));
        assert_eq!(english.codec, "eac3");
    }

    #[test]
    fn rejects_audio_only_when_video_is_required() {
        let data = serde_json::json!({
            "format": {"duration": "10.0", "size": "100"},
            "streams": [{"index": 0, "codec_type": "audio", "codec_name": "aac"}]
        });
        assert!(parse_probe(&data, None, true).is_err());
        let media = parse_probe(&data, None, false).unwrap();
        assert!(media.video().is_none());
    }
}
