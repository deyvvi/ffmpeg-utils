use std::collections::HashMap;
use std::sync::Mutex;

use tokio_util::sync::CancellationToken;

use crate::config::{Config, FfmpegTools};

pub struct AppState {
    pub config: Mutex<Config>,
    pub tools: Mutex<Option<FfmpegTools>>,
    pub jobs: Mutex<HashMap<String, CancellationToken>>,
}

impl AppState {
    pub fn new(config: Config, tools: Option<FfmpegTools>) -> Self {
        Self {
            config: Mutex::new(config),
            tools: Mutex::new(tools),
            jobs: Mutex::new(HashMap::new()),
        }
    }
}
