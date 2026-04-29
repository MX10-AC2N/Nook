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
        dotenv::dotenv().ok();

        let public_site_url =
            env::var("PUBLIC_SITE_URL").unwrap_or_else(|_| "http://localhost:6300".to_string());

        // Origines CORS : toujours inclure PUBLIC_SITE_URL + les valeurs de ALLOWED_ORIGINS
        // En production (release), n'inclure PAS localhost (H2)
        let mut origins: Vec<String> = vec![
            public_site_url.clone(),
        ];

        // Uniquement en développement : ajouter localhost
        if cfg!(debug_assertions) {
            origins.push("http://localhost:5173".to_string());
            origins.push("http://localhost:6300".to_string());
            origins.push("http://127.0.0.1:6300".to_string());
        }

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


#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        for key in &[
            "PORT", "DATABASE_URL", "STATIC_FILES_DIR", "UPLOADS_DIR",
            "GIFS_DIR", "PUBLIC_SITE_URL", "ALLOWED_ORIGINS",
            "TURN_HOST", "TURN_PORT", "TURN_SECRET",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn test_default_values() {
        clear_env();
        let config = Config::load();

        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "sqlite:/app/data/nook.db");
        assert_eq!(config.static_dir, "/app/static");
        assert_eq!(config.uploads_dir, "/app/data/uploads");
        assert_eq!(config.gifs_dir, "/app/data/gifs");
        assert_eq!(config.turn_port, 3478);
    }

    #[test]
    fn test_custom_port() {
        clear_env();
        std::env::set_var("PORT", "8080");
        let config = Config::load();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_allowed_origins_includes_defaults() {
        clear_env();
        // En développement (debug), localhost est présent
        // En production (release), il ne l'est pas
        let config = Config::load();

        if cfg!(debug_assertions) {
            assert!(config.allowed_origins.contains(&"http://localhost:5173".to_string()));
            assert!(config.allowed_origins.contains(&"http://localhost:6300".to_string()));
            assert!(config.allowed_origins.contains(&"http://127.0.0.1:6300".to_string()));
        } else {
            // En production, localhost ne doit PAS être présent
            assert!(!config.allowed_origins.contains(&"http://localhost:5173".to_string()));
            assert!(!config.allowed_origins.contains(&"http://localhost:6300".to_string()));
        }
    }

    #[test]
    fn test_allowed_origins_with_extra() {
        clear_env();
        std::env::set_var("ALLOWED_ORIGINS", "https://nook.example.com,https://api.example.com");
        let config = Config::load();

        assert!(config.allowed_origins.contains(&"https://nook.example.com".to_string()));
        assert!(config.allowed_origins.contains(&"https://api.example.com".to_string()));
        // Defaults : localhost présent uniquement en développement
        if cfg!(debug_assertions) {
            assert!(config.allowed_origins.contains(&"http://localhost:5173".to_string()));
        } else {
            assert!(!config.allowed_origins.contains(&"http://localhost:5173".to_string()));
        }
    }

    #[test]
    fn test_allowed_origins_no_duplicates() {
        clear_env();
        std::env::set_var("ALLOWED_ORIGINS", "http://localhost:5173,https://example.com");
        let config = Config::load();

        let count_5173 = config.allowed_origins.iter()
            .filter(|o| *o == "http://localhost:5173")
            .count();
        assert_eq!(count_5173, 1, "Should not have duplicates");
    }

    #[test]
    fn test_turn_host_from_env() {
        clear_env();
        std::env::set_var("TURN_HOST", "turn.example.com");
        let config = Config::load();
        assert_eq!(config.turn_host, "turn.example.com");
    }

    #[test]
    fn test_turn_host_fallback() {
        clear_env();
        std::env::set_var("PUBLIC_SITE_URL", "https://nook.mydomain.com:6300");
        let config = Config::load();
        assert_eq!(config.turn_host, "nook.mydomain.com");
    }

    #[test]
    fn test_turn_port_custom() {
        clear_env();
        std::env::set_var("TURN_PORT", "5349");
        let config = Config::load();
        assert_eq!(config.turn_port, 5349);
    }

    #[test]
    fn test_turn_port_invalid_fallback() {
        clear_env();
        std::env::set_var("TURN_PORT", "not_a_number");
        let config = Config::load();
        assert_eq!(config.turn_port, 3478, "Should fallback to 3478 on invalid");
    }
}
