# 🔐 Rôle : Ingénieur Sécurité & Crypto — Nook

> Spécialiste E2EE, WebRTC, auth, crypto pour Nook.
> Activer ce rôle pour : chiffrement E2E, signaling WebRTC, gestion clés, audit sécurité.

---

## 🎯 Périmètre exclusif

```
Backend :
├── auth.rs          → Cookie HttpOnly, argon2 password hashing, tokens
├── webrtc.rs        → Signaling WebSocket + chiffrement XChaCha20-Poly1305
├── e2ee.rs          → Échange clés publiques, chiffrement messages
└── upload.rs        → Chiffrement fichiers uploadés (nonce + key_text en DB)

Frontend :
├── lib/crypto.ts          → XChaCha20-Poly1305, fonctions bas niveau
├── lib/e2ee.ts            → Échange clés publiques, protocole E2EE
├── lib/sodium.svelte.js   → libsodium-wrappers (938 kB WASM)
├── lib/cryptoStore.svelte.ts → État clés, chiffrement en cours
└── lib/webrtc.ts          → Signaling WebRTC, ice candidates, offer/answer
```

---

## 🔑 Auth — Architecture de sécurité

### Cookie HttpOnly

```
Set-Cookie: auth_token=<user_id>:<token>; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400
```

- `HttpOnly` → inaccessible depuis JavaScript → immunisé XSS
- `SameSite=Lax` → protège contre CSRF sur requêtes POST cross-site
- `Max-Age=86400` → expiration 24h côté browser
- Token stocké en DB → révocable immédiatement (logout = `UPDATE users SET token=NULL`)

### Adaptation WAN (Nginx Proxy Manager)

```
Nginx injecte : X-Forwarded-Proto: https
Backend détecte → Set-Cookie avec SameSite=None; Secure
Pourquoi : SameSite=Lax bloque les cookies cross-site en HTTPS
```

### Argon2 — Configuration recommandée

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;  // ✅ rand_core::OsRng (pas rand::rngs::OsRng)

// Hash
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

// Verify
let parsed_hash = PasswordHash::new(&stored_hash)?;
argon2.verify_password(password.as_bytes(), &parsed_hash)?;
```

---

## 🔒 Chiffrement fichiers — XChaCha20-Poly1305

```rust
use chacha20poly1305::{XChaCha20Poly1305, Key, XNonce};
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::aead::rand_core::RngCore;

// Chiffrement à l'upload
pub fn encrypt_file(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Error> {
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let cipher = XChaCha20Poly1305::new(&key);
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|_| Error::EncryptionFailed)?;

    Ok((ciphertext, nonce_bytes.to_vec(), key.to_vec()))
}
// key_text et nonce stockés en DB (table uploads)
// → permet déchiffrement côté client autorisé
```

---

## 🎥 WebRTC — Architecture signaling Nook

```
Architecture : Mesh P2P avec signaling via WebSocket Axum

Flux de connexion :
1. Client A → POST /api/webrtc/offer     → {sdp, target_user_id}
2. Backend → notifie Client B via WS
3. Client B → POST /api/webrtc/answer    → {sdp, target_user_id}
4. Backend → notifie Client A via WS
5. ICE candidates échangés via WS
6. Connexion P2P établie (STUN/TURN)

Chiffrement signaling :
- Les messages WS de signaling sont chiffrés XChaCha20-Poly1305
- Nonce unique par message
- Clé dérivée de la session utilisateur
```

### WebSocket handler — Pattern Nook

```rust
// webrtc.rs — structure du message WS
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum SignalingMessage {
    Offer   { sdp: String, target: String },
    Answer  { sdp: String, target: String },
    Ice     { candidate: String, target: String },
    Ping,
}

// Envoi depuis le backend vers un client spécifique
// (nécessite un registry des connexions WS actives)
```

---

## 🔐 E2EE — Protocole Nook

```
État actuel : PARTIELLEMENT IMPLÉMENTÉ
- Backend (e2ee.rs) : API d'échange de clés publiques
- Frontend (e2ee.ts, crypto.ts) : fonctions disponibles
- Activation : commentée dans certains composants (sodium non chargé)

Schéma prévu :
1. À la connexion : user génère paire de clés Ed25519 (libsodium)
2. Clé publique → POST /api/e2ee/keys (stockée en DB)
3. Avant d'envoyer un message à user B :
   - Récupérer clé publique de B : GET /api/e2ee/keys/{user_id}
   - Chiffrer avec X25519 (ECDH) + XChaCha20-Poly1305
4. Message chiffré posté (content = ciphertext base64, encrypted=1)
5. Recipient déchiffre avec sa clé privée (jamais transmise)
```

### libsodium — Initialisation obligatoire

```typescript
// sodium.svelte.js
import sodium from 'libsodium-wrappers';

export async function waitForSodium() {
  await sodium.ready;
  return sodium;
}

// ⚠️ 938 kB — charger en lazy pour ne pas bloquer le LCP
// TODO: dynamic import() avec loading screen dédié
// import('./sodium').then(m => m.waitForSodium())
```

---

## 🛡️ Audit sécurité — Points de contrôle

### Auth
- [ ] Tokens générés avec entropie suffisante (≥ 32 bytes random)
- [ ] Tokens révocables (NULL en DB au logout)
- [ ] `needs_password_change` géré (redirect /change-password)
- [ ] Rate limiting sur `/api/auth/login` (governor configuré)
- [ ] Pas de timing attack sur comparaison de tokens (constant-time comparison)

### CORS
- [ ] Pas de `allow_any_origin()` avec `allow_credentials(true)`
- [ ] Liste blanche ALLOWED_ORIGINS depuis env (pas hardcodée)
- [ ] OPTIONS preflight géré correctement

### Upload
- [ ] Taille maximale vérifiée côté serveur (pas seulement client)
- [ ] Content-type validé (pas confiance aveugle au header)
- [ ] Fichiers chiffrés au repos (XChaCha20-Poly1305)
- [ ] TTL 48h respecté (cleanup.rs)
- [ ] Noms de fichiers sanitizés (path traversal)

### WebSocket
- [ ] Auth vérifiée à la connexion WS (cookie présent)
- [ ] Messages parsés avec serde (pas d'eval)
- [ ] Taille maximale des messages limitée

### DB
- [ ] Requêtes paramétrées (sqlx macros → pas d'injection SQL possible)
- [ ] Pas de données sensibles loggées (passwords, tokens)
- [ ] `conversation_participants` vérifié avant accès messages

---

## 🚨 Vulnérabilités connues à surveiller

| Vecteur | Risque | Mitigation en place |
|---------|--------|---------------------|
| Cookie vol XSS | Accès non autorisé | HttpOnly ✅ |
| CSRF | Actions non désirées | SameSite=Lax ✅ |
| Brute force login | Compromission comptes | governor (rate limiting, à finaliser) |
| Path traversal upload | LFI/RFI | À vérifier dans upload.rs |
| WS injection | XSS via message | serde strict ✅ |
| Token entropy faible | Prédiction token | À auditer dans auth.rs |
| E2EE clés en localStorage | Vol clés si XSS | Tolérable (HttpOnly cookie) |

---

## 🤝 Flux inter-agents

```
← (aucune dépendance — intervient en Phase 1)
→ 🦀 RUST    : algorithmes, endpoints à protéger, champs DB à chiffrer, env vars secrets
→ 🎨 SVELTE  : fonctions libsodium, format clés, quoi chiffrer côté client
→ 🚀 DEVOPS  : secrets GitHub requis
→ 🧪 E2E     : comportements sécurité à tester (401, token révocation)
```

**Contrat de sécurité** à fournir en Phase 1 : données sensibles | algo chiffrement | auth requise | vecteurs identifiés.

---

## 📚 Apprentissages

> *Section mise à jour à chaque session.*

### [APP-CRYPTO-01] SameSite=Lax bloque WAN — Session 13
→ **Promu** dans Auth — Architecture de sécurité.

### [APP-CRYPTO-02] CORS allow_any_origin + credentials = PANIC runtime — Session 5
→ **Promu** dans section principale.

### [APP-CRYPTO-03] rand_core::OsRng vs rand::rngs::OsRng — Session 2
→ **Promu** dans la section principale.

### [APP-CRYPTO-04] E2EE partiellement implémenté — Sessions 1+

Backend `e2ee.rs` expose les endpoints d'échange de clés.
Frontend `e2ee.ts` + `crypto.ts` ont les fonctions disponibles.
Mais l'activation dans les composants est commentée (sodium chargement bloquant).
→ Ne pas activer E2EE avant résolution DT-01 (libsodium dynamic import).
Status : Dette technique DT-05. Bloquer sur DT-01 d'abord.

### [APP-CRYPTO-05] IndexedDB pour les clés privées

Les clés privées E2EE sont stockées dans IndexedDB (pas localStorage).
IndexedDB est accessible depuis le service worker et survit aux rechargements.
→ Ne jamais stocker les clés privées en localStorage (moins sécurisé).
Status : Pattern à maintenir.
