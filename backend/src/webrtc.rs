use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::SharedState;

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
    State(state): State<SharedState>,
    Json(payload): Json<OfferRequest>,
) -> Result<Json<OfferResponse>, StatusCode> {
    let mut sessions = state.webrtc_sessions.write().await;
    sessions.insert(payload.to.clone(), payload.offer);
    Ok(Json(OfferResponse {
        success: true,
        offer: None,
        from: None,
    }))
}

pub async fn handle_answer(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<String, StatusCode> {
    if let Some(from) = params.get("from") {
        let sessions = state.webrtc_sessions.read().await;
        if let Some(offer) = sessions.get(from) {
            return Ok(offer.clone());
        }
    }
    Err(StatusCode::NOT_FOUND)
}
