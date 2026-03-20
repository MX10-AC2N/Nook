# 🛡️ Rôle : Auditeur Sécurité — Nook

> Analyse statique à chaque session. Vérifie : auth, XSS, injections, upload, WS, crypto, secrets.
> Activer ce rôle AVANT tout commit touchant auth.rs, upload.rs, webrtc.rs, e2ee.rs ou app.html.

---

## 🎯 Périmètre

```
Backend :
├── auth.rs       → tokens, cookies, Argon2, timing attacks
├── upload.rs     → path traversal, magic bytes, taille, content-type
├── webrtc.rs     → WS auth, taille messages, injection signaling
├── e2ee.rs       → gestion clés publiques, pas de clé privée côté serveur
├── main.rs       → CORS, rate limiting, routes publiques vs protégées
└── emergency.rs  → endpoint sensible — doit être protégé si activé

Frontend :
├── app.html           → CSP, meta tags sécurité
├── chat/+page.svelte  → {@html} → DOMPurify obligatoire
├── lib/api.ts         → credentials:include sur tous les fetch
└── lib/cryptoStore.ts → clés en IndexedDB (jamais localStorage)
```

---

## 🔍 Checklist d'audit — À exécuter à chaque session

### Auth & Sessions
```
□ Tokens générés avec OsRng (pas Math.random, pas thread_rng)
□ Entropie ≥ 122 bits (UUID v4 minimum) ou 256 bits (32 bytes OsRng) recommandé
□ Token stocké en DB, NULL au logout → révocable immédiatement
□ Cookie : HttpOnly ✓ | SameSite adaptatif ✓ | Secure en HTTPS ✓
□ Argon2id avec sel unique par mot de passe (SaltString::generate(&mut OsRng))
□ needs_password_change=1 → redirect /change-password (vérifié dans layout)
□ approved=1 vérifié dans require_auth avant tout accès
□ Pas de token dans les logs (tracing::info, println!)
```

### XSS & Injection Frontend
```
□ {@html} UNIQUEMENT avec DOMPurify.sanitize() — JAMAIS brut
□ Pas de innerHTML= dans le TS/JS
□ CSP dans app.html : default-src 'self' (bloque inline scripts)
□ Liens externes : rel="noopener noreferrer"
□ Content-Security-Policy couvre : script-src, img-src, connect-src
□ Pas d'eval(), no Function(string), pas de setTimeout(string)
```

### Upload & Fichiers
```
□ Taille max vérifiée CÔTÉ SERVEUR (pas seulement client)
□ Nom de fichier sanitizé → uuid pour le nom de stockage (✓ déjà le cas)
□ Extension extraite du nom original → utilisée seulement pour le Content-Type
□ Fichier stocké HORS du répertoire static/ (pas accessible directement)
□ Tous les fichiers chiffrés au repos (XChaCha20-Poly1305)
□ Content-Type validé par magic bytes (à implémenter — SEC-04)
□ TTL 48h respecté par prune.rs + cleanup.rs
□ /api/download/{id} vérifie require_auth avant de servir
```

### WebSocket
```
□ verify_ws_auth() appelé immédiatement à la connexion → sinon reject
□ Taille max des messages WS : 64 KB (à implémenter — SEC-05)
□ Messages parsés avec serde strict (pas d'eval, pas de format string)
□ Pas de broadcast à tous les utilisateurs depuis un message non authentifié
```

### CORS & Headers
```
□ ALLOWED_ORIGINS depuis .env (pas hardcodé, pas de wildcard)
□ allow_credentials(true) → origins JAMAIS wildcard (panic runtime sinon)
□ X-Forwarded-Proto détecté → SameSite=None;Secure activé côté HTTPS
□ OPTIONS preflight géré par CorsLayer d'Axum (automatique)
```

### Rate Limiting
```
□ Routes /auth/login, /auth/register, /join → rate limité
□ Quota actuel : 10 req/60s global (NotKeyed) → suffisant en familial
□ À terme : rate limit par IP (KeyedRateLimiter<IpAddr>) — SEC-02
□ E2E tests : describe.serial sur Rate Limiting pour éviter pollution
```

### Secrets & Configuration
```
□ Jamais de secret en dur dans le code (VAPID_KEY, SMTP_PASSWORD, etc.)
□ .env dans .gitignore ✓
□ E2E_SETUP=0 en production (jamais en prod)
□ RUST_LOG=info en prod (pas debug → trop verbeux)
□ RUST_BACKTRACE=0 en prod (évite fuites de structure interne)
```

### Module Emergency
```
□ emergency.rs non connecté dans main.rs → inactif (✓ S35)
□ Si activation future : require_auth + rate limit 1 req/10min par user
□ Destination (email/SMS) configurée via .env (pas hardcodée)
```

---

## 🚨 Vulnérabilités actives — Index

| ID | Sévérité | Statut | Fichier | Description |
|---|---|---|---|---|
| SEC-01 | 🔴 Critique | **OUVERT** | `chat/+page.svelte` | `{@html}` sans DOMPurify → XSS |
| SEC-01b | 🔴 Critique | **OUVERT** | `app.html` | Pas de Content-Security-Policy |
| SEC-02 | 🟡 Moyenne | OUVERT | `main.rs` | Rate limit global (NotKeyed) → pas par IP |
| SEC-03 | 🟢 Faible | OUVERT | `auth.rs` | UUID v4 token → 122 bits (ok, mais 256 recommandé) |
| SEC-04 | 🟡 Moyenne | OUVERT | `upload.rs` | Pas de validation magic bytes |
| SEC-05 | 🟡 Moyenne | OUVERT | `webrtc.rs` | Pas de limite taille messages WS |
| SEC-06 | ⚪ Info | OUVERT | `emergency.rs` | Module non exposé mais à protéger si activé |

---

## 🔧 Fixes prêts à implémenter

### Fix SEC-01 — DOMPurify (priorité absolue)

```typescript
// frontend/src/lib/sanitize.ts — NOUVEAU FICHIER
import DOMPurify from 'dompurify';

export function sanitizeHtml(html: string): string {
  if (typeof window === 'undefined') return html; // SSR safety
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'a', 'br', 'img', 'span', 'p'],
    ALLOWED_ATTR: ['href', 'src', 'alt', 'class', 'loading', 'target', 'rel'],
    ALLOW_DATA_ATTR: false,
    ADD_ATTR: ['target'],     // pour les liens externes
    FORCE_BODY: false,
    WHOLE_DOCUMENT: false,
  });
}
```

```bash
npm install dompurify
npm install --save-dev @types/dompurify
```

```svelte
<!-- chat/+page.svelte ligne 570 -->
<script>
  import { sanitizeHtml } from '$lib/sanitize';
</script>
<!-- AVANT -->
<div class="message-content">{@html msg.content}</div>
<!-- APRÈS -->
<div class="message-content">{@html sanitizeHtml(msg.content)}</div>
```

### Fix SEC-01b — CSP

```html
<!-- app.html — dans <head> -->
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' https://media.tenor.com https://c.tenor.com data: blob:;
  connect-src 'self' wss: ws:;
  font-src 'self';
  media-src 'self' blob:;
  worker-src 'self';
  object-src 'none';
  base-uri 'self';
">
```

> ⚠️ `'unsafe-inline'` sur style-src nécessaire pour les styles Svelte scoped.
> Les `<script>` inline sont bloqués → OK car SvelteKit compile vers des modules.

### Fix SEC-04 — Magic bytes upload

```rust
// backend/src/upload.rs — ajouter dans validate()
fn validate_magic_bytes(data: &[u8], content_type: &str) -> bool {
    if data.len() < 4 { return false; }
    let m = &data[..data.len().min(12)];
    match content_type {
        ct if ct.starts_with("image/jpeg") =>
            m.starts_with(&[0xFF, 0xD8, 0xFF]),
        ct if ct.starts_with("image/png") =>
            m.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        ct if ct.starts_with("image/gif") =>
            m.starts_with(b"GIF87a") || m.starts_with(b"GIF89a"),
        ct if ct.starts_with("image/webp") =>
            m.starts_with(b"RIFF") && m.get(8..12) == Some(b"WEBP"),
        ct if ct == "application/pdf" =>
            m.starts_with(b"%PDF"),
        _ => true, // permissif pour les autres types (documents, audio, vidéo)
    }
}
```

### Fix SEC-05 — WS message size

```rust
// backend/src/webrtc.rs — dans le handler WS
Ok(axum::extract::ws::Message::Text(text)) => {
    const MAX_WS_MSG: usize = 64 * 1024; // 64 KB
    if text.len() > MAX_WS_MSG {
        tracing::warn!(
            user_id = %current_user.id,
            size = text.len(),
            "Message WS surdimensionné — déconnexion"
        );
        break;
    }
    // ... reste du traitement
}
```

---

## 📚 Apprentissages

### [SEC-APP-01] `{@html}` = XSS garanti sans DOMPurify — Session 35
SvelteKit ne sanitise PAS `{@html}`. Toute valeur injectée depuis la DB ou le réseau
peut contenir du HTML malveillant. Les images uploadées (`<img src="...">`) et les GIFs
(`<img src="gif_url">`) sont légitimes — la sanitisation doit les laisser passer.
→ DOMPurify avec ALLOWED_TAGS incluant `img`, `a`, `span`.

### [SEC-APP-02] Cookie HttpOnly ≠ protection totale contre XSS
Même avec `HttpOnly`, un XSS peut :
- Faire des requêtes API authentifiées (le cookie est envoyé automatiquement)
- Exfiltrer les données visibles dans le DOM (messages, noms, fichiers)
- Rediriger vers un phishing
→ La CSP est la défense en profondeur indispensable.

### [SEC-APP-03] Rate limit NotKeyed — risque faible en prod familiale
Avec ~5 utilisateurs simultanés max, 10 req/60s global ne bloque pas les usages légitimes.
Le risque réel est: un test E2E agressif ou un bot externe peut épuiser le quota.
→ En attendant le fix par IP : les tests E2E utilisent `describe.serial` pour éviter ça.

### [SEC-APP-04] Fichiers chiffrés = protection suffisante contre upload malveillant
Même si un fichier `.html` malveillant est uploadé, il est :
1. Chiffré → illisible sans la clé
2. Servi via `/api/download/{id}` avec `Content-Disposition: attachment` pour les non-images
3. Jamais servi depuis `ServeDir` sans déchiffrement
→ Le risque résiduel est faible mais la validation magic bytes reste bonne pratique.
