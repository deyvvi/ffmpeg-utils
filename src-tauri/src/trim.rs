use crate::config::FfmpegTools;
use crate::models::{AmfStatus, MediaInfo};

#[allow(dead_code)]
pub fn parse_timecode(value: &str) -> Result<f64, String> {
    let text = value.trim().replace(',', ".");
    if text.is_empty() {
        return Err("Time cannot be empty.".to_string());
    }
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() > 3 {
        return Err("Use SS, MM:SS or HH:MM:SS.".to_string());
    }
    let mut numbers = Vec::with_capacity(parts.len());
    for part in &parts {
        let number: f64 = part
            .parse()
            .map_err(|_| "Invalid time format.".to_string())?;
        if number < 0.0 {
            return Err("Time cannot be negative.".to_string());
        }
        numbers.push(number);
    }
    if numbers.len() >= 2 && numbers[numbers.len() - 1] >= 60.0 {
        return Err("Seconds must be less than 60 when using minutes.".to_string());
    }
    if numbers.len() == 3 && numbers[1] >= 60.0 {
        return Err("Minutes must be less than 60 when using hours.".to_string());
    }
    let mut seconds = 0.0;
    for number in numbers {
        seconds = seconds * 60.0 + number;
    }
    Ok(seconds)
}

pub fn format_timecode(seconds: f64) -> String {
    let total = seconds.max(0.0);
    let hours = (total / 3600.0).floor() as u64;
    let minutes = ((total % 3600.0) / 60.0).floor() as u64;
    let secs = total % 60.0;
    let whole = secs.floor() as u64;
    let millis = ((secs - whole as f64) * 1000.0).round() as u64;
    if millis >= 1000 {
        return format_timecode(total + 0.001);
    }
    format!("{hours:02}:{minutes:02}:{whole:02}.{millis:03}")
}

pub fn validate_range(start: f64, end: f64, duration: f64) -> Result<(), String> {
    if start < 0.0 {
        return Err("The start time cannot be negative.".to_string());
    }
    if end <= start {
        return Err("The end time must be after the start time.".to_string());
    }
    if duration > 0.0 && end > duration + 0.001 {
        return Err("The end time exceeds the file duration.".to_string());
    }
    Ok(())
}

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

#[allow(clippy::too_many_arguments)]
pub fn build_trim_command(
    tools: &FfmpegTools,
    media: &MediaInfo,
    output: &str,
    start: f64,
    end: f64,
    mode: &str,
    amf: &AmfStatus,
    quality: &str,
    qp_i: i32,
    qp_p: i32,
    software_crf: i32,
    no_overwrite: bool,
) -> Result<Vec<String>, String> {
    if !matches!(mode, "fast" | "precise" | "software") {
        return Err(format!("Invalid trim mode: {mode}"));
    }
    let duration = (end - start).max(0.0);
    let timestamp = format_timecode(start);
    let mut command = vec![tools.ffmpeg.clone(), "-hide_banner".to_string()];
    if mode == "fast" {
        command.extend([
            "-ss".to_string(),
            timestamp,
            "-i".to_string(),
            media.path.clone(),
        ]);
    } else {
        command.extend([
            "-i".to_string(),
            media.path.clone(),
            "-ss".to_string(),
            timestamp,
        ]);
    }
    command.extend(["-t".to_string(), format!("{duration:.3}")]);

    if mode == "fast" {
        command.extend([
            "-map".to_string(),
            "0".to_string(),
            "-c".to_string(),
            "copy".to_string(),
        ]);
    } else {
        let video = media
            .video()
            .ok_or_else(|| "The file has no video track.".to_string())?;
        command.extend(["-map".to_string(), format!("0:{}", video.index)]);
        for stream in media.audios() {
            command.extend(["-map".to_string(), format!("0:{}", stream.index)]);
        }
        for stream in media.subtitles() {
            command.extend(["-map".to_string(), format!("0:{}", stream.index)]);
        }
        command.extend(["-map".to_string(), "0:t?".to_string()]);
        if mode == "software" {
            command.extend([
                "-c:v".to_string(),
                "libx264".to_string(),
                "-preset".to_string(),
                "medium".to_string(),
                "-crf".to_string(),
                software_crf.to_string(),
            ]);
        } else {
            command.extend(["-c:v".to_string(), "h264_amf".to_string()]);
            command.extend(amf_args(amf, quality, "cqp", qp_i, qp_p));
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
    command.extend([
        "-avoid_negative_ts".to_string(),
        "make_zero".to_string(),
        "-map_metadata".to_string(),
        "0".to_string(),
        "-map_chapters".to_string(),
        "-1".to_string(),
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
    use super::*;

    #[test]
    fn parses_times() {
        assert!((parse_timecode("90").unwrap() - 90.0).abs() < 0.001);
        assert!((parse_timecode("1:30").unwrap() - 90.0).abs() < 0.001);
        assert!((parse_timecode("01:02:03,500").unwrap() - 3723.5).abs() < 0.001);
        assert!(parse_timecode("1:90").is_err());
        assert!(parse_timecode("").is_err());
    }

    #[test]
    fn formats_times() {
        assert_eq!(format_timecode(90.0), "00:01:30.000");
        assert_eq!(format_timecode(3723.5), "01:02:03.500");
    }

    #[test]
    fn fast_seek_is_before_input() {
        let media = MediaInfo {
            path: "in.mkv".to_string(),
            size: 1,
            duration: 10.0,
            format_name: String::new(),
            format_long_name: String::new(),
            tags: std::collections::HashMap::new(),
            chapter_count: 0,
            streams: Vec::new(),
            is_full_hd_canvas: false,
        };
        let tools = FfmpegTools {
            ffmpeg: "ffmpeg".to_string(),
            ffprobe: "ffprobe".to_string(),
        };
        let amf = AmfStatus {
            available: true,
            message: String::new(),
            detail: String::new(),
            options: vec![
                "quality".to_string(),
                "rc".to_string(),
                "qp_i".to_string(),
                "qp_p".to_string(),
            ],
        };
        let command = build_trim_command(
            &tools, &media, "out.mkv", 1.0, 5.0, "fast", &amf, "speed", 20, 22, 20, true,
        )
        .unwrap();
        assert!(
            command.iter().position(|value| value == "-ss").unwrap()
                < command.iter().position(|value| value == "-i").unwrap()
        );
    }
}
