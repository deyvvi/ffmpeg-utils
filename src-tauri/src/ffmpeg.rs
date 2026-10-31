use std::path::{Path, PathBuf};

use crate::config::{Config, FfmpegTools};
use crate::errors::{AppError, AppResult};
use crate::models::{AmfStatus, EncoderReport, EncoderStatus, ToolsStatus};

fn executable_names() -> [&'static str; 2] {
    if cfg!(windows) {
        ["ffmpeg.exe", "ffprobe.exe"]
    } else {
        ["ffmpeg", "ffprobe"]
    }
}

fn candidate_ffmpeg_paths(config: &Config) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if !config.ffmpeg_dir.trim().is_empty() {
        let directory = PathBuf::from(&config.ffmpeg_dir);
        for name in executable_names() {
            let candidate = directory.join(name);
            if candidate.is_file() {
                candidates.push(candidate);
                break;
            }
        }
    }
    if let Ok(path) = which::which("ffmpeg") {
        candidates.push(path);
    }
    for directory in [
        PathBuf::from(r"C:\ffmpeg\bin"),
        PathBuf::from(r"C:\Program Files\ffmpeg\bin"),
        PathBuf::from(r"C:\Program Files (x86)\ffmpeg\bin"),
    ] {
        for name in executable_names() {
            let candidate = directory.join(name);
            if candidate.is_file() {
                candidates.push(candidate);
                break;
            }
        }
    }
    candidates
}

fn sibling_ffprobe(ffmpeg: &Path) -> Option<PathBuf> {
    let names = executable_names();
    let candidate = ffmpeg.with_file_name(names[1]);
    if candidate.is_file() {
        return Some(candidate);
    }
    which::which("ffprobe").ok()
}

async fn version_line(executable: &Path) -> Option<String> {
    let mut command = tokio::process::Command::new(executable);
    command.arg("-version");
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let output = command.output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::to_string)
}

pub async fn detect_tools(config: &Config) -> AppResult<FfmpegTools> {
    for ffmpeg in candidate_ffmpeg_paths(config) {
        let Some(ffprobe) = sibling_ffprobe(&ffmpeg) else {
            continue;
        };
        if version_line(&ffmpeg).await.is_none() || version_line(&ffprobe).await.is_none() {
            continue;
        }
        return Ok(FfmpegTools {
            ffmpeg: ffmpeg.to_string_lossy().to_string(),
            ffprobe: ffprobe.to_string_lossy().to_string(),
        });
    }
    Err(AppError::new(
        "tools_missing",
        "FFmpeg with FFprobe was not found. Install FFmpeg, add it to PATH, or select its folder in Settings.",
    ))
}

struct EncoderCandidate {
    id: &'static str,
    label: &'static str,
    codec: &'static str,
    vendor: &'static str,
    hardware: bool,
}

async fn test_hardware_encoder(tools: &FfmpegTools, id: &str) -> (bool, String) {
    let pix_fmt = if id == "h264_qsv" { "nv12" } else { "yuv420p" };
    let mut probe = tokio::process::Command::new(&tools.ffmpeg);
    probe.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=256x144:d=0.3",
        "-frames:v",
        "1",
        "-pix_fmt",
        pix_fmt,
        "-c:v",
        id,
        "-f",
        "null",
        "-",
    ]);
    match run_capture(probe).await {
        Ok((0, _, _)) => (true, String::new()),
        Ok((_, _, stderr)) => (false, stderr.trim().to_string()),
        Err(error) => (false, error.detail),
    }
}

pub async fn check_encoders(tools: &FfmpegTools) -> EncoderReport {
    let amf = check_amf(tools).await;
    let mut encoders_command = tokio::process::Command::new(&tools.ffmpeg);
    encoders_command.args(["-hide_banner", "-encoders"]);
    let listing = match run_capture(encoders_command).await {
        Ok((_, stdout, stderr)) => format!("{stdout}{stderr}"),
        Err(_) => String::new(),
    };
    let candidates = [
        EncoderCandidate {
            id: "h264_amf",
            label: "AMD AMF (H.264)",
            codec: "h264",
            vendor: "AMD",
            hardware: true,
        },
        EncoderCandidate {
            id: "h264_nvenc",
            label: "NVIDIA NVENC (H.264)",
            codec: "h264",
            vendor: "NVIDIA",
            hardware: true,
        },
        EncoderCandidate {
            id: "h264_qsv",
            label: "Intel Quick Sync (H.264)",
            codec: "h264",
            vendor: "Intel",
            hardware: true,
        },
        EncoderCandidate {
            id: "libx264",
            label: "Software (libx264)",
            codec: "h264",
            vendor: "CPU",
            hardware: false,
        },
    ];

    let mut statuses = Vec::new();
    for candidate in candidates {
        let listed = listing
            .lines()
            .any(|line| line.split_whitespace().any(|token| token == candidate.id));
        let (available, message, detail) = if !listed {
            (
                false,
                "This FFmpeg build does not include the encoder.".to_string(),
                String::new(),
            )
        } else if candidate.id == "h264_amf" {
            (amf.available, amf.message.clone(), amf.detail.clone())
        } else if !candidate.hardware {
            (
                true,
                "Software encoder available.".to_string(),
                String::new(),
            )
        } else {
            let (ok, detail) = test_hardware_encoder(tools, candidate.id).await;
            (
                ok,
                if ok {
                    "Hardware encoder tested successfully.".to_string()
                } else {
                    "Encoder is present but could not initialise on this system.".to_string()
                },
                detail,
            )
        };
        statuses.push(EncoderStatus {
            id: candidate.id.to_string(),
            label: candidate.label.to_string(),
            codec: candidate.codec.to_string(),
            vendor: candidate.vendor.to_string(),
            hardware: candidate.hardware,
            available,
            message,
            detail,
        });
    }

    EncoderReport {
        encoders: statuses,
        amf,
    }
}
pub async fn tools_status(config: &Config) -> ToolsStatus {
    match detect_tools(config).await {
        Ok(tools) => {
            let version = version_line(Path::new(&tools.ffmpeg))
                .await
                .unwrap_or_else(|| "FFmpeg".to_string());
            ToolsStatus {
                found: true,
                ffmpeg: tools.ffmpeg,
                ffprobe: tools.ffprobe,
                version,
                error: String::new(),
            }
        }
        Err(error) => ToolsStatus {
            found: false,
            ffmpeg: String::new(),
            ffprobe: String::new(),
            version: String::new(),
            error: error.message,
        },
    }
}

async fn run_capture(mut command: tokio::process::Command) -> AppResult<(i32, String, String)> {
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let output = command.output().await?;
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

pub async fn check_amf(tools: &FfmpegTools) -> AmfStatus {
    let mut encoders_command = tokio::process::Command::new(&tools.ffmpeg);
    encoders_command.args(["-hide_banner", "-encoders"]);
    let encoders = match run_capture(encoders_command).await {
        Ok((_, stdout, stderr)) => format!("{stdout}{stderr}"),
        Err(error) => {
            return AmfStatus {
                available: false,
                message: error.message,
                detail: error.detail,
                options: Vec::new(),
            };
        }
    };
    if !encoders.lines().any(|line| line.contains("h264_amf")) {
        return AmfStatus {
            available: false,
            message: "This FFmpeg build does not include the h264_amf encoder.".to_string(),
            detail: String::new(),
            options: Vec::new(),
        };
    }

    let mut help_command = tokio::process::Command::new(&tools.ffmpeg);
    help_command.args(["-hide_banner", "-h", "encoder=h264_amf"]);
    let options = match run_capture(help_command).await {
        Ok((_, stdout, stderr)) => {
            let text = format!("{stdout}{stderr}");
            let mut options = Vec::new();
            for line in text.lines() {
                let line = line.trim_start();
                if let Some(rest) = line.strip_prefix('-')
                    && let Some(name) = rest.split_whitespace().next()
                {
                    options.push(name.to_string());
                }
            }
            options
        }
        Err(_) => Vec::new(),
    };

    let mut probe = tokio::process::Command::new(&tools.ffmpeg);
    probe.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=256x144:d=0.3",
        "-c:v",
        "h264_amf",
    ]);
    if options.iter().any(|option| option == "quality") {
        probe.args(["-quality", "speed"]);
    }
    if options.iter().any(|option| option == "rc") {
        probe.args(["-rc", "cqp"]);
    }
    if options.iter().any(|option| option == "qp_i") {
        probe.args(["-qp_i", "20"]);
    }
    if options.iter().any(|option| option == "qp_p") {
        probe.args(["-qp_p", "22"]);
    }
    probe.args(["-pix_fmt", "yuv420p", "-f", "null", "-"]);

    match run_capture(probe).await {
        Ok((0, _, _)) => {
            let missing: Vec<String> = ["quality", "rc", "qp_i", "qp_p"]
                .iter()
                .filter(|name| !options.iter().any(|option| option == *name))
                .map(|name| name.to_string())
                .collect();
            let message = if missing.is_empty() {
                "AMD AMF (h264_amf) is available and tested.".to_string()
            } else {
                format!(
                    "AMD AMF is available. This FFmpeg build does not recognise: {}.",
                    missing.join(", ")
                )
            };
            AmfStatus {
                available: true,
                message,
                detail: String::new(),
                options,
            }
        }
        Ok((_, _, stderr)) => AmfStatus {
            available: false,
            message: "FFmpeg includes h264_amf, but the encoder could not initialise on this GPU."
                .to_string(),
            detail: stderr.trim().to_string(),
            options,
        },
        Err(error) => AmfStatus {
            available: false,
            message: format!("Could not test AMD AMF: {}", error.message),
            detail: error.detail,
            options,
        },
    }
}
