use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub detail: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: String::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::new("io", value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::new("json", value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

fn tail(text: &str, lines: usize) -> String {
    let mut values: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if values.len() > lines {
        values = values.split_off(values.len() - lines);
    }
    values.join("\n")
}

pub fn ffmpeg_error(stderr: &str) -> AppError {
    let lowered = stderr.to_lowercase();
    let candidates: [(&str, &str, &str); 11] = [
        (
            "amf",
            "amf",
            "AMD AMF could not initialise the encoder. Check the AMD drivers and that this FFmpeg build supports AMF.",
        ),
        (
            "unknown encoder",
            "encoder_missing",
            "The requested encoder is not available in this FFmpeg build.",
        ),
        (
            "no space left",
            "disk_full",
            "There is not enough free space on the output drive.",
        ),
        (
            "permission denied",
            "permission",
            "Windows denied access to the file.",
        ),
        (
            "invalid data found",
            "corrupt",
            "The file appears to be damaged or incomplete.",
        ),
        ("file exists", "exists", "The output file already exists."),
        (
            "subtitle",
            "subtitle",
            "A subtitle track cannot be copied into the output container.",
        ),
        (
            "not divisible by 2",
            "dimensions",
            "The calculated dimensions are not valid for the pixel format.",
        ),
        (
            "could not write header",
            "output",
            "The output file could not be created.",
        ),
        (
            "decoder",
            "decoder",
            "FFmpeg does not have the decoder required for this file.",
        ),
        (
            "conversion failed",
            "conversion",
            "FFmpeg could not complete the conversion.",
        ),
    ];
    for (needle, code, message) in candidates {
        if lowered.contains(needle) {
            return AppError::new(code, message).with_detail(tail(stderr, 5));
        }
    }
    AppError::new("ffmpeg", "FFmpeg failed.").with_detail(tail(stderr, 5))
}
