use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;

use crate::errors::{AppError, AppResult};
use crate::models::{LogEvent, ProgressEvent};

#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub status: i32,
    pub cancelled: bool,
    pub stderr_tail: String,
}

fn format_seconds(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

fn parse_progress_line(
    line: &str,
    duration: f64,
    base: f64,
    span: f64,
    speed: &mut String,
) -> Option<(i32, f64)> {
    let (key, value) = line.trim().split_once('=')?;
    match key {
        "out_time_us" | "out_time_ms" => {
            let micros: i64 = value.parse().ok()?;
            let seconds = micros as f64 / 1_000_000.0;
            let fraction = if duration > 0.0 {
                (seconds / duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let percent = ((base + fraction * span) * 100.0).round() as i32;
            Some((percent.clamp(0, 99), seconds))
        }
        "speed" => {
            *speed = value.to_string();
            None
        }
        _ => None,
    }
}

pub async fn run_ffmpeg(
    app: &AppHandle,
    job_id: &str,
    command: Vec<String>,
    duration: f64,
    cancel: CancellationToken,
    base: f64,
    span: f64,
) -> AppResult<RunOutcome> {
    if command.is_empty() {
        return Err(AppError::new(
            "empty_command",
            "The FFmpeg command is empty.",
        ));
    }
    if cancel.is_cancelled() {
        return Ok(RunOutcome {
            status: -2,
            cancelled: true,
            stderr_tail: String::new(),
        });
    }

    let mut process = tokio::process::Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        process.creation_flags(0x08000000);
    }
    let mut child = process
        .spawn()
        .map_err(|error| AppError::new("spawn", format!("Could not start FFmpeg: {error}")))?;
    let pid = child.id();
    let mut stdin = child.stdin.take();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::new("spawn", "Could not read FFmpeg stdout."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::new("spawn", "Could not read FFmpeg stderr."))?;

    let tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let tail_writer = tail.clone();
    let log_app = app.clone();
    let log_job = job_id.to_string();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            {
                let mut buffer = tail_writer.lock().unwrap();
                if buffer.len() >= 60 {
                    buffer.pop_front();
                }
                buffer.push_back(line.clone());
            }
            crate::logging::log_line(&line);
            let _ = log_app.emit(
                "job:log",
                LogEvent {
                    job_id: log_job.clone(),
                    line,
                },
            );
        }
    });

    let progress_app = app.clone();
    let progress_job = job_id.to_string();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        let mut speed = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some((percent, seconds)) =
                parse_progress_line(&line, duration, base, span, &mut speed)
            {
                let detail = if speed.is_empty() {
                    format!("{} / {}", format_seconds(seconds), format_seconds(duration))
                } else {
                    format!(
                        "{} / {} · {}",
                        format_seconds(seconds),
                        format_seconds(duration),
                        speed
                    )
                };
                let _ = progress_app.emit(
                    "job:progress",
                    ProgressEvent {
                        job_id: progress_job.clone(),
                        percent,
                        detail,
                        speed: speed.clone(),
                        out_time: seconds,
                        duration,
                    },
                );
            }
        }
    });

    let mut wait_task = tokio::spawn(async move { child.wait().await });
    let mut cancelled = false;
    let status = tokio::select! {
        result = &mut wait_task => {

            result
                .map_err(|error| AppError::new("wait", error.to_string()))?
                .map_err(|error| AppError::new("wait", error.to_string()))?
        }
        _ = cancel.cancelled() => {
            cancelled = true;
            if let Some(mut input) = stdin.take() {
                let _ = input.write_all(b"q\n").await;
                let _ = input.flush().await;
            }
            match tokio::time::timeout(Duration::from_secs(6), &mut wait_task).await {
                Ok(result) => result
                    .map_err(|error| AppError::new("wait", error.to_string()))?
                    .map_err(|error| AppError::new("wait", error.to_string()))?,
                Err(_) => {
                    if let Some(pid) = pid {
                        let mut kill = tokio::process::Command::new("taskkill");
                        kill.args(["/PID", &pid.to_string(), "/T", "/F"]);
                        #[cfg(windows)]
                        {
                                                kill.creation_flags(0x08000000);
                        }
                        let _ = kill.output().await;
                    }
                    wait_task
                        .await
                        .map_err(|error| AppError::new("wait", error.to_string()))?
                        .map_err(|error| AppError::new("wait", error.to_string()))?
                }
            }
        }
    };

    let _ = stderr_task.await;
    let _ = stdout_task.await;

    let stderr_tail = {
        let buffer = tail.lock().unwrap();
        buffer.iter().cloned().collect::<Vec<_>>().join("\n")
    };

    Ok(RunOutcome {
        status: status.code().unwrap_or(-1),
        cancelled,
        stderr_tail,
    })
}
