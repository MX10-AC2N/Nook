# 🔐 Mémoire Security - E2EE, Auth & WebRTC

> Dernière mise à jour: 2026-05-03
> Consulté lors de tout dev sécurité, auth, E2EE, WebRTC

## 🔑 Chiffrement & E2EE

### Argon2 (Password Hashing)
```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

// Hash password
let salt = rand::random::<[u8; 16]>();
let argon2 = Argon2::default();
let hash = argon2.hash_password(&password.as_bytes(), &salt).unwrap();

// Verify
let parsed_hash = PasswordHash::new(&stored_hash).unwrap();
argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
```

### XChaCha20-Poly1305 (E2EE Messages)
```rust
use chacha20poly1305::{XChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};

// Encrypt
let key = XChaCha20Poly1305::new(&Key::from_slice(&key_bytes));
let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());
let ciphertext = key.encrypt(&nonce, plaintext.as_bytes()).unwrap();

// Decrypt
let plaintext = key.decrypt(&nonce, ciphertext.as_ref()).unwrap();
```

## 🍪 Auth & Sessions

### Cookies (Axum 0.8)
```rust
use axum_extra::extract::cookie::{Cookie, CookieJar};

// Set cookie
let jar = CookieJar::new()
    .add(Cookie::new("session", session_id));

// Get cookie
let session = jar.get("session").map(|c| c.value());
```

### JWT (si utilisé)
- Clé secrète dans GitHub Secrets
- Expiration courte (15 min)
- Refresh token sécurisé

## 📡 WebRTC & TURN

### Configuration TURN Server
```rust
// rustrtc config
let config = RtcConfig {
    turn_server: "turn:nook.app:3478".to_string(),
    turn_username: "user".to_string(),
    turn_password: "pass".to_string(),
    // ...
};
```

### Sécurité WebRTC
- ✅ Toujours utiliser TURN (pas de P2P direct sans fallback)
- ✅ Clés E2EE pour médias
- ✅ Vérification origines (CORS strict)

## 🚫 CORS - Règles Strictes

```rust
// ❌ JAMAIS
CorsLayer::new()
    .allow_origin(Any)

// ✅ TOUJOURS
CorsLayer::new()
    .allow_origin([
        "https://192.168.1.192:6443".parse().unwrap(),
        "https://nook.app".parse().unwrap(),
    ].into_iter())
    .allow_credentials(true)
```

## 🔴 SECRETS - Règles Strictes

### GitHub Secrets (NE PAS commiter)
- ❌ `TURN_SECRET=abc123` dans code
- ❌ `.env` avec vrais secrets (ajouter à `.gitignore`)
- ✅ Utiliser GitHub Secrets + `.env.example`

### Permissions Fichiers
```bash
# ❌ JAMAIS
chmod 0777 /path/to/file

# ✅ TOUJOURS
chmod 0640 /path/to/sensitive-file
```

## 📝 Learnings Sessions

### Session 50
- ✅ 0 secret en dur (TURN_SECRET, admin password)
- ✅ chmod 0777 corrigé
- ✅ `.env.example` créé

### Erreurs fréquentes
1. **CORS Any avec credentials** → fail sécurité
2. **Secrets loggés** → fuite info
3. **Permissions trop ouvertes** → vulnérabilité

## 🔗 Ressources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Argon2 Crate](https://docs.rs/argon2/latest/argon2/)
- [ChaCha20Poly1305 Crate](https://docs.rs/chacha20poly1305/latest/)
- [WebRTC Security](https://webrtc-security.github.io/)

---
*Ajouter nouveaux apprentissages au fur et à mesure*
