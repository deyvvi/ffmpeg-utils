use serde_json::json;

use crate::ffprobe::probe_file;
use crate::models::{Check, JobKind, MediaInfo, StreamInfo};

#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub checks: Vec<Check>,
}

impl VerifyReport {
    pub fn ok(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|check| check.ok)
    }
}

fn check_file(checks: &mut Vec<Check>, target: &str) -> bool {
    let path = std::path::Path::new(target);
    if !path.is_file() {
        checks.push(Check::new(false, "file_missing", json!({ "path": target })));
        return false;
    }
    let size = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    if size == 0 {
        checks.push(Check::new(false, "file_empty", json!({})));
        return false;
    }
    checks.push(Check::new(true, "file_exists", json!({ "size": size })));
    true
}

pub async fn verify_output(
    ffprobe: &str,
    output: &str,
    kind: JobKind,
    input: &MediaInfo,
    expected_resolution: Option<(u32, u32)>,
    expected_subtitles: usize,
    expected_language: Option<&str>,
) -> VerifyReport {
    let mut checks = Vec::new();
    if !check_file(&mut checks, output) {
        return VerifyReport { checks };
    }
    let media = match probe_file(ffprobe, output, true).await {
        Ok(media) => media,
        Err(error) => {
            checks.push(Check::new(
                false,
                "probe_failed",
                json!({ "message": error.message, "detail": error.detail }),
            ));
            return VerifyReport { checks };
        }
    };

    checks.push(Check::new(
        media.video().is_some(),
        "video_track",
        json!({}),
    ));
    if let Some(video) = media.video()
        && kind == JobKind::Normalize
    {
        let (target_width, target_height) = expected_resolution.unwrap_or((1920, 1080));
        checks.push(Check::new(
            video.width == Some(target_width) && video.height == Some(target_height),
            "resolution",
            json!({
                "width": video.width,
                "height": video.height
            }),
        ));
    }

    if kind == JobKind::Normalize {
        checks.push(Check::new(
            media.audios().len() == input.audios().len(),
            "audio_kept",
            json!({
                "kept": media.audios().len(),
                "total": input.audios().len()
            }),
        ));
    } else {
        checks.push(Check::new(
            media.audios().len() == 1,
            "audio_count",
            json!({ "count": media.audios().len() }),
        ));
        if let Some(language) = expected_language {
            let actual = media
                .audios()
                .first()
                .map(|stream| stream.language.clone())
                .unwrap_or_else(|| "und".to_string());
            checks.push(Check::new(
                actual == language,
                "audio_language",
                json!({ "language": actual }),
            ));
        }
        checks.push(Check::new(
            media
                .audios()
                .first()
                .map(|stream| stream.is_default())
                .unwrap_or(false),
            "audio_default",
            json!({}),
        ));
    }

    checks.push(Check::new(
        media.subtitles().len() == expected_subtitles,
        "subtitles",
        json!({
            "count": media.subtitles().len(),
            "expected": expected_subtitles
        }),
    ));

    if input.chapter_count > 0 {
        checks.push(Check::new(
            media.chapter_count > 0,
            "chapters",
            json!({
                "kept": media.chapter_count,
                "total": input.chapter_count
            }),
        ));
    }

    if input.duration > 0.0 && media.duration > 0.0 {
        checks.push(Check::new(
            (media.duration - input.duration).abs() <= 2.0,
            "duration",
            json!({
                "duration": media.duration,
                "original": input.duration
            }),
        ));
    }

    VerifyReport { checks }
}

pub async fn verify_trim(
    ffprobe: &str,
    output: &str,
    input: &MediaInfo,
    expected_duration: f64,
    mode: &str,
) -> VerifyReport {
    let mut checks = Vec::new();
    if !check_file(&mut checks, output) {
        return VerifyReport { checks };
    }
    let media = match probe_file(ffprobe, output, true).await {
        Ok(media) => media,
        Err(error) => {
            checks.push(Check::new(
                false,
                "probe_failed",
                json!({ "message": error.message, "detail": error.detail }),
            ));
            return VerifyReport { checks };
        }
    };
    checks.push(Check::new(
        media.video().is_some(),
        "video_track",
        json!({}),
    ));
    let tolerance = if mode == "fast" { 12.0 } else { 1.5 };
    checks.push(Check::new(
        (media.duration - expected_duration).abs() <= tolerance,
        "trim_duration",
        json!({
            "duration": media.duration,
            "requested": expected_duration
        }),
    ));
    checks.push(Check::new(
        media.audios().len() == input.audios().len(),
        "audio_kept",
        json!({
            "kept": media.audios().len(),
            "total": input.audios().len()
        }),
    ));
    checks.push(Check::new(
        media.subtitles().len() == input.subtitles().len(),
        "subtitles_kept",
        json!({
            "kept": media.subtitles().len(),
            "total": input.subtitles().len()
        }),
    ));
    if mode == "fast" {
        checks.push(Check::new(true, "fast_trim", json!({})));
    }
    VerifyReport { checks }
}

pub async fn verify_tracks(
    ffprobe: &str,
    output: &str,
    input: &MediaInfo,
    expected_audio: usize,
    expected_subtitles: usize,
) -> VerifyReport {
    let mut checks = Vec::new();
    if !check_file(&mut checks, output) {
        return VerifyReport { checks };
    }
    let media = match probe_file(ffprobe, output, true).await {
        Ok(media) => media,
        Err(error) => {
            checks.push(Check::new(
                false,
                "probe_failed",
                json!({ "message": error.message, "detail": error.detail }),
            ));
            return VerifyReport { checks };
        }
    };

    checks.push(Check::new(
        media.video().is_some(),
        "video_track",
        json!({}),
    ));
    checks.push(Check::new(
        media.audios().len() == expected_audio,
        "audio_count",
        json!({ "count": media.audios().len(), "expected": expected_audio }),
    ));
    checks.push(Check::new(
        media.subtitles().len() == expected_subtitles,
        "subtitles",
        json!({ "count": media.subtitles().len(), "expected": expected_subtitles }),
    ));

    let default_audio = if expected_audio > 0 {
        media
            .audios()
            .iter()
            .filter(|stream| stream.is_default())
            .count()
            == 1
    } else {
        media.audios().is_empty()
    };
    checks.push(Check::new(default_audio, "default_audio", json!({})));

    let default_subtitle = if expected_subtitles > 0 {
        media
            .subtitles()
            .iter()
            .filter(|stream| stream.is_default())
            .count()
            == 1
    } else {
        media.subtitles().is_empty()
    };
    checks.push(Check::new(default_subtitle, "default_subtitle", json!({})));

    if input.duration > 0.0 && media.duration > 0.0 {
        checks.push(Check::new(
            (media.duration - input.duration).abs() <= 2.0,
            "duration",
            json!({ "duration": media.duration, "original": input.duration }),
        ));
    }

    VerifyReport { checks }
}

pub async fn verify_extract(ffprobe: &str, output: &str, stream: &StreamInfo) -> VerifyReport {
    let mut checks = Vec::new();
    if !check_file(&mut checks, output) {
        return VerifyReport { checks };
    }
    let media = match probe_file(ffprobe, output, false).await {
        Ok(media) => media,
        Err(error) => {
            checks.push(Check::new(
                false,
                "probe_failed",
                json!({ "message": error.message, "detail": error.detail }),
            ));
            return VerifyReport { checks };
        }
    };
    let extracted = media.of_type(&stream.kind);
    checks.push(Check::new(
        extracted.len() == 1,
        "extract_stream",
        json!({
            "count": extracted.len(),
            "kind": stream.kind
        }),
    ));
    VerifyReport { checks }
}
