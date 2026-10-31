use std::fs::OpenOptions;
use std::io::Write;

pub fn log_line(line: &str) {
    if line.trim().is_empty() {
        return;
    }
    let directory = crate::config::logs_dir();
    if std::fs::create_dir_all(&directory).is_err() {
        return;
    }
    let path = directory.join("ffmpeg_utils.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}
