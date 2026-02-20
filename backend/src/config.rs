// backend/src/config.rs
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub static_dir: String,
    pub uploads_dir: String,
    pub public_site_url: String,   // pour le <base> et WebRTC
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),

            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:/app/data/nook.db".to_string()),

            static_dir: env::var("STATIC_FILES_DIR")
                .unwrap_or_else(|_| "/app/static".to_string()),

            uploads_dir: env::var("UPLOADS_DIR")
                .unwrap_or_else(|_| "/app/data/uploads".to_string()),

            public_site_url: env::var("PUBLIC_SITE_URL")
                .unwrap_or_else(|_| "http://localhost:6300".to_string()),
        }
    }
}
