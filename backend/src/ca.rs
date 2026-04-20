use axum::{
    http::header,
    response::{IntoResponse, Response},
};
use std::path::Path;

/// GET /ca — retourne le certificat CA pour installation
pub async fn get_ca_cert() -> Response {
    let cert_path = Path::new("/app/ssl/ca.crt");
    
    if !cert_path.exists() {
        return (
            axum::http::StatusCode::NOT_FOUND,
            "CA certificate not found. Restart nginx-local to generate it.",
        ).into_response();
    }
    
    match tokio::fs::read(cert_path).await {
        Ok(cert_data) => {
            let filename = "nook-ca.crt";
            Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "application/x-x509-ca-cert")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename="{}"", filename),
                )
                .body(axum::body::Body::from(cert_data))
                .unwrap()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read CA cert: {}", e),
        ).into_response(),
    }
}

/// GET /ca/help — page HTML d'aide pour installer le certificat
pub async fn ca_help() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Nook — Certificat CA</title>
  <style>
    *{box-sizing:border-box;margin:0;padding:0}
    body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#f5f5f5;color:#333;line-height:1.6;padding:20px}
    .c{max-width:600px;margin:0 auto;background:#fff;border-radius:12px;padding:24px;box-shadow:0 2px 8px rgba(0,0,0,.1)}
    h1{color:#2d5016;margin-bottom:8px}
    .sub{color:#666;margin-bottom:24px}
    .dl{display:block;background:#2d5016;color:#fff;text-align:center;padding:14px;border-radius:8px;text-decoration:none;font-weight:bold;margin-bottom:24px}
    .dl:hover{background:#3a6a1c}
    .s{display:flex;align-items:flex-start;gap:12px;margin-bottom:16px}
    .n{background:#2d5016;color:#fff;width:28px;height:28px;border-radius:50%;display:flex;align-items:center;justify-content:center;font-weight:bold;flex-shrink:0}
    .note{background:#fff3cd;border:1px solid #ffc107;border-radius:8px;padding:12px;font-size:14px;margin-top:16px}
    .tabs{display:flex;gap:8px;margin-bottom:16px}
    .tab{padding:8px 16px;border-radius:8px;background:#eee;cursor:pointer;border:none;font-size:14px}
    .tab.active{background:#2d5016;color:#fff}
    .tc{display:none}.tc.active{display:block}
  </style>
</head>
<body>
<div class="c">
  <h1>🔒 Certificat CA Nook</h1>
  <p class="sub">Installez ce certificat sur votre appareil pour activer les notifications push.</p>
  <a href="/ca" download class="dl">⬇️ Télécharger nook-ca.crt</a>
  <div class="tabs">
    <button class="tab active" onclick="s('a')">Android</button>
    <button class="tab" onclick="s('i')">iOS</button>
    <button class="tab" onclick="s('p')">PC / Mac</button>
  </div>
  <div id="a" class="tc active">
    <div class="s"><div class="n">1</div><div>Téléchargez le certificat via le bouton ci-dessus</div></div>
    <div class="s"><div class="n">2</div><div><b>Paramètres → Sécurité → Chiffrement et données → Installer un certificat</b></div></div>
    <div class="s"><div class="n">3</div><div>Sélectionnez <b>"Certificat CA"</b></div></div>
    <div class="s"><div class="n">4</div><div>Naviguez vers <b>nook-ca.crt</b> et confirmez</div></div>
    <div class="s"><div class="n">5</div><div><b>Redémarrez</b> votre navigateur, puis réessayez les notifications</div></div>
    <div class="note">⚠️ Samsung : Paramètres → Biométrie et sécurité → Autres paramètres → Certificats → Installer depuis le stockage</div>
  </div>
  <div id="i" class="tc">
    <div class="s"><div class="n">1</div><div>Téléchargez le certificat</div></div>
    <div class="s"><div class="n">2</div><div><b>Réglages → Général → VPN et gestion de l'appareil</b></div></div>
    <div class="s"><div class="n">3</div><div>Appuyez sur <b>"Nook Local CA"</b> puis <b>"Installer"</b></div></div>
    <div class="s"><div class="n">4</div><div><b>Réglages → Général → À propos → Réglages de confiance des certificats</b></div></div>
    <div class="s"><div class="n">5</div><div>Activez le certificat <b>"Nook Local CA"</b></div></div>
    <div class="note">⚠️ iOS affichera un avertissement — c'est normal, c'est votre propre CA</div>
  </div>
  <div id="p" class="tc">
    <div class="s"><div class="n">1</div><div>Téléchargez le certificat</div></div>
    <div class="s"><div class="n">2</div><div><b>Windows :</b> Double-cliquez → Installer → Ordinateur local → Autorités de certification racines</div></div>
    <div class="s"><div class="n">3</div><div><b>Mac :</b> Double-cliquez → Accès aux clés → "Toujours faire confiance"</div></div>
    <div class="note">⚠️ Firefox utilise son propre magasin — importez via Paramètres → Vie privée → Certificats</div>
  </div>
  <div class="note">📱 Après installation, revenez sur Nook et cliquez sur "Activer les notifications"</div>
</div>
<script>function s(id){document.querySelectorAll('.tc').forEach(e=>e.classList.remove('active'));document.querySelectorAll('.tab').forEach(e=>e.classList.remove('active'));document.getElementById(id).classList.add('active');event.target.classList.add('active')}</script>
</body>
</html>"#;

    Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(html))
        .unwrap()
}
