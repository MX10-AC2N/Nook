use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct WebRtcState {
    pub sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

impl WebRtcState {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Deserialize)]
pub struct OfferRequest {
    pub offer: String,
    pub to: String,
}

#[derive(Serialize)]
pub struct OfferResponse {
    pub success: bool,
    pub offer: Option<String>,
    pub from: Option<String>,
}

pub async fn handle_offer(
    State(state): State<WebRtcState>,
    AxumJson(payload): AxumJson<OfferRequest>,
) -> Result<Json<OfferResponse>, StatusCode> {
    // Stocke l'offre pour le destinataire
    let mut sessions = state.sessions.write().await;
    sessions.insert(payload.to.clone(), payload.offer);
    Ok(Json(OfferResponse {
        success: true,
        offer: None,
        from: None,
    }))
}

pub async fn handle_answer(
    State(state): State<WebRtcState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<String, StatusCode> {
    if let Some(from) = params.get("from") {
        let sessions = state.sessions.read().await;
        if let Some(offer) = sessions.get(from) {
            return Ok(offer.clone());
        }
    }
    Err(StatusCode::NOT_FOUND)
}