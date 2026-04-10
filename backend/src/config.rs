// backend/src/config.rs
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub static_dir: String,
    pub uploads_dir: String,
    pub gifs_dir: String,
    #[allow(dead_code)]
    pub public_site_url: String,
    /// Liste des origines CORS autorisées, séparées par des virgules.
    /// Exemple : "http://192.168.1.10:6300,https://nook.mondomaine.com"
    /// Si vide, seules localhost:5173 et localhost:6300 sont autorisées.
    pub allowed_origins: Vec<String>,
    /// Configuration TURN pour WebRTC
    pub turn_host: String,
    pub turn_port: u16,
    pub turn_secret: String,
}

impl Config {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let public_site_url =
            env::var("PUBLIC_SITE_URL").unwrap_or_else(|_| "http://localhost:6300".to_string());

        // Origines CORS : toujours inclure PUBLIC_SITE_URL + les valeurs de ALLOWED_ORIGINS
        let mut origins: Vec<String> = vec![
            "http://localhost:5173".to_string(),
            "http://localhost:6300".to_string(),
            "http://127.0.0.1:6300".to_string(),
            public_site_url.clone(),
        ];

        if let Ok(extra) = env::var("ALLOWED_ORIGINS") {
            for origin in extra.split(',') {
                let o = origin.trim().to_string();
                if !o.is_empty() && !origins.contains(&o) {
                    origins.push(o);
                }
            }
        }

        // Dédoublonnage
        origins.dedup();

        let turn_host = env::var("TURN_HOST")
            .unwrap_or_else(|_| {
                // Fallback: utiliser l'hôte PUBLIC_SITE_URL si TURN_HOST non défini
                public_site_url
                    .replace("http://", "")
                    .replace("https://", "")
                    .split(':')
                    .next()
                    .unwrap_or("localhost")
                    .to_string()
            });

        let turn_port: u16 = env::var("TURN_PORT")
            .unwrap_or_else(|_| "3478".to_string())
            .parse()
            .unwrap_or(3478);

        let turn_secret = env::var("TURN_SECRET")
            .unwrap_or_else(|_| String::new());

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),

            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:/app/data/nook.db".to_string()),

            static_dir: env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "/app/static".to_string()),

            uploads_dir: env::var("UPLOADS_DIR")
                .unwrap_or_else(|_| "/app/data/uploads".to_string()),

            // GIFs dans le volume de données — mis à jour par update-gifs.sh sans rebuild
            gifs_dir: env::var("GIFS_DIR")
                .unwrap_or_else(|_| "/app/data/gifs".to_string()),

            public_site_url,
            allowed_origins: origins,
            turn_host,
            turn_port,
            turn_secret,
        }
    }
}
