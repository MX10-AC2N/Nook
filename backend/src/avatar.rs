//! Local, deterministic avatar generation — replaces the DiceBear HTTP proxy.
//!
//! ## Why this exists
//! `avatar_proxy` used to forward `GET /avatar/{style}/svg` to `api.dicebear.com`.
//! The Nook homeserver has no outbound internet, so every request failed with
//! **HTTP 502 + an empty body** → broken `<img>` tags and 77 console errors on
//! the admin page. On top of being broken, proxying leaked the `seed`
//! (which may contain a username) to a third party, which contradicts Nook's
//! "privacy-first / self-hosted" posture.
//!
//! ## Strategy (pure-local, no network)
//! Avatars are generated here, in Rust, from the `seed` using `sha2` (already a
//! project dependency via `csp.rs`) and the standard library only — no new
//! crates. DiceBear is NOT used even as a preferred source: the homeserver
//! cannot reach it, so attempting it would only ever re-introduce the 502/latency
//! and the seed leak. The local generator is therefore the single source of truth.
//!
//! Guarantees:
//! - **deterministic** — same `seed` + `style` → identical SVG (same `size`
//!   only changes the `width`/`height` attributes, not the content);
//! - **stateless & network-free** — the handler can never 502 or block on I/O;
//! - **no seed leak** — the `seed` is hashed and never echoed back verbatim;
//! - **style-distinct** — each style maps to its own cell primitive + palette.

use axum::{
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use sha2::{Digest, Sha256};

/// Grid is `GRID × GRID` cells of `UNIT` user-space units.
/// viewBox = `0 0 70 70`, so a `size=32` avatar keeps exact geometry.
const GRID: usize = 7;
const UNIT: usize = 10;
/// Columns we independently choose; the rest are mirrored for symmetry.
const HALF: usize = GRID.div_ceil(2);

/// Number of bits we read from the hash for the cell grid.
const GRID_BITS: usize = GRID * HALF; // 28, fits in 4 bytes

/// `(background, foreground)` color pairs as hex strings with leading `#`.
const PALETTES: [(&str, &str); 12] = [
    ("#6c5ce7", "#ffffff"),
    ("#0984e3", "#ffffff"),
    ("#00b894", "#ffffff"),
    ("#f5af19", "#ffffff"),
    ("#e17055", "#ffffff"),
    ("#e84393", "#ffffff"),
    ("#55efc4", "#2d3436"),
    ("#74b9ff", "#ffffff"),
    ("#a29bfe", "#ffffff"),
    ("#ffeaa7", "#2d3436"),
    ("#fab1a0", "#2d3436"),
    ("#81ecec", "#2d3436"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Square,
    Circle,
    RoundedSquare,
    Triangle,
    Diamond,
}

/// Per-style definition: the cell primitive, the chosen palette, and whether the
/// canvas is clipped to a circle (face-like styles).
struct StyleDef {
    shape: Shape,
    palette: usize,
    round: bool,
}

fn style_def(style: &str) -> StyleDef {
    match style {
        "adventurer" => StyleDef {
            shape: Shape::RoundedSquare,
            palette: 0,
            round: false,
        },
        "avataaars" => StyleDef {
            shape: Shape::Circle,
            palette: 1,
            round: true,
        },
        "open-peeps" => StyleDef {
            shape: Shape::Circle,
            palette: 2,
            round: true,
        },
        "notionists" => StyleDef {
            shape: Shape::Square,
            palette: 3,
            round: false,
        },
        "fun-emoji" => StyleDef {
            shape: Shape::Circle,
            palette: 4,
            round: true,
        },
        "big-smile" => StyleDef {
            shape: Shape::Circle,
            palette: 5,
            round: true,
        },
        "lorelei" => StyleDef {
            shape: Shape::Diamond,
            palette: 6,
            round: false,
        },
        "personas" => StyleDef {
            shape: Shape::RoundedSquare,
            palette: 7,
            round: true,
        },
        "bottts" => StyleDef {
            shape: Shape::Triangle,
            palette: 8,
            round: false,
        },
        _ => StyleDef {
            shape: Shape::RoundedSquare,
            palette: 0,
            round: false,
        },
    }
}

/// Deterministic 256-bit hash of `seed` + `style`.
///
/// Folding `style` into the hash guarantees two different styles with the same
/// seed produce different grids. The raw `seed` is never emitted to the client.
fn hash_seed(seed: &str, style: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    h.update(b"\x1f");
    h.update(style.as_bytes());
    let d = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&d);
    out
}

/// Draw a single cell of `shape` at grid position `(col, row)` (0..GRID).
/// All coordinates are integers in the 0..70 viewBox; each cell is inset by 1
/// unit so adjacent cells show a small gap.
fn cell_shape(shape: Shape, col: usize, row: usize) -> String {
    let x = col * UNIT + 1;
    let y = row * UNIT + 1;
    let cx = col * UNIT + UNIT / 2;
    let cy = row * UNIT + UNIT / 2;
    let x8 = x + 8;
    let y8 = y + 8;
    match shape {
        Shape::Square => format!(r#"<rect x="{x}" y="{y}" width="8" height="8"/>"#),
        Shape::Circle => format!(r#"<circle cx="{cx}" cy="{cy}" r="4"/>"#),
        Shape::RoundedSquare => {
            format!(r#"<rect x="{x}" y="{y}" width="8" height="8" rx="2" ry="2"/>"#)
        }
        Shape::Triangle => format!(r#"<polygon points="{x},{y} {x8},{y} {cx},{y8}"/>"#),
        Shape::Diamond => format!(r#"<polygon points="{cx},{y} {x8},{cy} {cx},{y8} {x},{cy}"/>"#),
    }
}

/// Generate the SVG identicon string for any style except `initials`.
///
/// `size` is the rendered pixel size (width × height); the internal viewBox is
/// fixed at 70×70 so geometry is resolution-independent.
pub fn generate_svg(style: &str, seed: &str, size: u32) -> String {
    let def = style_def(style);
    let hash = hash_seed(seed, style);
    let (bg, fg) = PALETTES[def.palette];

    // Symmetric grid: choose `HALF` columns per row, mirror the remainder.
    let mut cells = String::with_capacity(GRID_BITS * 2 * 40);
    for row in 0..GRID {
        for col in 0..HALF {
            let idx = row * HALF + col;
            let on = (hash[idx / 8] >> (idx % 8)) & 1 == 1;
            if on {
                cells.push_str(&cell_shape(def.shape, col, row));
                let mirror = GRID - 1 - col;
                if mirror != col {
                    cells.push_str(&cell_shape(def.shape, mirror, row));
                }
            }
        }
    }

    let (defs, bg_elt) = if def.round {
        (
            r#"<defs><clipPath id="nook-av-cl"><circle cx="35" cy="35" r="34"/></clipPath></defs>"#
                .to_string(),
            format!(r#"<circle cx="35" cy="35" r="34" fill="{bg}"/>"#),
        )
    } else {
        (
            String::new(),
            format!(r#"<rect width="70" height="70" fill="{bg}"/>"#),
        )
    };

    let cell_group = if def.round {
        format!(r#"<g fill="{fg}" clip-path="url(#nook-av-cl)">{cells}</g>"#)
    } else {
        format!(r#"<g fill="{fg}">{cells}</g>"#)
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 70 70" shape-rendering="crispEdges">{defs}{bg_elt}{cell_group}</svg>"#
    )
}

/// Build the `initials` avatar: the first letters of each word in `seed`
/// (padded with non-initial letters for mononyms), rendered as 2 uppercase
/// initials on a solid colored background. The raw `seed` is never embedded.
pub fn generate_initials_svg(seed: &str, size: u32) -> String {
    let hash = hash_seed(seed, "initials");
    let (bg, fg) = PALETTES[(hash[0] as usize) % PALETTES.len()];
    let init = extract_initials(seed);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 70 70" shape-rendering="crispEdges"><rect width="70" height="70" fill="{bg}"/><text x="35" y="47" fill="{fg}" font-family="sans-serif" font-weight="700" font-size="36" text-anchor="middle">{init}</text></svg>"#
    )
}

/// Extract up to 2 distinct uppercase initials from `seed`.
///
/// Word-initial letters are preferred (so `"Mo Ju"` → `"MJ"`), then any
/// remaining distinct letters for mononyms (so `"alice"` → `"AL"`). Falls back
/// to `"N"` when there are no alphabetic characters at all.
fn extract_initials(seed: &str) -> String {
    let mut out = String::new();
    let mut seen: Vec<char> = Vec::new();
    let mut at_word_start = true;

    for c in seed.chars() {
        if c.is_whitespace() {
            at_word_start = true;
            continue;
        }
        let is_alpha = c.is_ascii_alphabetic();
        if at_word_start && is_alpha {
            let up = c.to_ascii_uppercase();
            if !seen.contains(&up) {
                seen.push(up);
                out.push(up);
            }
        }
        at_word_start = false;
        if out.len() >= 2 {
            return out;
        }
    }

    // Pad with any remaining distinct letters (handles mononyms).
    if out.len() < 2 {
        for c in seed.chars() {
            if c.is_ascii_alphabetic() {
                let up = c.to_ascii_uppercase();
                if !seen.contains(&up) {
                    seen.push(up);
                    out.push(up);
                }
            }
            if out.len() >= 2 {
                break;
            }
        }
    }

    if out.is_empty() {
        out.push('N');
    }
    out
}

/// `GET /avatar/{style}/svg` handler.
///
/// Generates avatars locally (see [`generate_svg`] / [`generate_initials_svg`])
/// — never proxies to an external service — so it always returns HTTP 200 with
/// an `image/svg+xml` body and can never 502 or block on network I/O. The
/// `seed` is folded into a SHA-256 digest and is therefore never echoed back.
pub async fn avatar_proxy(
    Path(style): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let valid_styles = [
        "adventurer", "avataaars", "open-peeps", "notionists", "fun-emoji",
        "big-smile", "lorelei", "personas", "bottts", "initials",
    ];

    let style = if valid_styles.contains(&style.as_str()) {
        style
    } else {
        "adventurer".to_string()
    };

    let seed = params
        .get("seed")
        .cloned()
        .unwrap_or_else(|| "nook".to_string());
    // Default 32, capped at 512, floored at 1 (never emit a 0×0 image).
    let size = params
        .get("size")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(32)
        .clamp(1, 512);

    let svg = if style == "initials" {
        generate_initials_svg(&seed, size)
    } else {
        generate_svg(&style, &seed, size)
    };

    // 200 is guaranteed: generation is purely local and cannot fail.
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("image/svg+xml"));
    headers.insert(
        "cache-control",
        HeaderValue::from_static("public, max-age=31536000"),
    );
    (StatusCode::OK, headers, svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_same_seed_and_style() {
        let a = generate_svg("adventurer", "test", 32);
        let b = generate_svg("adventurer", "test", 32);
        assert_eq!(a, b, "same seed+style must be byte-identical");
    }

    #[test]
    fn different_seeds_differ() {
        let a = generate_svg("adventurer", "test", 32);
        let b = generate_svg("adventurer", "autre", 32);
        assert_ne!(a, b, "different seeds must produce different SVGs");
    }

    #[test]
    fn different_styles_are_distinct() {
        let adventurer = generate_svg("adventurer", "test", 32);
        let bottts = generate_svg("bottts", "test", 32);
        let avataaars = generate_svg("avataaars", "test", 32);
        assert_ne!(adventurer, bottts);
        assert_ne!(adventurer, avataaars);
        assert_ne!(bottts, avataaars);
    }

    #[test]
    fn svg_is_well_formed_and_sized() {
        let svg = generate_svg("lorelei", "test", 128);
        assert!(
            svg.starts_with(r#"<svg xmlns="#),
            "must start with an <svg> tag"
        );
        assert!(svg.ends_with("</svg>"), "must be closed");
        assert!(svg.contains(r#"width="128""#));
        assert!(svg.contains(r#"height="128""#));
        assert!(svg.contains(r#"viewBox="0 0 70 70""#));
        // the handler (not the generator) sets content-type; keep it out of the body
        assert!(!svg.contains("image/svg+xml"));
    }

    #[test]
    fn size_attribute_reflects_input() {
        let a = generate_svg("bottts", "test", 16);
        let b = generate_svg("bottts", "test", 512);
        assert!(a.contains(r#"width="16""#));
        assert!(b.contains(r#"width="512""#));
    }

    #[test]
    fn round_family_uses_circle_clip() {
        // circle-family styles are clipped to a circle
        assert!(generate_svg("avataaars", "test", 32).contains("clipPath"));
        assert!(!generate_svg("adventurer", "test", 32).contains("clipPath"));
        // square-family styles have a plain <rect> background (no clip)
        let sq = generate_svg("adventurer", "test", 32);
        assert!(sq.contains("<rect"));
        // bottts now uses triangles — exercises every shape variant, no dead code
        let bottts = generate_svg("bottts", "test", 32);
        assert!(bottts.contains("<polygon"));
    }

    #[test]
    fn initials_two_words() {
        let svg = generate_initials_svg("Mo Ju", 64);
        assert!(svg.contains("MJ"));
        assert!(svg.starts_with(r#"<svg xmlns="#));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn initials_mononym_pads() {
        assert!(generate_initials_svg("alice", 32).contains("AL"));
    }

    #[test]
    fn initials_no_alpha_falls_back_to_n() {
        assert!(generate_initials_svg("123 456", 32).contains(">N<"));
    }

    #[test]
    fn initials_deterministic() {
        assert_eq!(
            generate_initials_svg("Mo Ju", 32),
            generate_initials_svg("Mo Ju", 32)
        );
    }

    #[test]
    fn seed_never_leaked() {
        let secret = "supersecret_username_42";
        let svg = generate_svg("adventurer", secret, 32);
        assert!(!svg.contains("supersecret_username_42"));
        let s2 = generate_initials_svg(secret, 32);
        assert!(!s2.contains("supersecret_username_42"));
    }

    #[test]
    fn hash_feeds_grid_bits() {
        let h = hash_seed("x", "adventurer");
        assert_eq!(h.len(), 32, "sha256 -> 32 bytes");
        assert_eq!(h, hash_seed("x", "adventurer"), "deterministic hash");
        assert_ne!(
            h,
            hash_seed("y", "adventurer"),
            "different seed -> different hash"
        );
    }
}

#[cfg(test)]
mod http_tests {
    use super::avatar_proxy;
    use axum::{body::Body, extract::Request, http::StatusCode, routing::get, Router};
    use tower::ServiceExt;

    /// Mirror of the route declared in `main.rs`.
    fn app() -> Router {
        Router::new().route("/avatar/{style}/svg", get(avatar_proxy))
    }

    #[tokio::test]
    async fn proxy_returns_200_svg_and_headers() {
        let req = Request::builder()
            .uri("/avatar/adventurer/svg?seed=test&size=32")
            .body(Body::empty())
            .unwrap();
        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "image/svg+xml"
        );
        assert!(resp.headers().get("cache-control").is_some());
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(svg.starts_with("<svg"), "body must be an SVG");
        assert!(svg.ends_with("</svg>"), "SVG must be closed");
        assert!(!svg.contains("test"), "seed must never be echoed verbatim");
    }

    #[tokio::test]
    async fn proxy_initials_emits_letters() {
        // `Mo Ju` -> initials `MJ`; seed is URL-encoded.
        let req = Request::builder()
            .uri("/avatar/initials/svg?seed=Mo%20Ju&size=64")
            .body(Body::empty())
            .unwrap();
        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(svg.contains("MJ"), "expected initials 'MJ' in: {svg}");
    }

    #[tokio::test]
    async fn proxy_is_deterministic() {
        async fn fetch(uri: &str) -> Vec<u8> {
            let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
            let resp = app().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec()
        }
        let a = fetch("/avatar/bottts/svg?seed=test&size=32").await;
        let b = fetch("/avatar/bottts/svg?seed=test&size=32").await;
        assert_eq!(a, b, "identical request must yield byte-identical SVG");
    }

    #[tokio::test]
    async fn proxy_falls_back_on_unknown_style() {
        let req = Request::builder()
            .uri("/avatar/does-not-exist/svg?seed=test")
            .body(Body::empty())
            .unwrap();
        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(
            String::from_utf8(bytes.to_vec()).unwrap().starts_with("<svg"),
            "unknown style falls back to adventurer (still 200 + SVG)"
        );
    }
}

/// Live HTTP variant: binds a *real* TCP socket, serves the actual
/// `avatar_proxy` handler through a real `axum::Router`, and issues real
/// HTTP/1.1 GET requests via `reqwest`. This is the closest in-process analog
/// to the task's `curl` verification and proves the endpoint behaves correctly
/// at the wire level (status, content-type, cache-control, body, determinism,
/// seed-variety) — without depending on the full `main()` server startup
/// (which needs `/app/data` + a DB and can't run in every dev sandbox).
#[cfg(test)]
mod live_http_tests {
    use super::avatar_proxy;
    use axum::{Router, routing::get};
    use sha2::{Digest, Sha256};

    async fn spawn_server() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();
        let app = Router::new().route("/avatar/{style}/svg", get(avatar_proxy));
        // Detach the server; it lives for the duration of the test runtime.
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve avatar");
        });
        // Yield once so the accept loop is ready before we connect.
        tokio::task::yield_now().await;
        format!("http://{addr}")
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn live_avatar_returns_200_with_svg_and_caching() {
        let base = spawn_server().await;

        let resp = reqwest::get(format!(
            "{base}/avatar/adventurer/svg?seed=test&size=32"
        ))
        .await
        .expect("GET succeeds");

        // --- status ---
        assert_eq!(resp.status(), 200);
        eprintln!("[live] status = {}", resp.status());

        // --- headers ---
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert_eq!(ct, "image/svg+xml");
        let cc = resp.headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cc, "public, max-age=31536000");
        eprintln!("[live] content-type = {ct}");
        eprintln!("[live] cache-control = {cc}");

        // --- body ---
        let body = resp.bytes().await.expect("read body");
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        eprintln!("[live] body[:120] = {}", body_str.chars().take(120).collect::<String>());
        assert!(body_str.starts_with("<svg"), "body must be an SVG");
        assert!(body_str.ends_with("</svg>"), "SVG must be closed");
        assert!(!body_str.contains("test"), "seed must never be echoed verbatim");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn live_avatar_is_deterministic_and_seed_distinct() {
        let base = spawn_server().await;

        // Two identical requests MUST produce byte-identical output.
        let a = reqwest::get(format!("{base}/avatar/bottts/svg?seed=test&size=64"))
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let b = reqwest::get(format!("{base}/avatar/bottts/svg?seed=test&size=64"))
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(a, b, "identical request must be byte-identical");
        let ha = sha256_hex(&a);
        let hb = sha256_hex(&b);
        eprintln!("[live] determinism: identical seed sha256 = {ha}");
        assert_eq!(ha, hb);

        // Two different seeds MUST produce different output.
        let c = reqwest::get(format!("{base}/avatar/adventurer/svg?seed=autre&size=64"))
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let hc = sha256_hex(&c);
        eprintln!("[live] variety: seed 'autre' sha256 = {hc}");
        assert_ne!(ha, hc, "different seeds must differ");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn live_initials_avatar_from_url() {
        let base = spawn_server().await;
        // URL-encoded space in seed ("Mo Ju") — mirrors the task verification.
        let resp = reqwest::get(format!(
            "{base}/avatar/initials/svg?seed=Mo%20Ju&size=64"
        ))
        .await
        .unwrap();
        assert_eq!(resp.status(), 200);
        let body = String::from_utf8(
            resp.bytes().await.unwrap().to_vec(),
        )
        .unwrap();
        eprintln!("[live] initials body[:80] = {}", body.chars().take(80).collect::<String>());
        assert!(body.contains("MJ"), "expected initials 'MJ' in: {body}");
    }

}
