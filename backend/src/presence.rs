// backend/src/presence.rs
// Gestion du statut en ligne (presence)

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Extension, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::CurrentUser;
use crate::SharedState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub username: String,
    pub online: bool,
    pub last_seen: i64,
}

#[derive(Clone)]
pub struct PresenceState {
    /// Map user_id → last_seen timestamp
    pub online_users: Arc<Mutex<HashMap<String, i64>>>,
}

impl PresenceState {
    pub fn new() -> Self {
        Self {
            online_users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Marquer un utilisateur comme en ligne
    pub async fn set_online(&self, user_id: &str) {
        let now = Utc::now().timestamp();
        let mut users = self.online_users.lock().await;
        users.insert(user_id.to_string(), now);
    }

    /// Marquer un utilisateur comme hors ligne
    pub async fn set_offline(&self, user_id: &str) {
        let mut users = self.online_users.lock().await;
        users.remove(user_id);
    }

    /// Mettre à jour le last_seen d'un utilisateur
    pub async fn heartbeat(&self, user_id: &str) {
        let now = Utc::now().timestamp();
        let mut users = self.online_users.lock().await;
        users.insert(user_id.to_string(), now);
    }

    /// Obtenir la liste des utilisateurs en ligne
    pub async fn get_online_users(&self) -> Vec<String> {
        let users = self.online_users.lock().await;
        users.keys().cloned().collect()
    }

    /// Vérifier si un utilisateur est en ligne (activité dans les 5 dernières minutes)
    pub async fn is_online(&self, user_id: &str) -> bool {
        let users = self.online_users.lock().await;
        if let Some(last_seen) = users.get(user_id) {
            let now = Utc::now().timestamp();
            // Considéré en ligne si actif dans les 5 dernières minutes
            (now - last_seen) < 300
        } else {
            false
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Routes
// ─────────────────────────────────────────────────────────────────────────────

pub fn presence_routes() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/presence", get(get_presence))
        .route("/presence/heartbeat", get(heartbeat))
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/presence — Obtenir le statut de tous les utilisateurs
async fn get_presence(
    State(state): State<Arc<SharedState>>,
    Extension(_user): Extension<CurrentUser>,
) -> Result<Json<Vec<UserPresence>>, StatusCode> {
    // Récupérer tous les utilisateurs approuvés
    let users = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, username, COALESCE(name, username) FROM users WHERE approved = 1"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Erreur récupération utilisateurs");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut presences = Vec::new();
    
    for (id, _username, name) in users {
        let online = state.presence_state.is_online(&id).await;
        let last_seen = if online {
            Utc::now().timestamp()
        } else {
            // Pour les utilisateurs hors ligne, on pourrait stocker le last_seen en DB
            // Pour l'instant, on retourne 0
            0
        };
        
        presences.push(UserPresence {
            user_id: id,
            username: name,
            online,
            last_seen,
        });
    }

    Ok(Json(presences))
}

/// GET /api/push/heartbeat — Heartbeat pour maintenir le statut en ligne
async fn heartbeat(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> Result<impl IntoResponse, StatusCode> {
    state.presence_state.heartbeat(&user.id).await;
    Ok(StatusCode::OK)
}
