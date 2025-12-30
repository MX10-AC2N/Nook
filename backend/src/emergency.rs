use axum::{
    extract::Json as AxumJson,
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct EmergencyRequest {
    pub message: String,
}

pub async fn handle_emergency(
    AxumJson(payload): AxumJson<EmergencyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Log de l'urgence en console (visible dans les logs du conteneur)
    println!("🚨 ALERTE D'URGENCE REÇUE : {}", payload.message);

    // TODO : Implémenter l'envoi réel selon ton choix
    // Exemples possibles :
    // - Envoi d'email via lettre
    // - Envoi de SMS via Twilio
    // - Notification push via Gotify ou ntfy.sh
    /*
    // Exemple avec lettre (SMTP) :
    use lettre::message::Message;
    use lettre::transport::smtp::async_smtp::AsyncSmtpTransport;
    use lettre::transport::smtp::client::Tls;
    use lettre::AsyncTransport;

    let email = Message::builder()
        .from("nook@tondomaine.com".parse().unwrap())
        .to("admin@tondomaine.com".parse().unwrap())
        .subject("🚨 ALERTE URGENCE NOOK")
        .body(format!("Message d'urgence :\n\n{}", payload.message))
        .unwrap();

    let mailer = AsyncSmtpTransport::relay("smtp.tondomaine.com")
        .unwrap()
        .credentials(lettre::transport::smtp::client::Credentials::new(
            "ton_user".to_string(),
            "ton_password".to_string(),
        ))
        .build();

    mailer.send(email).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    */

    // Réponse JSON standardisée
    Ok(Json(json!({
        "success": true,
        "message": "Alerte d'urgence reçue et enregistrée"
    })))
}
