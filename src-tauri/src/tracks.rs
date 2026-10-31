use std::collections::HashMap;

use crate::languages::normalize_language;
use crate::models::{MediaInfo, StreamInfo, SubtitlePlan};

pub const MKV_SUBTITLE_COPY: [&str; 6] = [
    "subrip",
    "ass",
    "ssa",
    "webvtt",
    "hdmv_pgs_subtitle",
    "dvd_subtitle",
];

pub const SUBTITLE_CONVERT_TO_SRT: [&str; 11] = [
    "mov_text",
    "microdvd",
    "sami",
    "realtext",
    "subviewer",
    "subviewer1",
    "jacosub",
    "mpl2",
    "pjs",
    "vplayer",
    "stl",
];

pub fn find_audio<'a>(media: &'a MediaInfo, language: &str) -> Vec<&'a StreamInfo> {
    let canonical = normalize_language(language);
    let mut candidates: Vec<&StreamInfo> = media
        .audios()
        .into_iter()
        .filter(|stream| stream.language == canonical)
        .collect();
    candidates.sort_by_key(|stream| {
        let title = stream.title.to_lowercase();
        let hint = if canonical == "spa" {
            if title.contains("latino") || title.contains("latin") {
                0
            } else if title.contains("castellano") {
                2
            } else {
                1
            }
        } else {
            1
        };
        (if stream.is_default() { 0 } else { 1 }, hint, stream.index)
    });
    candidates
}

pub fn subtitle_plan(media: &MediaInfo) -> SubtitlePlan {
    let mut indices = Vec::new();
    let mut conversions: HashMap<i64, String> = HashMap::new();
    let mut excluded = Vec::new();
    let mut warnings = Vec::new();

    for stream in media.subtitles() {
        let codec = stream.codec.to_lowercase();
        let label = if stream.title.is_empty() {
            format!("track {}", stream.index)
        } else {
            stream.title.clone()
        };
        if matches!(codec.as_str(), "teletext" | "eia_608") {
            excluded.push(format!("{label} ({codec})"));
            warnings.push(format!(
                "Subtitle “{label}” ({codec}) is not compatible with MKV and will be omitted."
            ));
            continue;
        }
        indices.push(stream.index);
        if SUBTITLE_CONVERT_TO_SRT.contains(&codec.as_str()) {
            conversions.insert(stream.index, "srt".to_string());
            warnings.push(format!(
                "Subtitle “{label}” ({codec}) will be converted to SRT; formatting or styles may be lost."
            ));
        } else if !MKV_SUBTITLE_COPY.contains(&codec.as_str()) {
            warnings.push(format!(
                "Subtitle “{label}” uses codec “{codec}”; it will be copied, but it may not be compatible with MKV."
            ));
        }
    }

    SubtitlePlan {
        indices,
        conversions,
        excluded,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::models::{MediaInfo, StreamInfo};

    fn stream(index: i64, kind: &str, codec: &str, language: &str, title: &str) -> StreamInfo {
        StreamInfo {
            index,
            kind: kind.to_string(),
            codec: codec.to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: language.to_string(),
            raw_language: language.to_string(),
            title: title.to_string(),
            channels: if kind == "audio" { Some(2) } else { None },
            channel_layout: String::new(),
            bit_rate: None,
            width: None,
            height: None,
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: HashMap::new(),
            duration: None,
        }
    }

    fn media(streams: Vec<StreamInfo>) -> MediaInfo {
        MediaInfo {
            path: PathBuf::from(r"C:\Videos\test.mkv")
                .to_string_lossy()
                .to_string(),
            size: 1,
            duration: 10.0,
            format_name: "matroska".to_string(),
            format_long_name: "Matroska".to_string(),
            tags: HashMap::new(),
            chapter_count: 0,
            streams,
            is_full_hd_canvas: false,
        }
    }

    #[test]
    fn selects_english_track() {
        let media = media(vec![
            stream(1, "audio", "aac", "spa", ""),
            stream(2, "audio", "aac", "eng", ""),
        ]);
        assert_eq!(find_audio(&media, "en")[0].index, 2);
    }

    #[test]
    fn ignores_titles_for_language() {
        let mut track = stream(4, "audio", "aac", "spa", "English");
        track.disposition.insert("default".to_string(), 1);
        let media = media(vec![stream(2, "audio", "aac", "eng", ""), track]);
        assert!(
            find_audio(&media, "eng")
                .iter()
                .all(|stream| stream.index != 4)
        );
    }

    #[test]
    fn subtitle_plan_copies_and_converts() {
        let media = media(vec![
            stream(3, "subtitle", "subrip", "spa", ""),
            stream(4, "subtitle", "mov_text", "spa", ""),
        ]);
        let plan = subtitle_plan(&media);
        assert_eq!(plan.indices, vec![3, 4]);
        assert_eq!(plan.conversions.get(&4).map(String::as_str), Some("srt"));
        assert_eq!(plan.warnings.len(), 1);
    }
}
