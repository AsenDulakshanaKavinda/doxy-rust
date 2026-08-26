use std::time::Instant;

use config::Config;

pub struct AppState {
    pub start_time: Instant,
    pub config: Config
}

impl AppState {
    pub fn new() -> Self {
        Self { 
            start_time: Instant::now(),
            config: config::Config::load().expect("Failed to load application configuration"),
        }
    }
}