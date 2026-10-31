use crate::config::FfmpegTools;
use crate::languages::normalize_language;
use crate::models::{AmfStatus, MediaInfo, SubtitlePlan};
use crate::resolution::CanvasPlan;

fn amf_args(amf: &AmfStatus, quality: &str, rc: &str, qp_i: i32, qp_p: i32) -> Vec<String> {
    let has = |name: &str| amf.options.iter().any(|option| option == name);
    let mut args = Vec::new();
    if has("quality") {
        args.extend(["-quality".to_string(), quality.to_string()]);
    }
    if has("rc") {
        args.extend(["-rc".to_string(), rc.to_string()]);
    }
    if rc == "cqp" {
        if has("qp_i") {
            args.extend(["-qp_i".to_string(), qp_i.to_string()]);
        }
        if has("qp_p") {
            args.extend(["-qp_p".to_string(), qp_p.to_string()]);
        }
    }
    args
}

fn tail(output: &str, no_overwrite: bool) -> Vec<String> {
    vec![
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        if no_overwrite { "-n" } else { "-y" }.to_string(),
        output.to_string(),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn build_normalize_command(
    tools: &FfmpegTools,
    media: &MediaInfo,
    output: &str,
    video_mode: &str,
    canvas: &CanvasPlan,
    subtitles: &SubtitlePlan,
    amf: &AmfStatus,
    quality: &str,
    qp_i: i32,
    qp_p: i32,
    software_crf: i32,
    no_overwrite: bool,
) -> Result<Vec<String>, String> {
    if !matches!(
        video_mode,
        "copy" | "h264_amf" | "h264_nvenc" | "h264_qsv" | "libx264"
    ) {
        return Err(format!("Invalid video encoder: {video_mode}"));
    }
    if video_mode == "copy" && canvas.filter_chain.is_some() {
        return Err("The video cannot be kept if scaling or padding is required.".to_string());
    }
    let video = media
        .video()
        .ok_or_else(|| "The file has no usable video track.".to_string())?;
    let mut command = vec![
        tools.ffmpeg.clone(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        media.path.clone(),
        "-map".to_string(),
        format!("0:{}", video.index),
    ];
    for stream in media.audios() {
        command.extend(["-map".to_string(), format!("0:{}", stream.index)]);
    }
    for index in &subtitles.indices {
        command.extend(["-map".to_string(), format!("0:{index}")]);
    }
    command.extend(["-map".to_string(), "0:t?".to_string()]);

    if video_mode == "copy" {
        command.extend(["-c".to_string(), "copy".to_string()]);
    } else {
        if let Some(filter) = &canvas.filter_chain {
            command.extend(["-vf".to_string(), filter.clone()]);
        }
        match video_mode {
            "h264_amf" => {
                command.extend(["-c:v".to_string(), "h264_amf".to_string()]);
                command.extend(amf_args(amf, quality, "cqp", qp_i, qp_p));
            }
            "h264_nvenc" => {
                let preset = match quality {
                    "speed" => "p3",
                    "quality" => "p7",
                    _ => "p5",
                };
                let cq = match quality {
                    "speed" => "26",
                    "quality" => "20",
                    _ => "23",
                };
                command.extend([
                    "-c:v".to_string(),
                    "h264_nvenc".to_string(),
                    "-preset".to_string(),
                    preset.to_string(),
                    "-rc".to_string(),
                    "vbr".to_string(),
                    "-cq".to_string(),
                    cq.to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                ]);
            }
            "h264_qsv" => {
                let preset = match quality {
                    "speed" => "veryfast",
                    "quality" => "slow",
                    _ => "medium",
                };
                let q = match quality {
                    "speed" => "26",
                    "quality" => "20",
                    _ => "23",
                };
                command.extend([
                    "-c:v".to_string(),
                    "h264_qsv".to_string(),
                    "-preset".to_string(),
                    preset.to_string(),
                    "-global_quality".to_string(),
                    q.to_string(),
                ]);
            }
            _ => {
                command.extend([
                    "-c:v".to_string(),
                    "libx264".to_string(),
                    "-preset".to_string(),
                    "medium".to_string(),
                    "-crf".to_string(),
                    software_crf.to_string(),
                ]);
            }
        }
        command.extend([
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-c:s".to_string(),
            "copy".to_string(),
            "-c:t".to_string(),
            "copy".to_string(),
            "-metadata:s:v:0".to_string(),
            "rotate=0".to_string(),
        ]);
    }
    for (position, index) in subtitles.indices.iter().enumerate() {
        if let Some(codec) = subtitles.conversions.get(index) {
            command.extend([format!("-c:s:{position}"), codec.clone()]);
        }
    }
    command.extend([
        "-map_metadata".to_string(),
        "0".to_string(),
        "-map_chapters".to_string(),
        "0".to_string(),
    ]);
    command.extend(tail(output, no_overwrite));
    Ok(command)
}

pub fn build_single_audio_command(
    tools: &FfmpegTools,
    media: &MediaInfo,
    output: &str,
    audio_index: i64,
    subtitles: &SubtitlePlan,
    no_overwrite: bool,
) -> Result<Vec<String>, String> {
    let audio = media
        .audios()
        .into_iter()
        .find(|stream| stream.index == audio_index)
        .ok_or_else(|| format!("Audio track {audio_index} does not exist in the file."))?;
    let video = media
        .video()
        .ok_or_else(|| "The file has no usable video track.".to_string())?;
    let mut command = vec![
        tools.ffmpeg.clone(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        media.path.clone(),
        "-map".to_string(),
        format!("0:{}", video.index),
        "-map".to_string(),
        format!("0:{}", audio.index),
    ];
    for index in &subtitles.indices {
        command.extend(["-map".to_string(), format!("0:{index}")]);
    }
    command.extend([
        "-map".to_string(),
        "0:t?".to_string(),
        "-c".to_string(),
        "copy".to_string(),
    ]);
    let language = normalize_language(&audio.language);
    let title = if audio.title.trim().is_empty() {
        format!("Audio {language}")
    } else {
        audio.title.clone()
    };
    command.extend([
        "-metadata:s:a:0".to_string(),
        format!("language={language}"),
        "-metadata:s:a:0".to_string(),
        format!("title={title}"),
        "-disposition:a:0".to_string(),
        "default".to_string(),
    ]);
    for (position, index) in subtitles.indices.iter().enumerate() {
        if let Some(codec) = subtitles.conversions.get(index) {
            command.extend([format!("-c:s:{position}"), codec.clone()]);
        }
    }
    command.extend([
        "-map_metadata".to_string(),
        "0".to_string(),
        "-map_chapters".to_string(),
        "0".to_string(),
    ]);
    command.extend(tail(output, no_overwrite));
    Ok(command)
}

fn quote_argument(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if value
        .chars()
        .any(|character| character.is_whitespace() || "\"&|<>^()".contains(character))
    {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

pub fn display_command(command: &[String]) -> String {
    command
        .iter()
        .map(|value| quote_argument(value))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::models::{MediaInfo, StreamInfo, SubtitlePlan};
    use crate::resolution::compute_canvas;

    fn tools() -> FfmpegTools {
        FfmpegTools {
            ffmpeg: "ffmpeg".to_string(),
            ffprobe: "ffprobe".to_string(),
        }
    }

    fn amf() -> AmfStatus {
        AmfStatus {
            available: true,
            message: String::new(),
            detail: String::new(),
            options: vec![
                "quality".to_string(),
                "rc".to_string(),
                "qp_i".to_string(),
                "qp_p".to_string(),
            ],
        }
    }

    fn stream(index: i64, kind: &str) -> StreamInfo {
        StreamInfo {
            index,
            kind: kind.to_string(),
            codec: "h264".to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: "und".to_string(),
            raw_language: String::new(),
            title: String::new(),
            channels: if kind == "audio" { Some(2) } else { None },
            channel_layout: String::new(),
            bit_rate: None,
            width: if kind == "video" { Some(1920) } else { None },
            height: if kind == "video" { Some(804) } else { None },
            fps: None,
            sample_aspect_ratio: if kind == "video" {
                "1:1".to_string()
            } else {
                String::new()
            },
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: HashMap::new(),
            duration: None,
        }
    }

    fn media() -> MediaInfo {
        MediaInfo {
            path: "in.mkv".to_string(),
            size: 1,
            duration: 10.0,
            format_name: String::new(),
            format_long_name: String::new(),
            tags: HashMap::new(),
            chapter_count: 0,
            streams: vec![
                stream(0, "video"),
                stream(1, "audio"),
                stream(2, "audio"),
                stream(3, "subtitle"),
            ],
            is_full_hd_canvas: false,
        }
    }

    #[test]
    fn normalize_pads_and_maps_absolute_indexes() {
        let media = media();
        let canvas = compute_canvas(1920, 804, "1:1", "", 0, 1920, 1080);
        let subtitles = SubtitlePlan {
            indices: vec![3],
            conversions: HashMap::new(),
            excluded: Vec::new(),
            warnings: Vec::new(),
        };
        let command = build_normalize_command(
            &tools(),
            &media,
            "out.mkv",
            "h264_amf",
            &canvas,
            &subtitles,
            &amf(),
            "speed",
            20,
            22,
            20,
            true,
        )
        .unwrap();
        assert!(
            command
                .windows(2)
                .any(|pair| pair == ["-vf", "pad=1920:1080:0:138:black"])
        );
        assert_eq!(command.iter().filter(|value| *value == "-map").count(), 5);
        assert!(command.contains(&"-n".to_string()));
    }

    #[test]
    fn single_audio_sets_language_and_default() {
        let subtitles = SubtitlePlan {
            indices: vec![3],
            conversions: HashMap::new(),
            excluded: Vec::new(),
            warnings: Vec::new(),
        };
        let command =
            build_single_audio_command(&tools(), &media(), "out.mkv", 2, &subtitles, true).unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:2"]));
        assert!(command.contains(&"language=und".to_string()));
        assert!(
            command
                .windows(2)
                .any(|pair| pair == ["-disposition:a:0", "default"])
        );
        assert!(!command.contains(&"-vf".to_string()));
    }

    #[test]
    fn maps_hardware_encoders() {
        let media = media();
        let canvas = compute_canvas(1920, 804, "1:1", "", 0, 1280, 720);
        let subtitles = SubtitlePlan {
            indices: Vec::new(),
            conversions: HashMap::new(),
            excluded: Vec::new(),
            warnings: Vec::new(),
        };
        for (mode, codec) in [
            ("h264_amf", "h264_amf"),
            ("h264_nvenc", "h264_nvenc"),
            ("h264_qsv", "h264_qsv"),
            ("libx264", "libx264"),
        ] {
            let command = build_normalize_command(
                &tools(),
                &media,
                "out.mkv",
                mode,
                &canvas,
                &subtitles,
                &amf(),
                "balanced",
                20,
                22,
                20,
                true,
            )
            .unwrap();
            assert!(command.windows(2).any(|pair| pair == ["-c:v", codec]));
        }
    }
}
