---
name: nook-secrets-management
category: devops
description: "Remove hardcoded secrets from frontend bundles — backend API pattern for TURN/STUN credentials, HMAC generation, caching."
---

# 🔐 Secrets Management Skill

## Trigger
- "Secret hardcoded", "remove secret from frontend", "TURN credential leak"
- Security audit finds secrets in JS bundle
- Any API key/token/credential in frontend/src/

## The Problem
Frontend JavaScript is bundled and served to clients. Any secret hardcoded in frontend code (TURN_SECRET, API keys, passwords) is extractable by anyone who loads the page.

## The Solution Pattern
Move secret to backend, create API endpoint that generates short-lived credentials.

### Backend: Create credential endpoint

```rust
// backend/src/webrtc.rs
async fn handle_ice_config(
    AxumState(state): AxumState<Arc<crate::SharedState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Auth check — reuse existing cookie validation pattern
    let cookie_header = match headers.get(COOKIE).and_then(|v| v.to_str().ok()) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, AxumJson(json!({"error": "Non authentifié"}))).into_response(),
    };
    // ... validate cookie, check DB ...

    // Generate short-lived credentials
    let username = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + (24 * 3600);
    
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    type HmacSha1 = Hmac<Sha1>;
    
    let mut mac = <HmacSha1 as hmac::Mac>::new_from_slice(config.turn_secret.as_bytes()).unwrap();
    mac.update(username.to_string().as_bytes());
    let credential = base64ct::Base64Unpadded::encode_string(&mac.finalize().into_bytes());

    (StatusCode::OK, AxumJson(json!({
        "host": config.turn_host,
        "port": config.turn_port,
        "username": username.to_string(),
        "credential": credential,
    }))).into_response()
}
```

**Dependencies** (Cargo.toml):
```toml
hmac = "0.12"
sha1 = "0.10"
```

**Config** (config.rs):
```rust
pub turn_host: String,   // from TURN_HOST env var
pub turn_port: u16,      // from TURN_PORT env var (default 3478)
pub turn_secret: String, // from TURN_SECRET env var
```

### Frontend: Fetch credentials from API

```typescript
// frontend/src/lib/webrtc-calls.svelte.ts
interface IceConfig {
  host: string;
  port: number;
  username: string;
  credential: string;
}

let cachedIceConfig: IceConfig | null = null;
let iceConfigFetchTime = 0;
const ICE_CONFIG_TTL = 20 * 3600 * 1000; // 20 hours (config valid 24h)

async function fetchIceConfig(): Promise<IceConfig> {
  if (cachedIceConfig && (Date.now() - iceConfigFetchTime) < ICE_CONFIG_TTL) {
    return cachedIceConfig;
  }
  const resp = await fetch('/api/webrtc/ice-config', { credentials: 'include' });
  if (!resp.ok) throw new Error(`Erreur ICE config: ${resp.status}`);
  cachedIceConfig = await resp.json();
  iceConfigFetchTime = Date.now();
  return cachedIceConfig!;
}
```

### Remove the old hardcoded code

```typescript
// ❌ DELETE THIS
const TURN_SECRET = 'change...cret'; // exposed in bundle!
const TURN_HOST = '192.168.1.100';

// ✅ USE THIS
const iceConfig = await fetchIceConfig();
const iceServers: RTCIceServer[] = [
  { urls: `stun:${iceConfig.host}:${iceConfig.port}` },
  { urls: `turn:${iceConfig.host}:${iceConfig.port}?transport=udp`, 
    username: iceConfig.username, credential: iceConfig.credential },
];
```

## Common Pitfalls

1. **HMAC ambiguous `new_from_slice`**: Use `<HmacSha1 as hmac::Mac>::new_from_slice(...)` not `HmacSha1::new_from_slice(...)`
2. **`state.pool` vs `state.db`**: Nook uses `state.db` for the SqlitePool
3. **Auth pattern**: Reuse the cookie validation from `verify_ws_auth`, don't create new auth functions
4. **Cache TTL**: Credentials valid 24h, cache 20h to avoid edge cases
5. **CORS**: Endpoint must be on same origin or CORS must allow it

## Verification
```bash
# Build frontend and check bundle
cd frontend && npm run build
grep -r "TURN_SECRET\|change.*cret\|192\.168" build/  # Should return nothing
```
