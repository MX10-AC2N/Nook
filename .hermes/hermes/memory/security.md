# 🔐 Mémoire SECURITY - E2EE, Auth, WebRTC

> **DERNIÈRE MISE À JOUR** : 2026-05-04
> E2EE, Auth, WebRTC, Sécurité Nook

## 🔑 Authentification & JWT

### JsonWebToken (JWT)
- **Crate** : `jsonwebtoken` 9.3.1
- **Algorithme** : HS256 (par défaut)
- **Secret** : Configuré via variable d'environnement `JWT_SECRET`

### Structure Token
```rust
// Claims JWT typiques
struct Claims {
    sub: String,      // User ID
    exp: usize,       // Expiration
    iat: usize,       // Issued at
    // ... autres champs
}
```

### Vérification
- Middleware Axum pour vérifier le token
- Refresh token mechanism (à vérifier si implémenté)

## 🔐 Chiffrement E2EE (End-to-End Encryption)

### Argon2 (Password Hashing)
- **Crate** : `argon2` 0.5.3
- **Params** : 
  - Memory: 64MB (vérifié dans le code)
  - Iterations: 3
  - Parallelism: 4

### Stockage Mots de Passe
```rust
// Hash
let hash = argon2.hash_password(password.as_bytes(), &salt)?;

// Verify
argon2.verify_password(password.as_bytes(), &parsed_hash)?;
```

⚠️ **P0 - Vérifier** : Les paramètres Argon2 sont-ils correctement configurés ?

## 🌐 WebRTC & TURN

### TURN Server (turn-rs)
- **Image** : `ghcr.io/mx10-ac2n/turn-server:dev`
- **Ports** : 3478 UDP/TCP
- **Config** : `/etc/turn-server/config.toml`
- **Secret** : `${TURN_SECRET}` (variable d'environnement)

### Problème P0 - WebRTC ICE Config Manquante
- ❌ **Symptôme** : Les appels vidéo/audio ne se connectent pas
- ❌ **Cause probable** : Le fichier de config TURN n'est pas généré correctement
- ✅ **Solution** : Vérifier `turn-config/` directory et template

### Configuration TURN Attendue
```toml
[server]
name = "nook.turn"
secret = "TURN_SECRET_VALUE"
max-threads = 4

[[server.interfaces]]
transport = "udp"
listen = "0.0.0.0:3478"
external = "0.0.0.0:3478"

[[server.interfaces]]
transport = "tcp"
listen = "0.0.0.0:3478"
external = "0.0.0.0:3478"
```

## 🛡️ OWASP Top 10 - Status

### Vulnérabilités Checkées
- ✅ **A01-Broken Access Control** : Vérifié dans `SECURITY-REPORT-2026-05-03.md`
- ✅ **A02-Cryptographic Failures** : Argon2 + JWT ok
- ⚠️ **A03-Injection** : SQLx protège contre SQL injection
- ⚠️ **A04-Insecure Design** : Rate limiting manquant (P1)
- ⚠️ **A05-Security Misconfiguration** : CORS à vérifier
- ⚠️ **A06-Vulnerable Components** : `cargo audit` à ajouter en CI
- ⚠️ **A07-Identification & Auth Failures** : MFA manquant
- ⚠️ **A08-Software & Data Integrity** : CI/CD sécurisé
- ⚠️ **A09-Security Logging** : Logs insuffisants
- ⚠️ **A10-Server-Side Request Forgery** : À vérifier

## 🚫 CORS Configuration

### Axum CORS (À vérifier)
```rust
// ✅ Bon pattern - origins explicites
let cors = CorsLayer::new()
    .allow_origin([
        "https://192.168.1.192:6443".parse().unwrap(),
        "http://localhost:5173".parse().unwrap(),
    ])
    .allow_methods(Any)
    .allow_headers(Any);

// ❌ Mauvais pattern - Any avec credentials
// .allow_origin(Any) // DANGEREUX avec credentials
```

⚠️ **Règle** : Pas `Any` avec credentials activés !

## 🔒 HTTPS & Certificats

### Certificat Actuel
- **Type** : Auto-signé
- **URL** : `https://192.168.1.192:6443`
- **Problème** : Navigateur affiche warning (normal pour auto-signé)

### Recommandation
- [ ] Utiliser Let's Encrypt pour un vrai certificat
- [ ] Ou configurer un CA interne

## 📝 Audit Trail

### Rapports de Sécurité
- `SECURITY-REPORT.md` (ancien - 2026-04-28)
- `SECURITY-REPORT-2026-05-03.md` (récent - 68/100)

### Score Actuel
- **Global** : 68/100 (Audit 2026-05-03)
- **Backend** : À vérifier (events.rs a 34 erreurs compil)
- **Frontend** : PWA broken + HTTPS ok

## 🔴 Troubles de Sécurité à Fixer (P0/P1)

### P0 - Critique
1. **events.rs 34 erreurs compilation** → Fixer immédiatement
2. **WebRTC ICE config manquante** → Générer config TURN
3. **PGN export cassé** → Vérifier endpoint + frontend

### P1 - Important
1. **Pas de tests frontend** → Ajouter Jest/Vitest
2. **Pas cargo test en CI** → Ajouter step test
3. **106 E2E skippés** → Investiguer et activer
4. **ADR vides** → Documenter les décisions d'architecture
5. **<5% doc** → Améliorer documentation

### P2 - Mineur
1. **Bundle 939kB trop lourd** → Code splitting
2. **wasm-pack manquant** → Installer ou contourner
3. **SQLx prepare fail** → Fixer en CI
4. **Pas GitHub Releases** → Automatiser

## 📝 Notes de Session

- Security audit 68/100 (2026-05-03)
- 34 erreurs compilation events.rs (P0)
- TURN server config à vérifier (ICE config)
- CORS à audit correctement

---
*Mettre à jour après chaque audit ou fix sécurité*
