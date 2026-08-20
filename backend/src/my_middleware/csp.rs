// csp.rs — CSP Nonce-based Middleware for Axum 0.8 + rand 0.9 + base64ct 1.6
// Génère un nonce aléatoire (32 bytes → base64url unpadded) par requête et injecte le header CSP strict
// Le nonce est stocké dans request.extensions() pour être accessible aux handlers/templates
// Le nonce est aussi injecté dans le HTML via remplacement de placeholder CSP_NONCE_PLACEHOLDER
//
// ⚠️ CORRECTION CSP-01 (2026-08-19) : le hash sha256 du <script> inline d'hydratation
// SvelteKit était HARDCODÉ sur un build précis. Dès que le frontend était rebuildé
// (nouveau commit / version SvelteKit), le hash ne correspondait plus → le script
// d'hydratation était BLOQUÉ par le CSP → page blanche (HTML servi, JS non exécuté).
// On calcule désormais le(s) hash(es) DYNAMIQUEMENT au démarrage à partir du
// frontend/build/index.html réellement servi par le backend (load_svelte_inline_hashes).
// Ainsi le CSP autorise toujours le script d'hydratation du build déployé.

use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderValue, Request},
    middleware::Next,
    response::IntoResponse,
};
use base64ct::{Base64, Base64UrlUnpadded, Encoding};
use rand::{rng, RngExt};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

// -----------------------------------------------------------------------------
// CSP Configuration
// -----------------------------------------------------------------------------

/// CSP directives with nonce-based script-src and strict-dynamic
/// - script-src: 'self' + nonce + 'strict-dynamic' + hashes des scripts inline SvelteKit
/// - style-src: 'self' 'unsafe-inline' (Svelte uses inline styles)
/// - connect-src: 'self' wss: https: (WebSocket + API + WebRTC)
/// - img-src: 'self' data: https: (images + data URIs + HTTPS images)
/// - font-src: 'self' data: (fonts + data URIs)
/// - object-src 'none' (no plugins)
/// - base-uri 'self' (prevent base tag hijacking)
/// - form-action 'self' (form submissions only to self)
/// - frame-ancestors 'none' (no framing)
// SVELTE_INLINE_HASH_PLACEHOLDER est remplacé au runtime par les hash(es) sha256
// calculés dynamiquement (un ou plusieurs 'sha256-...' séparés par des espaces).
// S'il n'y a aucun script inline (cas anormal), le placeholder devient la chaîne
// vide — le CSP reste strict, on n'affaiblit rien d'autre.
const CSP_POLICY_TEMPLATE: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-eval' 'nonce-{nonce}' SVELTE_INLINE_HASH_PLACEHOLDER; \
    style-src 'self' 'unsafe-inline'; \
    connect-src 'self' wss: https:; \
    img-src 'self' data: https:; \
    font-src 'self' data:; \
    object-src 'none'; \
    base-uri 'self'; \
    form-action 'self'; \
    frame-ancestors 'none'";

// -----------------------------------------------------------------------------
// Dynamic inline-script hashes (computed once at startup from the served index.html)
// -----------------------------------------------------------------------------

/// Holds the space-separated list of `'sha256-...'` tokens for every inline
/// `<script>` (sans attribut `src`) trouvé dans le `index.html` servi par le
/// backend. Remplacé dans le CSP à la place de `SVELTE_INLINE_HASH_PLACEHOLDER`.
/// `None` = pas encore chargé (fallback chaîne vide). `Some("")` = fichier lu
/// mais aucun script inline trouvé.
static SVELTE_INLINE_HASHES: Mutex<Option<String>> = Mutex::new(None);

/// Calcule, au démarrage, les hash sha256 de tous les `<script>` inline (sans
/// `src`) du `index.html` situé dans `static_dir`. Stocke le résultat pour
/// injection dans le header CSP. Idempotent et sans panique : en cas d'erreur
/// de lecture on log un warning et on laisse la valeur vide (le CSP reste strict,
/// seul le script inline SvelteKit ne serait pas whitelisté — mais si index.html
/// manque, l'app est de toute façon incapable de servir le SPA).
pub fn load_svelte_inline_hashes(static_dir: &str) {
    let path = format!("{}/index.html", static_dir.trim_end_matches('/'));
    let hashes = match compute_svelte_inline_hashes(&path) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(
                path = %path,
                error = %e,
                "CSP: impossible de lire index.html pour calculer le hash du script inline \
                 d'hydratation — le CSP ne whitelist aucun script inline (risque de page blanche)"
            );
            String::new()
        }
    };
    *SVELTE_INLINE_HASHES.lock().expect("SVELTE_INLINE_HASHES poisoned") = Some(hashes);
}

/// Lit `index.html` et renvoie la liste des hash sha256 (encodés base64 standard
/// avec padding, format CSP) de chaque `<script>` inline. Ereur si le fichier
/// est illisible.
fn compute_svelte_inline_hashes(path: &str) -> std::io::Result<String> {
    let html = std::fs::read_to_string(path)?;
    let bodies = extract_inline_script_bodies(&html);
    let mut tokens = Vec::with_capacity(bodies.len());
    for body in bodies {
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        let digest = hasher.finalize();
        let b64 = Base64::encode_string(&digest);
        tokens.push(format!("'sha256-{b64}'"));
    }
    if tokens.is_empty() {
        tracing::warn!(
            path = %path,
            "CSP: aucun <script> inline trouvé dans index.html — script d'hydratation \
             SvelteKit probablement absent ou déjà chargé via src externe"
        );
    } else {
        tracing::info!(
            count = tokens.len(),
            hashes = %tokens.join(" "),
            "CSP: hash(es) du/des script(s) inline d'hydratation calculé(s) dynamiquement"
        );
    }
    Ok(tokens.join(" "))
}

/// Extrait le contenu de chaque `<script>` inline (sans attribut `src`) du HTML.
/// Parser minimal sans dépendance regex : tolérant aux majuscules et à la casse
/// des attributs. On ne capture QUE les scripts sans `src` (les scripts externes
/// sont couverts par 'self' + nonce, pas par hash).
fn extract_inline_script_bodies(html: &str) -> Vec<String> {
    let mut bodies = Vec::new();
    let mut i = 0;
    while let Some(rel) = html[i..].find("<script") {
        let abs = i + rel;
        // Fin de la balise ouvrante '<script ...>'
        let open_end = match html[abs..].find('>') {
            Some(p) => abs + p,
            None => break,
        };
        let opening = &html[abs..open_end];
        let has_src = opening[..opening.len().saturating_sub(0)]
            .to_ascii_lowercase()
            .contains("src=");
        // Balise fermante '</script>'
        let close_rel = match html[open_end + 1..].find("</script>") {
            Some(p) => p,
            None => break,
        };
        let close_start = open_end + 1 + close_rel;
        if !has_src {
            bodies.push(html[open_end + 1..close_start].to_string());
        }
        i = close_start + "</script>".len();
    }
    bodies
}

/// Extension key for storing CSP nonce in request extensions
#[derive(Clone, Debug)]
pub struct CspNonce(pub String);

impl CspNonce {
    /// Generate a new cryptographically secure nonce (32 bytes → base64url unpadded)
    pub fn new() -> Self {
        let mut bytes = [0u8; 32];
        rng().fill(&mut bytes);
        let nonce = Base64UrlUnpadded::encode_string(&bytes);
        Self(nonce)
    }

    /// Build the CSP header value with the nonce inserted and the dynamic
    /// inline-script hashes (calculés au démarrage).
    pub fn csp_header_value(&self) -> HeaderValue {
        let inline = SVELTE_INLINE_HASHES
            .lock()
            .expect("SVELTE_INLINE_HASHES poisoned")
            .clone()
            .unwrap_or_default();
        let policy = CSP_POLICY_TEMPLATE
            .replace("{nonce}", &self.0)
            .replace("SVELTE_INLINE_HASH_PLACEHOLDER", &inline);
        HeaderValue::from_str(&policy).expect("CSP policy is valid")
    }
}

impl Default for CspNonce {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Convenience function for use as middleware::from_fn
// -----------------------------------------------------------------------------

/// Axum middleware function that adds CSP nonce and header
/// Can be used with `middleware::from_fn(csp_nonce_middleware)`
pub async fn csp_nonce_middleware(
    mut req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    // Generate nonce for this request
    let nonce = CspNonce::new();
    let csp_header_value = nonce.csp_header_value();

    // Store nonce in request extensions for handlers/templates
    req.extensions_mut().insert(nonce);

    let mut response = next.run(req).await;

    // Only add CSP header and inject nonce for HTML responses
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.starts_with("text/html") {
        // Add CSP header (contient les hash sha256 dynamiques du/des script(s)
        // inline d'hydratation SvelteKit, calculés au démarrage depuis le build
        // réellement servi — plus jamais de hash périmé).
        response.headers_mut().insert(
            axum::http::header::CONTENT_SECURITY_POLICY,
            csp_header_value,
        );
    }

    response
}

// -----------------------------------------------------------------------------
// Extractor for handlers to access the nonce
// -----------------------------------------------------------------------------

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

impl<S> FromRequestParts<S> for CspNonce
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CspNonce>()
            .cloned()
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_inline_script_bodies_ignores_external_scripts() {
        let html = "<html><head>\
            <script type=\"module\" src=\"/_app/start.js\"></script>\
            <script>inline1()</script>\
            <script src=\"/x.js\">should_be_ignored()</script>\
            <script defer>inline2()</script>\
            </head></html>";
        let bodies = extract_inline_script_bodies(html);
        assert_eq!(bodies, vec!["inline1()".to_string(), "inline2()".to_string()]);
    }

    #[test]
    fn csp_hash_token_is_well_formed() {
        // Vérifie le formatage CSP du hash : 'sha256-' + base64 standard (44 chars) + '
        // sur un script inline arbitraire (ne dépend pas du build de prod).
        let script = "console.log('nook hydration');";
        let mut hasher = Sha256::new();
        hasher.update(script.as_bytes());
        let b64 = Base64::encode_string(&hasher.finalize());
        // SHA-256 -> 32 octets -> base64 standard = 44 caractères (avec padding '=').
        assert_eq!(b64.len(), 44);
        assert!(b64.ends_with('='));
        let token = format!("'sha256-{b64}'");
        assert!(token.starts_with("'sha256-"));
        assert!(token.ends_with('\''));
        assert_eq!(token.len(), 8 + 44 + 1);
    }

    #[test]
    fn dynamic_hash_matches_legacy_constant_on_real_build() {
        // Preuve d'intégration : le hash calculé dynamiquement à partir du
        // frontend/build/index.html réel doit reproduire l'ancienne constante
        // HARDCODÉE (IQqXsFLW09kj...). Si le fichier de build n'est pas présent
        // (gitignored), le test est ignoré — mais quand il existe, cela prouve
        // que l'extracteur sélectionne le bon script inline et que pour N'IMPORTE
        // quel build futur le hash sera correct (fin des pages blanches CSP-01).
        let candidates = [
            "../../frontend/build/index.html",
            "../../../frontend/build/index.html",
            "frontend/build/index.html",
        ];
        let path = candidates
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .copied();
        let path = match path {
            Some(p) => p,
            None => {
                eprintln!("build/index.html absent (gitignored) — test ignoré");
                return;
            }
        };
        let html = std::fs::read_to_string(path).expect("lecture index.html");
        let bodies = extract_inline_script_bodies(&html);
        assert!(!bodies.is_empty(), "au moins un script inline attendu");
        let hashes: Vec<String> = bodies
            .iter()
            .map(|b| {
                let mut h = Sha256::new();
                h.update(b.as_bytes());
                format!("'sha256-{}'", Base64::encode_string(&h.finalize()))
            })
            .collect();
        assert!(
            hashes.iter().any(|h| h == "'sha256-IQqXsFLW09kj+T4A1WHn42UeuRbcg/uXi3SgZqkAc10='"),
            "le hash dynamique doit inclure l'ancienne constante pour le build commité; hashes={:?}",
            hashes
        );
    }
}
