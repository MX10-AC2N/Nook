use axum::{http::StatusCode, response::Json, Json as AxumJson};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EmergencyRequest {
    pub message: String,
}

pub async fn handle_emergency(
    AxumJson(payload): AxumJson<EmergencyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // À implémenter selon ton choix :
    // - SMTP (email via lettre)
    // - Twilio (SMS)
    // - Gotify (notifications push auto-hébergées)

    println!("🚨 URGENCE : {}", payload.message);

    // Exemple SMTP basique (à étendre)
    /*
    let email = Message::builder()
        .from("nook@yourdomain.com".parse().unwrap())
        .to("admin@yourdomain.com".parse().unwrap())
        .subject("ALERTE NOOK")
        .body(payload.message)
        .unwrap();

    lettre::AsyncSmtpTransport::relay("smtp.yourdomain.com")
        .unwrap()
        .credentials(Credentials::new("user".into(), "pass".into()))
        .build()
        .send(email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    */

    Ok(Json(serde_json::json!({ "success": true })))
}
