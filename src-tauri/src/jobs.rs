use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::command::{build_normalize_command, build_single_audio_command, display_command};
use crate::config::{Config, FfmpegTools};
use crate::errors::{AppError, AppResult, ffmpeg_error};
use crate::extract::{build_extract_command, extraction_output, subtitle_extension};
use crate::ffmpeg::check_amf;
use crate::ffprobe::probe_file;
use crate::models::{
    AmfStatus, Check, CommandPlan, JobFinished, JobKind, JobRequest, MediaInfo, OutputRequest,
    OutputSuggestion, ProgressEvent, StreamInfo,
};
use crate::naming::{
    alternate_output, edit_tracks_output, normalize_output, single_audio_output, trim_output,
};
use crate::process::run_ffmpeg;
use crate::resolution::{canvas_for_stream, validate_target};
use crate::state::AppState;
use crate::track_edit::{AddedTrackSpec, build_edit_tracks_command};
use crate::tracks::{find_audio, subtitle_plan};
use crate::trim::{build_trim_command, validate_range};
use crate::verify::{verify_extract, verify_output, verify_tracks, verify_trim};

struct PreparedCommand {
    command: Vec<String>,
    final_output: PathBuf,
    run_output: PathBuf,
    expected_audio: Option<usize>,
    expected_resolution: Option<(u32, u32)>,
    expected_subtitles: usize,
    expected_language: Option<String>,
    trim_duration: Option<f64>,
    trim_mode: Option<String>,
    stream: Option<StreamInfo>,
}

struct PreparedJob {
    media: MediaInfo,
    kind: JobKind,
    commands: Vec<PreparedCommand>,
    notes: Vec<String>,
}

fn default_amf() -> AmfStatus {
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

fn output_directory(config: &Config, input: &str) -> PathBuf {
    if config.output_dir.trim().is_empty() {
        Path::new(input)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(&config.output_dir)
    }
}

fn temp_path(target: &Path) -> PathBuf {
    let stem = target.file_stem().unwrap_or_default().to_string_lossy();
    let extension = target
        .extension()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "mkv".to_string());
    target.with_file_name(format!("{stem}.__ffmpeg_utils_tmp.{extension}"))
}

fn replace_file(source: &Path, target: &Path) -> AppResult<()> {
    if target.exists() {
        fs::remove_file(target).map_err(|error| {
            AppError::new(
                "replace",
                format!("Could not replace the existing output: {error}"),
            )
        })?;
    }
    fs::rename(source, target).map_err(|error| {
        AppError::new(
            "move",
            format!("Could not move the temporary output: {error}"),
        )
    })
}

fn ensure_output_safe(input: &str, output: &str) -> AppResult<()> {
    if let (Ok(input), Ok(output)) = (fs::canonicalize(input), fs::canonicalize(output))
        && input == output
    {
        return Err(AppError::new(
            "same_file",
            "The output cannot be the same file as the input.",
        ));
    }
    if input == output {
        return Err(AppError::new(
            "same_file",
            "The output cannot be the same file as the input.",
        ));
    }
    Ok(())
}

async fn prepare(
    request: &JobRequest,
    config: &Config,
    tools: &FfmpegTools,
    amf: &AmfStatus,
) -> AppResult<PreparedJob> {
    match request {
        JobRequest::Normalize {
            path,
            output,
            video_mode,
            width,
            height,
            replace,
        } => {
            ensure_output_safe(path, output)?;
            validate_target(*width, *height)
                .map_err(|message| AppError::new("resolution", message))?;
            let media = probe_file(&tools.ffprobe, path, true).await?;
            let canvas = canvas_for_stream(media.video().unwrap(), *width, *height);
            let subtitles = subtitle_plan(&media);
            let final_output = PathBuf::from(output);
            let run_output = if *replace {
                temp_path(&final_output)
            } else {
                final_output.clone()
            };
            let command = build_normalize_command(
                tools,
                &media,
                &run_output.to_string_lossy(),
                video_mode,
                &canvas,
                &subtitles,
                amf,
                &config.quality,
                config.qp_i,
                config.qp_p,
                config.software_crf,
                !replace,
            )
            .map_err(|message| AppError::new("command", message))?;
            let mut notes = vec![format!("Video mode: {video_mode}")];
            if let Some(filter) = &canvas.filter_chain {
                notes.push(format!("Filter: {filter}"));
            }
            notes.push(format!(
                "Scale: {}×{}; padding: {}",
                canvas.scaled_width,
                canvas.scaled_height,
                canvas.pad_text()
            ));
            notes.extend(subtitles.warnings.iter().cloned());
            notes.extend(
                subtitles
                    .excluded
                    .iter()
                    .map(|item| format!("Excluded subtitle: {item}")),
            );
            Ok(PreparedJob {
                kind: JobKind::Normalize,
                media,
                commands: vec![PreparedCommand {
                    command,
                    final_output,
                    run_output,
                    expected_audio: None,
                    expected_resolution: Some((*width, *height)),
                    expected_subtitles: subtitles.indices.len(),
                    expected_language: None,
                    trim_duration: None,
                    trim_mode: None,
                    stream: None,
                }],
                notes,
            })
        }
        JobRequest::Trim {
            path,
            output,
            start,
            end,
            mode,
            replace,
        } => {
            ensure_output_safe(path, output)?;
            validate_range(*start, *end, 0.0)
                .map_err(|message| AppError::new("trim_range", message))?;
            let media = probe_file(&tools.ffprobe, path, true).await?;
            validate_range(*start, *end, media.duration)
                .map_err(|message| AppError::new("trim_range", message))?;
            let final_output = PathBuf::from(output);
            let run_output = if *replace {
                temp_path(&final_output)
            } else {
                final_output.clone()
            };
            let command = build_trim_command(
                tools,
                &media,
                &run_output.to_string_lossy(),
                *start,
                *end,
                mode,
                amf,
                &config.quality,
                config.qp_i,
                config.qp_p,
                config.software_crf,
                !replace,
            )
            .map_err(|message| AppError::new("command", message))?;
            Ok(PreparedJob {
                kind: JobKind::Trim,
                media,
                commands: vec![PreparedCommand {
                    command,
                    final_output,
                    run_output,
                    expected_audio: None,
                    expected_resolution: None,
                    expected_subtitles: 0,
                    expected_language: None,
                    trim_duration: Some(end - start),
                    trim_mode: Some(mode.clone()),
                    stream: None,
                }],
                notes: vec![format!("Trim mode: {mode}")],
            })
        }
        JobRequest::SingleAudio {
            path,
            output,
            language,
            audio_index,
            replace,
        } => {
            ensure_output_safe(path, output)?;
            let media = probe_file(&tools.ffprobe, path, true).await?;
            let audio = if let Some(index) = audio_index {
                media
                    .audios()
                    .into_iter()
                    .find(|stream| stream.index == *index)
                    .cloned()
            } else {
                let candidates = find_audio(&media, language);
                candidates.first().map(|stream| (*stream).clone())
            }
            .ok_or_else(|| {
                AppError::new(
                    "no_audio",
                    format!("No audio track was found for language {language}."),
                )
            })?;
            let subtitles = subtitle_plan(&media);
            let final_output = PathBuf::from(output);
            let run_output = if *replace {
                temp_path(&final_output)
            } else {
                final_output.clone()
            };
            let command = build_single_audio_command(
                tools,
                &media,
                &run_output.to_string_lossy(),
                audio.index,
                &subtitles,
                !replace,
            )
            .map_err(|message| AppError::new("command", message))?;
            Ok(PreparedJob {
                kind: JobKind::SingleAudio,
                media,
                commands: vec![PreparedCommand {
                    command,
                    final_output,
                    run_output,
                    expected_audio: None,
                    expected_resolution: None,
                    expected_subtitles: subtitles.indices.len(),
                    expected_language: Some(audio.language.clone()),
                    trim_duration: None,
                    trim_mode: None,
                    stream: None,
                }],
                notes: {
                    let mut notes = vec![format!("Audio track: {}", audio.index)];
                    notes.extend(subtitles.warnings.iter().cloned());
                    notes.extend(
                        subtitles
                            .excluded
                            .iter()
                            .map(|item| format!("Excluded subtitle: {item}")),
                    );
                    notes
                },
            })
        }
        JobRequest::EditTracks {
            path,
            output,
            audio,
            subtitles,
            added,
            replace,
        } => {
            ensure_output_safe(path, output)?;
            let media = probe_file(&tools.ffprobe, path, true).await?;
            let mut added_specs = Vec::with_capacity(added.len());
            let mut external_files: HashMap<String, MediaInfo> = HashMap::new();
            for track in added {
                if !matches!(track.kind.as_str(), "audio" | "subtitle") {
                    return Err(AppError::new(
                        "track_kind",
                        format!("Unsupported track kind: {}", track.kind),
                    ));
                }
                ensure_output_safe(&track.path, output)?;
                if !external_files.contains_key(&track.path) {
                    let external = probe_file(&tools.ffprobe, &track.path, false).await?;
                    external_files.insert(track.path.clone(), external);
                }
                let external = external_files
                    .get(&track.path)
                    .expect("external file was probed");
                let stream = external
                    .streams
                    .iter()
                    .find(|stream| stream.index == track.stream_index)
                    .ok_or_else(|| {
                        AppError::new(
                            "missing_stream",
                            format!(
                                "Track {} does not exist in {}.",
                                track.stream_index, track.path
                            ),
                        )
                    })?;
                if stream.kind != track.kind {
                    return Err(AppError::new(
                        "track_kind",
                        format!(
                            "Track {} is a {}, not a {}.",
                            track.stream_index, stream.kind, track.kind
                        ),
                    ));
                }
                added_specs.push(AddedTrackSpec {
                    path: track.path.clone(),
                    stream_index: track.stream_index,
                    kind: track.kind.clone(),
                    codec: stream.codec.clone(),
                    language: if track.language.trim().is_empty() {
                        stream.language.clone()
                    } else {
                        track.language.clone()
                    },
                    title: track.title.clone(),
                    default: track.default,
                    forced: track.kind == "subtitle" && track.forced,
                });
            }

            let final_output = PathBuf::from(output);
            let run_output = if *replace {
                temp_path(&final_output)
            } else {
                final_output.clone()
            };
            let command = build_edit_tracks_command(
                tools,
                &media,
                &run_output.to_string_lossy(),
                audio,
                subtitles,
                &added_specs,
                !replace,
            )
            .map_err(|message| AppError::new("command", message))?;
            let added_audio = added_specs
                .iter()
                .filter(|track| track.kind == "audio")
                .count();
            let added_subtitles = added_specs
                .iter()
                .filter(|track| track.kind == "subtitle")
                .count();
            let expected_audio = audio.len() + added_audio;
            let expected_subtitles = subtitles.len() + added_subtitles;
            let mut notes = vec![
                "Video: copy without re-encoding".to_string(),
                format!("Audio: {} kept, {} added", audio.len(), added_audio),
                format!(
                    "Subtitles: {} kept, {} added",
                    subtitles.len(),
                    added_subtitles
                ),
            ];
            for track in &added_specs {
                notes.push(format!(
                    "Added {}: {} (stream {})",
                    track.kind, track.path, track.stream_index
                ));
            }
            Ok(PreparedJob {
                kind: JobKind::EditTracks,
                media,
                commands: vec![PreparedCommand {
                    command,
                    final_output,
                    run_output,
                    expected_audio: Some(expected_audio),
                    expected_resolution: None,
                    expected_subtitles,
                    expected_language: None,
                    trim_duration: None,
                    trim_mode: None,
                    stream: None,
                }],
                notes,
            })
        }
        JobRequest::Extract { path, items } => {
            if items.is_empty() {
                return Err(AppError::new(
                    "no_tracks",
                    "Select at least one track to extract.",
                ));
            }
            let media = probe_file(&tools.ffprobe, path, true).await?;
            let mut commands = Vec::new();
            let mut notes = vec![format!("{} track(s)", items.len())];
            for item in items {
                ensure_output_safe(path, &item.output)?;
                let stream = media
                    .streams
                    .iter()
                    .find(|stream| stream.index == item.stream_index)
                    .cloned()
                    .ok_or_else(|| {
                        AppError::new(
                            "missing_stream",
                            format!("Track {} does not exist.", item.stream_index),
                        )
                    })?;
                let final_output = PathBuf::from(&item.output);
                let run_output = if item.replace {
                    temp_path(&final_output)
                } else {
                    final_output.clone()
                };
                let command = build_extract_command(
                    tools,
                    &media,
                    &stream,
                    &run_output,
                    &item.format,
                    !item.replace,
                )
                .map_err(|message| AppError::new("command", message))?;
                notes.push(format!("{}: {}", stream.index, final_output.display()));
                if !item.format.eq_ignore_ascii_case("copy") {
                    notes.push(format!(
                        "Track {} converted to {}.",
                        stream.index, item.format
                    ));
                }
                if stream.kind == "subtitle" && item.format.eq_ignore_ascii_case("copy") {
                    let (extension, convert) = subtitle_extension(&stream);
                    if let Some(codec) = convert {
                        notes.push(format!(
                            "Subtitle {} will be converted to {codec}.",
                            stream.index
                        ));
                    } else if extension == "mks" {
                        notes.push(format!(
                            "Subtitle {} will be stored in a .mks container.",
                            stream.index
                        ));
                    }
                }
                commands.push(PreparedCommand {
                    command,
                    final_output,
                    run_output,
                    expected_audio: None,
                    expected_resolution: None,
                    expected_subtitles: 0,
                    expected_language: None,
                    trim_duration: None,
                    trim_mode: None,
                    stream: Some(stream),
                });
            }
            Ok(PreparedJob {
                kind: JobKind::Extract,
                media,
                commands,
                notes,
            })
        }
    }
}

fn request_requires_amf(request: &JobRequest) -> Option<String> {
    match request {
        JobRequest::Normalize { video_mode, .. } if video_mode == "h264_amf" => {
            Some("h264_amf".to_string())
        }
        JobRequest::Trim { mode, .. } if mode == "precise" => Some("precise".to_string()),
        _ => None,
    }
}

pub async fn preview_plan(
    request: &JobRequest,
    config: &Config,
    tools: &FfmpegTools,
) -> AppResult<CommandPlan> {
    let prepared = prepare(request, config, tools, &default_amf()).await?;
    let commands = prepared
        .commands
        .iter()
        .map(|command| command.command.clone())
        .collect::<Vec<_>>();
    let display = commands
        .iter()
        .map(|command| display_command(command))
        .collect();
    let outputs = prepared
        .commands
        .iter()
        .map(|command| command.final_output.to_string_lossy().to_string())
        .collect();
    Ok(CommandPlan {
        commands,
        display,
        notes: prepared.notes,
        outputs,
    })
}

pub async fn start_job(
    request: JobRequest,
    config: Config,
    tools: FfmpegTools,
    app: AppHandle,
    state: &AppState,
) -> AppResult<String> {
    {
        let jobs = state.jobs.lock().unwrap();
        if !jobs.is_empty() {
            return Err(AppError::new(
                "busy",
                "A job is already running. Wait or cancel it first.",
            ));
        }
    }
    let amf = if request_requires_amf(&request).is_some() {
        let status = check_amf(&tools).await;
        if !status.available {
            return Err(AppError::new("amf_unavailable", status.message).with_detail(status.detail));
        }
        status
    } else {
        default_amf()
    };
    let prepared = prepare(&request, &config, &tools, &amf).await?;
    let job_id = Uuid::new_v4().to_string();
    let cancel = CancellationToken::new();
    {
        let mut jobs = state.jobs.lock().unwrap();
        if !jobs.is_empty() {
            return Err(AppError::new(
                "busy",
                "A job is already running. Wait or cancel it first.",
            ));
        }
        jobs.insert(job_id.clone(), cancel.clone());
    }

    let task_job_id = job_id.clone();
    let task_app = app.clone();
    tokio::spawn(async move {
        run_job(task_app, task_job_id, prepared, tools, cancel).await;
    });
    Ok(job_id)
}

pub fn cancel_job(job_id: &str, state: &AppState) {
    let jobs = state.jobs.lock().unwrap();
    if let Some(token) = jobs.get(job_id) {
        token.cancel();
    }
}

fn emit_log(app: &AppHandle, job_id: &str, line: impl Into<String>) {
    let line = line.into();
    crate::logging::log_line(&line);
    let _ = app.emit(
        "job:log",
        crate::models::LogEvent {
            job_id: job_id.to_string(),
            line,
        },
    );
}

fn emit_progress(app: &AppHandle, job_id: &str, percent: i32, detail: impl Into<String>) {
    let _ = app.emit(
        "job:progress",
        ProgressEvent {
            job_id: job_id.to_string(),
            percent: percent.clamp(0, 100),
            detail: detail.into(),
            speed: String::new(),
            out_time: 0.0,
            duration: 0.0,
        },
    );
}

fn cleanup(item: &PreparedCommand) {
    if item.run_output != item.final_output {
        let _ = fs::remove_file(&item.run_output);
    }
}

async fn run_job(
    app: AppHandle,
    job_id: String,
    prepared: PreparedJob,
    tools: FfmpegTools,
    cancel: CancellationToken,
) {
    let started = Instant::now();
    let total = prepared.commands.len();
    let mut outputs = Vec::new();
    let mut checks: Vec<Check> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut first_error: Option<AppError> = None;
    let mut cancelled = false;

    for (index, item) in prepared.commands.iter().enumerate() {
        if cancel.is_cancelled() {
            cancelled = true;
            cleanup(item);
            break;
        }
        emit_log(
            &app,
            &job_id,
            format!("Command: {}", display_command(&item.command)),
        );
        emit_progress(
            &app,
            &job_id,
            ((index as f64 / total as f64) * 100.0) as i32,
            format!("{} / {}", index + 1, total),
        );
        let base = index as f64 / total as f64;
        let span = 1.0 / total as f64;
        let outcome = match run_ffmpeg(
            &app,
            &job_id,
            item.command.clone(),
            prepared.media.duration,
            cancel.clone(),
            base,
            span,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                cleanup(item);
                errors.push(error.message.clone());
                first_error = Some(error);
                break;
            }
        };
        if outcome.cancelled {
            cancelled = true;
            cleanup(item);
            break;
        }
        if outcome.status != 0 {
            cleanup(item);
            let error = ffmpeg_error(&outcome.stderr_tail);
            errors.push(error.message.clone());
            first_error = Some(error);
            if prepared.kind != JobKind::Extract {
                break;
            }
            continue;
        }
        if item.run_output != item.final_output
            && let Err(error) = replace_file(&item.run_output, &item.final_output)
        {
            errors.push(error.message.clone());
            first_error = Some(error);
            if prepared.kind != JobKind::Extract {
                break;
            }
            continue;
        }
        let report = match prepared.kind {
            JobKind::Normalize | JobKind::SingleAudio => {
                verify_output(
                    &tools.ffprobe,
                    &item.final_output.to_string_lossy(),
                    prepared.kind,
                    &prepared.media,
                    item.expected_resolution,
                    item.expected_subtitles,
                    item.expected_language.as_deref(),
                )
                .await
            }
            JobKind::EditTracks => {
                verify_tracks(
                    &tools.ffprobe,
                    &item.final_output.to_string_lossy(),
                    &prepared.media,
                    item.expected_audio.unwrap_or(0),
                    item.expected_subtitles,
                )
                .await
            }
            JobKind::Trim => {
                verify_trim(
                    &tools.ffprobe,
                    &item.final_output.to_string_lossy(),
                    &prepared.media,
                    item.trim_duration.unwrap_or(0.0),
                    item.trim_mode.as_deref().unwrap_or("fast"),
                )
                .await
            }
            JobKind::Extract => {
                let stream = item
                    .stream
                    .as_ref()
                    .expect("extract command must have a stream");
                verify_extract(&tools.ffprobe, &item.final_output.to_string_lossy(), stream).await
            }
        };
        let report_ok = report.ok();
        checks.extend(report.checks);
        outputs.push(item.final_output.to_string_lossy().to_string());
        if !report_ok {
            errors.push(format!(
                "Verification failed for {}",
                item.final_output.display()
            ));
        }
    }

    let _ = app.emit(
        "job:progress",
        ProgressEvent {
            job_id: job_id.clone(),
            percent: 100,
            detail: "Completed".to_string(),
            speed: String::new(),
            out_time: prepared.media.duration,
            duration: prepared.media.duration,
        },
    );

    if let Some(state) = app.try_state::<AppState>() {
        state.jobs.lock().unwrap().remove(&job_id);
    }

    let ok = !cancelled && errors.is_empty() && checks.iter().all(|check| check.ok);
    let (code, message, detail) = if cancelled {
        (
            "cancelled".to_string(),
            "Cancelled by the user.".to_string(),
            String::new(),
        )
    } else if let Some(error) = first_error {
        (error.code.clone(), error.message, error.detail)
    } else if !errors.is_empty() {
        (
            "verification_failed".to_string(),
            errors
                .first()
                .cloned()
                .unwrap_or_else(|| "Verification failed.".to_string()),
            errors.join("\n"),
        )
    } else {
        (
            "completed".to_string(),
            "Completed and verified.".to_string(),
            String::new(),
        )
    };

    let _ = app.emit(
        "job:finished",
        JobFinished {
            job_id,
            ok,
            kind: prepared.kind,
            message,
            code: code.to_string(),
            detail,
            outputs,
            checks,
            elapsed: started.elapsed().as_secs_f64(),
        },
    );
}

pub async fn suggest_outputs(
    request: &OutputRequest,
    config: &Config,
    tools: &FfmpegTools,
) -> AppResult<Vec<OutputSuggestion>> {
    let mut suggestions = Vec::new();
    match request {
        OutputRequest::Normalize {
            path,
            width,
            height,
        } => {
            let directory = output_directory(config, path);
            let target = directory.join(
                normalize_output(Path::new(path), *width, *height)
                    .file_name()
                    .unwrap_or_default(),
            );
            suggestions.push(suggestion(target, None));
        }
        OutputRequest::Trim { path, start, end } => {
            if end <= start {
                return Err(AppError::new(
                    "trim_range",
                    "The end time must be after the start time.",
                ));
            }
            let directory = output_directory(config, path);
            let target = directory.join(
                trim_output(Path::new(path), *start, *end)
                    .file_name()
                    .unwrap_or_default(),
            );
            suggestions.push(suggestion(target, None));
        }
        OutputRequest::SingleAudio { path, language } => {
            let directory = output_directory(config, path);
            let target = directory.join(
                single_audio_output(Path::new(path), language)
                    .file_name()
                    .unwrap_or_default(),
            );
            suggestions.push(suggestion(target, None));
        }
        OutputRequest::EditTracks { path } => {
            let directory = output_directory(config, path);
            let target = directory.join(
                edit_tracks_output(Path::new(path))
                    .file_name()
                    .unwrap_or_default(),
            );
            suggestions.push(suggestion(target, None));
        }
        OutputRequest::Extract {
            path,
            output_dir,
            items,
        } => {
            let media = probe_file(&tools.ffprobe, path, true).await?;
            let directory = if output_dir.trim().is_empty() {
                output_directory(config, path)
            } else {
                PathBuf::from(output_dir)
            };
            for item in items {
                let stream = media
                    .streams
                    .iter()
                    .find(|stream| stream.index == item.stream_index)
                    .ok_or_else(|| {
                        AppError::new(
                            "missing_stream",
                            format!("Track {} does not exist.", item.stream_index),
                        )
                    })?;
                let target =
                    extraction_output(Path::new(path), stream, Some(&directory), &item.format)
                        .map_err(|message| AppError::new("command", message))?;
                suggestions.push(suggestion(target, Some(item.stream_index)));
            }
        }
    }
    Ok(suggestions)
}

fn suggestion(target: PathBuf, stream_index: Option<i64>) -> OutputSuggestion {
    let exists = target.exists();
    let alternate = alternate_output(&target);
    OutputSuggestion {
        path: target.to_string_lossy().to_string(),
        exists,
        alternate: alternate.to_string_lossy().to_string(),
        stream_index,
    }
}
