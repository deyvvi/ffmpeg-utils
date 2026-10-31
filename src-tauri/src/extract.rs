use std::path::{Path, PathBuf};

use crate::config::FfmpegTools;
use crate::languages::normalize_language;
use crate::models::{MediaInfo, StreamInfo};

const AUDIO_EXTENSIONS: [(&str, &str); 10] = [
    ("aac", "m4a"),
    ("ac3", "ac3"),
    ("eac3", "eac3"),
    ("dts", "dts"),
    ("flac", "flac"),
    ("opus", "opus"),
    ("vorbis", "ogg"),
    ("mp3", "mp3"),
    ("mp2", "mp3"),
    ("alac", "m4a"),
];

const SUBTITLE_EXTENSIONS: [(&str, &str); 6] = [
    ("subrip", "srt"),
    ("ass", "ass"),
    ("ssa", "ssa"),
    ("webvtt", "vtt"),
    ("hdmv_pgs_subtitle", "sup"),
    ("mov_text", "srt"),
];

pub fn audio_extension(stream: &StreamInfo) -> &'static str {
    let codec = stream.codec.to_lowercase();
    let profile = stream.profile.to_lowercase();
    if codec.starts_with("pcm_") {
        return "wav";
    }
    if codec == "dts" && profile.contains("hd") {
        return "mka";
    }
    if matches!(codec.as_str(), "truehd" | "mlp") {
        return "mka";
    }
    AUDIO_EXTENSIONS
        .iter()
        .find(|(key, _)| *key == codec)
        .map(|(_, extension)| *extension)
        .unwrap_or("mka")
}

pub fn subtitle_extension(stream: &StreamInfo) -> (&'static str, Option<&'static str>) {
    let codec = stream.codec.to_lowercase();
    let convert = if codec == "mov_text" {
        Some("srt")
    } else {
        None
    };
    let extension = SUBTITLE_EXTENSIONS
        .iter()
        .find(|(key, _)| *key == codec)
        .map(|(_, extension)| *extension)
        .unwrap_or("mks");
    (extension, convert)
}

fn subtitle_is_text(stream: &StreamInfo) -> bool {
    !matches!(
        stream.codec.to_lowercase().as_str(),
        "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle" | "xsub" | "dvb_teletext"
    )
}

pub fn format_extension(stream: &StreamInfo, format: &str) -> Result<&'static str, String> {
    let format = if format.trim().is_empty() {
        "copy".to_string()
    } else {
        format.trim().to_lowercase()
    };
    match stream.kind.as_str() {
        "audio" => match format.as_str() {
            "copy" => Ok(audio_extension(stream)),
            "mp3" => Ok("mp3"),
            "m4a" | "aac" => Ok("m4a"),
            "flac" => Ok("flac"),
            "wav" => Ok("wav"),
            "ac3" => Ok("ac3"),
            "opus" => Ok("opus"),
            other => Err(format!("Unsupported audio format: {other}")),
        },
        "subtitle" => match format.as_str() {
            "copy" => Ok(subtitle_extension(stream).0),
            "srt" => {
                if !subtitle_is_text(stream) {
                    return Err(
                        "Bitmap subtitles cannot be converted to SRT without OCR.".to_string()
                    );
                }
                Ok("srt")
            }
            "vtt" => {
                if !subtitle_is_text(stream) {
                    return Err(
                        "Bitmap subtitles cannot be converted to WebVTT without OCR.".to_string(),
                    );
                }
                Ok("vtt")
            }
            other => Err(format!("Unsupported subtitle format: {other}")),
        },
        _ => Err("Only audio and subtitle streams can be extracted.".to_string()),
    }
}

pub fn extraction_output(
    input: &Path,
    stream: &StreamInfo,
    output_dir: Option<&Path>,
    format: &str,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let kind = if stream.kind == "audio" {
        "audio"
    } else {
        "subtitle"
    };
    let extension = format_extension(stream, format)?;
    let language = normalize_language(&stream.language);
    let codec_token = if format.trim().is_empty() || format.eq_ignore_ascii_case("copy") {
        let codec = if stream.codec.is_empty() {
            "unknown".to_string()
        } else {
            stream.codec.to_lowercase()
        };
        format!(".{codec}")
    } else {
        String::new()
    };
    let name = format!(
        "{stem}.{kind}.{}.{language}{codec_token}.{extension}",
        stream.index
    );
    Ok(match output_dir {
        Some(directory) => directory.join(name),
        None => input.with_file_name(name),
    })
}

pub fn build_extract_command(
    tools: &FfmpegTools,
    media: &MediaInfo,
    stream: &StreamInfo,
    output: &Path,
    format: &str,
    no_overwrite: bool,
) -> Result<Vec<String>, String> {
    let format = if format.trim().is_empty() {
        "copy".to_string()
    } else {
        format.trim().to_lowercase()
    };
    format_extension(stream, &format)?;

    let mut command = vec![
        tools.ffmpeg.clone(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        media.path.clone(),
        "-map".to_string(),
        format!("0:{}", stream.index),
    ];
    match stream.kind.as_str() {
        "audio" => {
            command.push("-c:a".to_string());
            command.push(
                match format.as_str() {
                    "copy" => "copy",
                    "mp3" => "libmp3lame",
                    "m4a" | "aac" => "aac",
                    "flac" => "flac",
                    "wav" => "pcm_s16le",
                    "ac3" => "ac3",
                    "opus" => "libopus",
                    _ => return Err(format!("Unsupported audio format: {format}")),
                }
                .to_string(),
            );
            match format.as_str() {
                "mp3" => command.extend([
                    "-b:a".to_string(),
                    "192k".to_string(),
                    "-ac".to_string(),
                    "2".to_string(),
                ]),
                "m4a" | "aac" => command.extend(["-b:a".to_string(), "192k".to_string()]),
                "ac3" => command.extend(["-b:a".to_string(), "192k".to_string()]),
                "opus" => command.extend(["-b:a".to_string(), "128k".to_string()]),
                _ => {}
            }
            command.push("-metadata:s:a:0".to_string());
            command.push(format!("language={}", normalize_language(&stream.language)));
            if !stream.title.is_empty() {
                command.push("-metadata:s:a:0".to_string());
                command.push(format!("title={}", stream.title));
            }
        }
        "subtitle" => {
            let codec = match format.as_str() {
                "copy" => subtitle_extension(stream).1.unwrap_or("copy"),
                "srt" => "srt",
                "vtt" => "webvtt",
                _ => return Err(format!("Unsupported subtitle format: {format}")),
            };
            command.push("-c:s".to_string());
            command.push(codec.to_string());
            command.push("-metadata:s:s:0".to_string());
            command.push(format!("language={}", normalize_language(&stream.language)));
            if !stream.title.is_empty() {
                command.push("-metadata:s:s:0".to_string());
                command.push(format!("title={}", stream.title));
            }
        }
        _ => return Err("Only audio and subtitle streams can be extracted.".to_string()),
    }
    command.extend([
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
        if no_overwrite { "-n" } else { "-y" }.to_string(),
        output.to_string_lossy().to_string(),
    ]);
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_mapping() {
        let mut stream = StreamInfo {
            index: 2,
            kind: "audio".to_string(),
            codec: "eac3".to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: "eng".to_string(),
            raw_language: "eng".to_string(),
            title: String::new(),
            channels: Some(6),
            channel_layout: String::new(),
            bit_rate: None,
            width: None,
            height: None,
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: std::collections::HashMap::new(),
            duration: None,
        };
        assert_eq!(audio_extension(&stream), "eac3");
        stream.codec = "mov_text".to_string();
        stream.kind = "subtitle".to_string();
        assert_eq!(subtitle_extension(&stream), ("srt", Some("srt")));
        assert_eq!(format_extension(&stream, "vtt").unwrap(), "vtt");
        stream.codec = "hdmv_pgs_subtitle".to_string();
        assert_eq!(subtitle_extension(&stream), ("sup", None));
        assert!(format_extension(&stream, "srt").is_err());
    }

    #[test]
    fn extraction_name() {
        let stream = StreamInfo {
            index: 5,
            kind: "subtitle".to_string(),
            codec: "ass".to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: "eng".to_string(),
            raw_language: "eng".to_string(),
            title: String::new(),
            channels: None,
            channel_layout: String::new(),
            bit_rate: None,
            width: None,
            height: None,
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: std::collections::HashMap::new(),
            duration: None,
        };
        let path =
            extraction_output(Path::new(r"C:\Videos\Movie.mkv"), &stream, None, "copy").unwrap();
        assert_eq!(
            path.file_name().unwrap().to_string_lossy(),
            "Movie.subtitle.5.eng.ass.ass"
        );
        let converted =
            extraction_output(Path::new(r"C:\Videos\Movie.mkv"), &stream, None, "srt").unwrap();
        assert_eq!(
            converted.file_name().unwrap().to_string_lossy(),
            "Movie.subtitle.5.eng.srt"
        );
    }

    #[test]
    fn audio_conversion_uses_target_encoder() {
        let stream = StreamInfo {
            index: 1,
            kind: "audio".to_string(),
            codec: "eac3".to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: "spa".to_string(),
            raw_language: "spa".to_string(),
            title: "Latin".to_string(),
            channels: Some(6),
            channel_layout: String::new(),
            bit_rate: None,
            width: None,
            height: None,
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: std::collections::HashMap::new(),
            duration: None,
        };
        let media = MediaInfo {
            path: "in.mkv".to_string(),
            size: 1,
            duration: 10.0,
            format_name: String::new(),
            format_long_name: String::new(),
            tags: std::collections::HashMap::new(),
            chapter_count: 0,
            streams: vec![stream.clone()],
            is_full_hd_canvas: false,
        };
        let tools = FfmpegTools {
            ffmpeg: "ffmpeg".to_string(),
            ffprobe: "ffprobe".to_string(),
        };
        let command =
            build_extract_command(&tools, &media, &stream, Path::new("out.mp3"), "mp3", true)
                .unwrap();
        assert!(
            command
                .windows(2)
                .any(|pair| pair == ["-c:a", "libmp3lame"])
        );
        assert!(command.windows(2).any(|pair| pair == ["-ac", "2"]));
        assert!(command.windows(2).any(|pair| pair == ["-b:a", "192k"]));
        assert!(command.contains(&"title=Latin".to_string()));
    }

    #[test]
    fn subtitle_conversion_and_bitmap_validation() {
        let mut stream = StreamInfo {
            index: 3,
            kind: "subtitle".to_string(),
            codec: "ass".to_string(),
            codec_long: String::new(),
            profile: String::new(),
            language: "eng".to_string(),
            raw_language: "eng".to_string(),
            title: String::new(),
            channels: None,
            channel_layout: String::new(),
            bit_rate: None,
            width: None,
            height: None,
            fps: None,
            sample_aspect_ratio: String::new(),
            display_aspect_ratio: String::new(),
            rotation: 0,
            disposition: std::collections::HashMap::new(),
            duration: None,
        };
        assert_eq!(format_extension(&stream, "vtt").unwrap(), "vtt");
        stream.codec = "hdmv_pgs_subtitle".to_string();
        assert!(format_extension(&stream, "srt").is_err());
        assert_eq!(format_extension(&stream, "copy").unwrap(), "sup");
    }
}
