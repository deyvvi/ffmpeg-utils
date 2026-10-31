use std::path::{Path, PathBuf};

const OLD_SINGLE_SUFFIXES: [&str; 2] = ["_solo_ingles", "_solo_latino"];

fn strip_known_suffixes(stem: &str) -> String {
    let lowered = stem.to_lowercase();
    for suffix in [
        OLD_SINGLE_SUFFIXES[0],
        OLD_SINGLE_SUFFIXES[1],
        "_single_english",
        "_single_spanish",
    ] {
        if lowered.ends_with(suffix) {
            return stem[..stem.len() - suffix.len()].to_string();
        }
    }
    // Remove a previous _trim_HH-MM-SS-HH-MM-SS suffix.
    if let Some(index) = lowered.rfind("_trim_") {
        let tail = &lowered[index + 6..];
        let parts: Vec<&str> = tail.split('-').collect();
        if parts.len() == 6
            && parts
                .iter()
                .all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_digit()))
        {
            return stem[..index].to_string();
        }
    }
    stem.to_string()
}

fn append_suffix(stem: &str, suffix: &str) -> String {
    if stem.to_lowercase().ends_with(&suffix.to_lowercase()) {
        stem.to_string()
    } else {
        format!("{stem}{suffix}")
    }
}

fn resolution_suffix(width: u32, height: u32) -> String {
    match (width, height) {
        (640, 360) => "_360p".to_string(),
        (854, 480) => "_480p".to_string(),
        (1280, 720) => "_720p".to_string(),
        (1920, 1080) => "_1080p".to_string(),
        (2560, 1440) => "_1440p".to_string(),
        (3840, 2160) => "_2160p".to_string(),
        _ => format!("_{width}x{height}"),
    }
}

pub fn normalize_output(input: &Path, width: u32, height: u32) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let suffix = resolution_suffix(width, height);
    input.with_file_name(format!("{}.mkv", append_suffix(&stem, &suffix)))
}

pub fn single_audio_output(input: &Path, language: &str) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let base = strip_known_suffixes(&stem);
    let suffix = match language.to_lowercase().as_str() {
        "eng" | "en" | "english" => "_single_english",
        "spa" | "es" | "spanish" | "latino" => "_single_spanish",
        other => {
            return input.with_file_name(format!("{base}_single_{other}.mkv"));
        }
    };
    input.with_file_name(format!("{}.mkv", append_suffix(&base, suffix)))
}

pub fn trim_output(input: &Path, start: f64, end: f64) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let base = strip_known_suffixes(&stem);
    let suffix = format!("_trim_{}-{}", name_time(start), name_time(end));
    input.with_file_name(format!("{}.mkv", append_suffix(&base, &suffix)))
}

pub fn edit_tracks_output(input: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let base = strip_known_suffixes(&stem);
    input.with_file_name(format!("{}.mkv", append_suffix(&base, "_tracks")))
}

fn name_time(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    format!("{hours:02}-{minutes:02}-{secs:02}")
}

pub fn alternate_output(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path
        .extension()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    for number in 1..=999 {
        let candidate = if extension.is_empty() {
            path.with_file_name(format!("{stem}_{number}"))
        } else {
            path.with_file_name(format!("{stem}_{number}.{extension}"))
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    path.with_file_name(format!("{stem}_9999.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_names() {
        assert_eq!(
            normalize_output(Path::new(r"C:\Videos\Coyote.mkv"), 1920, 1080)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_1080p.mkv"
        );
        assert_eq!(
            normalize_output(Path::new(r"C:\Videos\Coyote.mkv"), 1280, 720)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_720p.mkv"
        );
        assert_eq!(
            normalize_output(Path::new(r"C:\Videos\Coyote.mkv"), 1600, 900)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_1600x900.mkv"
        );
        assert_eq!(
            single_audio_output(Path::new(r"C:\Videos\Coyote_1080p.mkv"), "eng")
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_1080p_single_english.mkv"
        );
        assert_eq!(
            trim_output(Path::new(r"C:\Videos\Coyote_1080p.mkv"), 83.0, 405.0)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_1080p_trim_00-01-23-00-06-45.mkv"
        );
        assert_eq!(
            edit_tracks_output(Path::new(r"C:\Videos\Coyote.mkv"))
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_tracks.mkv"
        );
    }

    #[test]
    fn switching_language_does_not_stack_suffixes() {
        let path = Path::new("Coyote_1080p_single_english.mkv");
        assert_eq!(
            single_audio_output(path, "spa")
                .file_name()
                .unwrap()
                .to_string_lossy(),
            "Coyote_1080p_single_spanish.mkv"
        );
    }
}
