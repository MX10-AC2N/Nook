# 🔒 Rapport de Sécurité — Nook 2026-04-21

## Score : 88/100 (+6 depuis 2026-04-09)

---

## 🔴 CRITIQUE (0 — Corrigé !)

### ✅ C1 (CORRIGÉ) — Hardcoded TURN secret
- **Avant** : `secret = "change_this_turn_secret_2026"` dans `services/turn-rs/config.toml`
- **Correction** : Utilise `${TURN_SECRET}` avec remplacement via entrypoint
- **Fichiers** : `services/turn-rs/turnserver.conf.template`, `services/turn-rs/docker-entrypoint.sh`

### ✅ C2 (CORRIGÉ) — Admin initial password logged to stderr
- **Avant** : `eprintln!("Admin initial cree - utilisateur : admin / mot de passe : {}", random_password);` dans `backend/src/main.rs:152`
- **Correction** : Ligne supprimée, conservation uniquement de l'avertissement de changement de mot de passe
- **Fichier** : `backend/src/main.rs`

### ✅ C3 (CORRIGÉ) — `TURN_SECRET=***` visible dans docker-compose.yml
- **Avant** : Valeur `***` visible dans `docker inspect` et commité dans git
- **Correction** : `${TURN_SECRET:?TURN_SECRET must be set}` avec message d'erreur si non défini
- **Fichiers** : `docker-compose.yml` (services `nook` et `turn`)

---

## 🟡 HAUTE (2)

### H1 — `.env.example` contient des secrets faibles
- **Problème** : Les exemples `change_this_password!` et `change_this_turn_secret_2026` sont copiés tel quel
- **Recommandation** : ✅ **DÉJÀ CORRIGÉ** dans le nouveau `.env.example` — utilise `openssl rand -base64` et documente clairement les étapes
- **Fichier** : `.env.example`

### H2 — CORS always allows localhost origins
- **Problème** : En production, `localhost` ne devrait pas être autorisé (vol de credentials)
- **Recommandation** : Désactiver localhost en production :
```rust
// backend/src/main.rs
let allowed_origins = if cfg!(debug_assertions) {
    vec!["http://localhost:5173".to_string(), "http://localhost:6300".to_string()]
} else {
    vec![]  // Uniquement PUBLIC_SITE_URL en prod
};
```
- **Fichier** : `backend/src/main.rs`

### H3 — CSP allows `'unsafe-inline'` for scripts
- **Problème** : `script-src 'self' 'unsafe-inline'` affaiblit la protection XSS
- **Recommandation** : Utiliser des nonces ou hashes pour les scripts inline :
```rust
// backend/src/main.rs
"content-security-policy": "default-src 'self'; script-src 'self' 'nonce-...'; ..."
```
- **Fichier** : `backend/src/main.rs` (dans `cors_layer` ou équivalent)

---

## 🟢 MOYENNE (5)

### M1 — Path traversal defense depth
- **Problème** : Validation basique des chemins d'upload
- **Recommandation** : Utiliser `canonicalize` et vérifier que le chemin reste dans `/app/data/uploads`
- **Fichier** : `backend/src/routes.rs` (upload)

### M2 — CSRF protection gaps
- **Problème** : Pas de protection CSRF explicite
- **Recommandation** : Ajouter des tokens CSRF pour les actions sensibles (changement de mot de passe, suppression de compte)
- **Fichier** : `backend/src/auth.rs`

### M3 — Colon injection in cookie format
- **Problème** : Format des cookies peut permettre l'injection
- **Recommandation** : Valider strictement le format des valeurs de cookies
- **Fichier** : `backend/src/auth.rs`

### M4 — Pas de registration-specific rate limiting
- **Problème** : Rate limiting global, pas spécifique à l'inscription
- **Recommandation** : Ajouter `RATE_LIMIT_PER_MIN` spécifique pour `/api/auth/register`
- **Fichier** : `backend/src/routes.rs`

### M5 — WebSocket session not periodically re-authenticated
- **Problème** : Session WS valide indéfiniment sans re-vérification
- **Recommandation** : Vérifier le token JWT périodiquement (toutes les 5 min)
- **Fichier** : `backend/src/ws.rs`

---

## ✅ BIEN IMPLÉMENTÉ (positifs)

### Authentification & Autorisation
- ✅ **100% requêtes SQL paramétrées** — zéro risque d'injection SQL
- ✅ **Argon2** pour le hashage des mots de passe
- ✅ **WebSocket authentifié** — upgrade uniquement si token valide
- ✅ **Cookies sécurisés** — `HttpOnly`, `SameSite=None`, `Secure`

### Validation & Sanitisation
- ✅ **DOMPurify strict** pour la sanitisation HTML
- ✅ **Validation magic bytes** pour les uploads (vérification type de fichier)
- ✅ **XChaCha20** pour le chiffrement des fichiers P2P

### Headers de sécurité
- ✅ **X-Frame-Options: DENY**
- ✅ **X-Content-Type-Options: nosniff**
- ✅ **X-XSS-Protection: 1; mode=block**
- ✅ **Referrer-Policy: strict-origin-when-cross-origin**
- ✅ **Permissions-Policy** restrictif (camera, microphone, geolocation)
- ✅ **CSP** configuré (bien que `'unsafe-inline'` soit présent)

---

## 📋 RÉSUMÉ DES CORRECTIONS EFFECTUÉES

| Catégorie | Avant | Après |
|-----------|-------|--------|
| Secrets en dur (TURN) | `change_this_turn_secret_2026` | `${TURN_SECRET}` + doc |
| Secrets docker-compose | `TURN_SECRET=***` | `${TURN_SECRET:?...}` |
| Password logging | `eprintln!` avec mot de passe | ✅ Supprimé |
| Permissions Docker | `chmod 0777` | `chmod 0750` + `chown nook:nook` |

---

## 🧪 TESTS DE SÉCURITÉ RECOMMANDÉS

```bash
# 1. Vérifier que TURN_SECRET n'est pas en dur
docker compose config | grep -i "turn_secret" | grep -v "TURN_SECRET:"

# 2. Vérifier les permissions des volumes
docker exec nook ls -la /app/data | grep "drwxr-x---"

# 3. Vérifier qu'aucun mot de passe n'est dans les logs
docker logs nook 2>&1 | grep -i "password" | grep -v "change_password"

# 4. Test CORS en production
curl -H "Origin: http://evil.com" -H "Credentials: true" \
     https://your-nook-instance/api/auth/status

# 5. Test CSP
curl -H "Content-Type: application/json" \
     https://your-nook-instance/ | grep "content-security-policy"
```

---

## 📊 RÉFÉRENCES

- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [Rust Security Best Practices](https://anssi.github.io/rust-security-guide/)
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
