use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub db_path: String,
    pub static_dir: String,
    pub uploads_dir: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap(),
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "sqlite:/app/data/nook.db".to_string()),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "/app/static".to_string()),
            uploads_dir: env::var("UPLOADS_DIR").unwrap_or_else(|_| "/app/data/uploads".to_string()),
        }
    }
}
