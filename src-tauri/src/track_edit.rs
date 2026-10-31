use std::collections::HashSet;

use crate::config::FfmpegTools;
use crate::languages::normalize_language;
use crate::models::{MediaInfo, SourceTrackChoice, StreamInfo};
use crate::tracks::SUBTITLE_CONVERT_TO_SRT;

#[derive(Debug, Clone)]
pub struct AddedTrackSpec {
    pub path: String,
    pub stream_index: i64,
    pub kind: String,
    pub codec: String,
    pub language: String,
    pub title: String,
    pub default: bool,
    pub forced: bool,
}

fn subtitle_conversion(codec: &str) -> Result<Option<&'static str>, String> {
    let codec = codec.to_lowercase();
    if matches!(codec.as_str(), "teletext" | "eia_608") {
        return Err(format!(
            "Subtitle codec {codec} is not compatible with Matroska."
        ));
    }
    if SUBTITLE_CONVERT_TO_SRT.contains(&codec.as_str()) {
        Ok(Some("srt"))
    } else {
        Ok(None)
    }
}

fn choose_source<'a>(
    media: &'a MediaInfo,
    choices: &[SourceTrackChoice],
    kind: &str,
) -> Result<Vec<&'a StreamInfo>, String> {
    let mut seen = HashSet::new();
    let mut streams = Vec::new();
    for choice in choices {
        if !seen.insert(choice.stream_index) {
            return Err(format!(
                "Track {} is selected more than once.",
                choice.stream_index
            ));
        }
        let stream = media
            .streams
            .iter()
            .find(|stream| stream.index == choice.stream_index)
            .ok_or_else(|| format!("Track {} does not exist.", choice.stream_index))?;
        if stream.kind != kind {
            return Err(format!(
                "Track {} is not a {kind} track.",
                choice.stream_index
            ));
        }
        streams.push(stream);
    }
    Ok(streams)
}

fn disposition(default: bool, forced: bool) -> &'static str {
    match (default, forced) {
        (true, true) => "default+forced",
        (true, false) => "default",
        (false, true) => "forced",
        (false, false) => "0",
    }
}

pub fn build_edit_tracks_command(
    tools: &FfmpegTools,
    media: &MediaInfo,
    output: &str,
    audio_choices: &[SourceTrackChoice],
    subtitle_choices: &[SourceTrackChoice],
    added: &[AddedTrackSpec],
    no_overwrite: bool,
) -> Result<Vec<String>, String> {
    let video = media
        .video()
        .ok_or_else(|| "The file has no usable video track.".to_string())?;
    let kept_audio = choose_source(media, audio_choices, "audio")?;
    let kept_subtitles = choose_source(media, subtitle_choices, "subtitle")?;
    let added_audio: Vec<&AddedTrackSpec> =
        added.iter().filter(|track| track.kind == "audio").collect();
    let added_subtitles: Vec<&AddedTrackSpec> = added
        .iter()
        .filter(|track| track.kind == "subtitle")
        .collect();

    let audio_defaults = audio_choices.iter().filter(|choice| choice.default).count()
        + added_audio.iter().filter(|track| track.default).count();
    if audio_defaults > 1 {
        return Err("Only one audio track can be the default.".to_string());
    }
    if (!kept_audio.is_empty() || !added_audio.is_empty()) && audio_defaults == 0 {
        return Err("Choose one default audio track.".to_string());
    }
    let subtitle_defaults = subtitle_choices
        .iter()
        .filter(|choice| choice.default)
        .count()
        + added_subtitles.iter().filter(|track| track.default).count();
    if subtitle_defaults > 1 {
        return Err("Only one subtitle track can be the default.".to_string());
    }
    if (!kept_subtitles.is_empty() || !added_subtitles.is_empty()) && subtitle_defaults == 0 {
        return Err("Choose one default subtitle track.".to_string());
    }

    for track in added {
        if !matches!(track.kind.as_str(), "audio" | "subtitle") {
            return Err(format!("Unsupported added track kind: {}", track.kind));
        }
    }

    let mut input_paths: Vec<&str> = Vec::new();
    for track in added {
        if !input_paths.contains(&track.path.as_str()) {
            input_paths.push(&track.path);
        }
    }

    let mut command = vec![
        tools.ffmpeg.clone(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        media.path.clone(),
    ];
    for path in &input_paths {
        command.extend(["-i".to_string(), (*path).to_string()]);
    }

    command.extend(["-map".to_string(), format!("0:{}", video.index)]);
    for stream in &kept_audio {
        command.extend(["-map".to_string(), format!("0:{}", stream.index)]);
    }
    for stream in &kept_subtitles {
        command.extend(["-map".to_string(), format!("0:{}", stream.index)]);
    }
    command.extend(["-map".to_string(), "0:t?".to_string()]);

    for track in added {
        let input_index = input_paths
            .iter()
            .position(|path| *path == track.path)
            .map(|index| index + 1)
            .ok_or_else(|| format!("Input {} was not added.", track.path))?;
        command.extend([
            "-map".to_string(),
            format!("{input_index}:{}", track.stream_index),
        ]);
    }

    command.extend(["-c".to_string(), "copy".to_string()]);

    for (position, choice) in audio_choices.iter().enumerate() {
        command.extend([
            format!("-disposition:a:{position}"),
            disposition(choice.default, false).to_string(),
        ]);
    }
    for (offset, track) in added_audio.iter().enumerate() {
        let position = kept_audio.len() + offset;
        command.extend([
            format!("-metadata:s:a:{position}"),
            format!("language={}", normalize_language(&track.language)),
        ]);
        if !track.title.trim().is_empty() {
            command.extend([
                format!("-metadata:s:a:{position}"),
                format!("title={}", track.title.trim()),
            ]);
        }
        command.extend([
            format!("-disposition:a:{position}"),
            disposition(track.default, false).to_string(),
        ]);
    }

    for (position, choice) in subtitle_choices.iter().enumerate() {
        let forced = kept_subtitles
            .get(position)
            .is_some_and(|stream| stream.is_forced());
        command.extend([
            format!("-disposition:s:{position}"),
            disposition(choice.default, forced).to_string(),
        ]);
        if let Some(stream) = kept_subtitles.get(position)
            && let Some(codec) = subtitle_conversion(&stream.codec)?
        {
            command.extend([format!("-c:s:{position}"), codec.to_string()]);
        }
    }
    for (offset, track) in added_subtitles.iter().enumerate() {
        let position = kept_subtitles.len() + offset;
        command.extend([
            format!("-metadata:s:s:{position}"),
            format!("language={}", normalize_language(&track.language)),
        ]);
        if !track.title.trim().is_empty() {
            command.extend([
                format!("-metadata:s:s:{position}"),
                format!("title={}", track.title.trim()),
            ]);
        }
        command.extend([
            format!("-disposition:s:{position}"),
            disposition(track.default, track.forced).to_string(),
        ]);
        if let Some(codec) = subtitle_conversion(&track.codec)? {
            command.extend([format!("-c:s:{position}"), codec.to_string()]);
        }
    }

    command.extend([
        "-map_metadata".to_string(),
        "0".to_string(),
        "-map_chapters".to_string(),
        "0".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        if no_overwrite { "-n" } else { "-y" }.to_string(),
        output.to_string(),
    ]);
    Ok(command)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::config::FfmpegTools;
    use crate::models::{MediaInfo, SourceTrackChoice, StreamInfo};

    fn stream(index: i64, kind: &str, codec: &str, language: &str) -> StreamInfo {
        StreamInfo {
            index,
            kind: kind.to_string(),
            codec: codec.to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: language.to_string(),
            raw_language: language.to_string(),
            title: String::new(),
            channels: if kind == "audio" { Some(2) } else { None },
            channel_layout: String::new(),
            bit_rate: None,
            width: if kind == "video" { Some(1920) } else { None },
            height: if kind == "video" { Some(1080) } else { None },
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: HashMap::new(),
            duration: None,
        }
    }

    fn media() -> MediaInfo {
        MediaInfo {
            path: "source.mkv".to_string(),
            size: 1,
            duration: 10.0,
            format_name: String::new(),
            format_long_name: String::new(),
            tags: HashMap::new(),
            chapter_count: 0,
            streams: vec![
                stream(0, "video", "h264", "und"),
                stream(1, "audio", "aac", "spa"),
                stream(2, "audio", "ac3", "eng"),
                stream(3, "subtitle", "subrip", "spa"),
            ],
            is_full_hd_canvas: true,
        }
    }

    fn tools() -> FfmpegTools {
        FfmpegTools {
            ffmpeg: "ffmpeg".to_string(),
            ffprobe: "ffprobe".to_string(),
        }
    }

    #[test]
    fn maps_kept_source_and_added_tracks() {
        let command = build_edit_tracks_command(
            &tools(),
            &media(),
            "out.mkv",
            &[SourceTrackChoice {
                stream_index: 2,
                default: true,
            }],
            &[SourceTrackChoice {
                stream_index: 3,
                default: false,
            }],
            &[
                AddedTrackSpec {
                    path: "extra.aac".to_string(),
                    stream_index: 0,
                    kind: "audio".to_string(),
                    codec: "aac".to_string(),
                    language: "spa".to_string(),
                    title: "Latino".to_string(),
                    default: false,
                    forced: false,
                },
                AddedTrackSpec {
                    path: "extra.srt".to_string(),
                    stream_index: 0,
                    kind: "subtitle".to_string(),
                    codec: "mov_text".to_string(),
                    language: "eng".to_string(),
                    title: "English".to_string(),
                    default: true,
                    forced: true,
                },
            ],
            true,
        )
        .unwrap();

        assert_eq!(command.iter().filter(|value| *value == "-i").count(), 3);
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:0"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:2"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:3"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "1:0"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "2:0"]));
        assert!(command.contains(&"language=spa".to_string()));
        assert!(command.contains(&"language=eng".to_string()));
        assert!(command.contains(&"title=Latino".to_string()));
        assert!(command.contains(&"title=English".to_string()));
        assert!(
            command
                .windows(2)
                .any(|pair| pair == ["-disposition:a:0", "default"])
        );
        assert!(
            command
                .windows(2)
                .any(|pair| pair == ["-disposition:s:1", "default+forced"])
        );
        assert!(command.windows(2).any(|pair| pair == ["-c:s:1", "srt"]));
        assert!(command.contains(&"-n".to_string()));
    }

    #[test]
    fn rejects_duplicate_defaults() {
        let error = build_edit_tracks_command(
            &tools(),
            &media(),
            "out.mkv",
            &[
                SourceTrackChoice {
                    stream_index: 1,
                    default: true,
                },
                SourceTrackChoice {
                    stream_index: 2,
                    default: true,
                },
            ],
            &[],
            &[],
            true,
        )
        .unwrap_err();
        assert!(error.contains("Only one audio"));
    }
}
