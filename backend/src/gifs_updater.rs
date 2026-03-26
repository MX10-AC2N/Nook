// backend/src/gifs_updater.rs
// Mise à jour hebdomadaire automatique des GIFs Nook
//
// Tâche tokio lancée au démarrage — se réveille tous les 7 jours.
// Appelle l'API Giphy (clé GIPHY_API_KEY dans .env).
// Sans clé configurée : skip silencieux, les GIFs existants sont conservés.
//
// GIFs stockés dans GIFS_DIR (/app/data/gifs/) — volume persistant.
// Nook les sert via ServeDir sans rebuild de l'image.

use serde_json::Value;
use std::path::Path;
use tokio::fs;
use tokio::time::{sleep, Duration};

/// 12 thèmes les plus populaires sur Giphy avec leur libellé affiché
const THEMES: &[(&str, &str, &str)] = &[
    ("reactions",    "💥 Réactions",      "thumbs up reaction"),
    ("lol",          "😂 Humour",       "laughing funny"),
    ("love",         "❤️ Amour",        "love heart"),
    ("celebration",  "🎉 Fête",         "celebration party"),
    ("birthday",     "🎂 Anniversaire", "happy birthday"),
    ("animals",      "🐾 Animaux",      "cute animals"),
    ("hello",        "👋 Bonjour",      "hello wave"),
    ("bye",          "👋 Au revoir",    "goodbye wave"),
    ("yes",          "✅ Oui",          "yes nodding"),
    ("no",           "❌ Non",          "no shaking head"),
    ("wow",          "😮 Wow",          "wow surprised"),
    ("facepalm",     "🤦 Facepalm",     "facepalm"),
];

const GIFS_PER_THEME: usize  = 12;
const GIPHY_BASE:      &str  = "https://api.giphy.com/v1/gifs/search";
const INTERVAL_SECS:   u64   = 7 * 24 * 3600; // 7 jours
const STARTUP_DELAY:   u64   = 30;             // attendre 30s après boot

/// Lance la tâche de mise à jour en arrière-plan.
/// Appelée une fois depuis main.rs au démarrage.
pub fn start(gifs_dir: String) {
    tokio::spawn(async move {
        // Délai initial : laisser le container se stabiliser
        sleep(Duration::from_secs(STARTUP_DELAY)).await;

        loop {
            if let Err(e) = update_gifs(&gifs_dir).await {
                tracing::warn!(error = %e, "GIF update échoué — retry dans 7 jours");
            }
            sleep(Duration::from_secs(INTERVAL_SECS)).await;
        }
    });
}

async fn update_gifs(gifs_dir: &str) -> Result<(), String> {
    let api_key = match std::env::var("GIPHY_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
    {
        Some(k) => k,
        None => {
            tracing::info!("GIPHY_API_KEY absent — mise à jour GIFs ignorée (configurez GIPHY_API_KEY dans .env)");
            return Ok(());
        }
    };

    fs::create_dir_all(gifs_dir)
        .await
        .map_err(|e| format!("Impossible de créer {gifs_dir}: {e}"))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    tracing::info!(gifs_dir = %gifs_dir, "🎬 Mise à jour GIFs — début");

    let mut index_entries: Vec<serde_json::Value> = Vec::new();
    let mut total = 0usize;
    let mut ok    = 0usize;

    for (cat_key, cat_label, query) in THEMES {
        let url = format!(
            "{}?api_key={}&q={}&limit={}&rating=g&lang=fr",
            GIPHY_BASE,
            api_key,
            urlencoding::encode(query),
            GIFS_PER_THEME,
        );

        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!(status = %r.status(), theme = %cat_key, "Giphy réponse non-2xx");
                continue;
            }
            Err(e) => {
                tracing::warn!(error = %e, theme = %cat_key, "Giphy requête échouée");
                continue;
            }
        };

        let data: Value = resp.json().await.unwrap_or(serde_json::json!({}));
        let gifs = match data["data"].as_array() {
            Some(a) => a.clone(),
            None    => continue,
        };

        for (i, gif) in gifs.iter().enumerate() {
            total += 1;

            let gif_id = match gif["id"].as_str() {
                Some(id) => id,
                None     => continue,
            };

            // Préférer fixed_width (~100-200 kB) pour limiter la taille
            let gif_url = gif["images"]["fixed_width"]["url"]
                .as_str()
                .or_else(|| gif["images"]["original"]["url"].as_str())
                .unwrap_or("");

            if gif_url.is_empty() {
                continue;
            }

            let filename = format!("{}-{}-{}.gif", cat_key, i, &gif_id[..8.min(gif_id.len())]);
            let dest     = Path::new(gifs_dir).join(&filename);

            match client.get(gif_url).send().await {
                Ok(r) if r.status().is_success() => {
                    match r.bytes().await {
                        Ok(bytes) => {
                            if let Err(e) = fs::write(&dest, &bytes).await {
                                tracing::warn!(error = %e, file = %filename, "Écriture GIF échouée");
                                continue;
                            }
                            let size_kb = bytes.len() / 1024;
                            tracing::debug!(file = %filename, size_kb = %size_kb, "GIF téléchargé");
                            ok += 1;

                            index_entries.push(serde_json::json!({
                                "id":        gif_id,
                                "category":  cat_key,
                                "cat_label": cat_label,
                                "file":      filename,
                                "title":     gif["title"].as_str().unwrap_or(cat_key),
                                "size_kb":   size_kb,
                            }));
                        }
                        Err(e) => tracing::warn!(error = %e, "Lecture bytes GIF échouée"),
                    }
                }
                _ => tracing::warn!(file = %filename, "Téléchargement GIF échoué"),
            }

            // Petite pause pour rester dans les limites de taux Giphy
            sleep(Duration::from_millis(100)).await;
        }
    }

    // Générer index.json
    let index = serde_json::json!({
        "version":    1,
        "generated":  chrono::Utc::now().to_rfc3339(),
        "total":      ok,
        "categories": THEMES.iter().map(|(k, l, _)| [*k, *l]).collect::<Vec<_>>(),
        "gifs":       index_entries,
    });

    let index_path = Path::new(gifs_dir).join("index.json");
    fs::write(&index_path, serde_json::to_string_pretty(&index).unwrap_or_default())
        .await
        .map_err(|e| format!("Écriture index.json: {e}"))?;

    tracing::info!(ok = %ok, total = %total, "🎬 Mise à jour GIFs terminée");
    Ok(())
}
