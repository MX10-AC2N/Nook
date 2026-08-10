// csp.rs — CSP Nonce-based Middleware for Axum 0.8 + rand 0.9 + base64ct 1.6
// Génère un nonce aléatoire (32 bytes → base64url unpadded) par requête et injecte le header CSP strict
// Le nonce est stocké dans request.extensions() pour être accessible aux handlers/templates
// Le nonce est aussi injecté dans le HTML via remplacement de placeholder <!-- CSP_NONCE_PLACEHOLDER -->

use axum::{
    body::{Body, Bytes, to_bytes},
    http::{header::CONTENT_TYPE, HeaderValue, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use base64ct::{Base64UrlUnpadded, Encoding};
use rand::{rng, RngExt};

// -----------------------------------------------------------------------------
// CSP Configuration
// -----------------------------------------------------------------------------

/// CSP directives with nonce-based script-src and strict-dynamic
/// - script-src: 'self' + nonce + 'strict-dynamic' (allows trusted scripts to load other trusted scripts)
/// - style-src: 'self' 'unsafe-inline' (Svelte uses inline styles)
/// - connect-src: 'self' wss: https: (WebSocket + API + WebRTC)
/// - img-src: 'self' data: https: (images + data URIs + HTTPS images)
/// - font-src: 'self' data: (fonts + data URIs)
/// - object-src 'none' (no plugins)
/// - base-uri 'self' (prevent base tag hijacking)
/// - form-action 'self' (form submissions only to self)
/// - frame-ancestors 'none' (no framing)
const CSP_POLICY_TEMPLATE: &str = "default-src 'self'; \
    script-src 'self' 'nonce-{nonce}' 'strict-dynamic'; \
    style-src 'self' 'unsafe-inline'; \
    connect-src 'self' wss: https:; \
    img-src 'self' data: https:; \
    font-src 'self' data:; \
    object-src 'none'; \
    base-uri 'self'; \
    form-action 'self'; \
    frame-ancestors 'none'";

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

    /// Get the nonce value
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Build the CSP header value with the nonce inserted
    pub fn csp_header_value(&self) -> HeaderValue {
        let policy = CSP_POLICY_TEMPLATE.replace("{nonce}", &self.0);
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
    let nonce_value = nonce.value().to_string();

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
        // Add CSP header
        response.headers_mut().insert(
            axum::http::header::CONTENT_SECURITY_POLICY,
            csp_header_value,
        );

        // Inject nonce into HTML body
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
        
        let html = String::from_utf8_lossy(&bytes);
        let modified_html = html.replace(
            "<!-- CSP_NONCE_PLACEHOLDER -->",
            &format!("<script nonce=\"{}\"></script>", nonce_value),
        );
        
        let new_body = Body::from(modified_html);
        let mut new_response = Response::from_parts(parts, new_body);
        new_response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        
        return new_response;
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