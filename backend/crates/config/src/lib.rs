use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app_port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        // load .env file for local development if present
        let _ = dotenvy::dotenv();

        // Deserialize environment variables into the typed Config struct
        envy::from_env::<Config>()
    }
}