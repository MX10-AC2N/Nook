# 🔒 Rapport de Sécurité Nook — Branche develop

**Date**: 2026-05-03  
**Auditeur**: Hermes Agent (Security Auditor Skill)  
**Scope**: Audit complet (OWASP Top 10, Secrets, E2EE, Auth, WebRTC, CORS, Headers)  
**Branche**: develop  
**Repository**: https://github.com/MX10-AC2N/Nook  
**Dernier rapport**: 2026-04-28 (Score: 78/100)

---

## 📊 Résumé Exécutif

Ce rapport présente l'audit de sécurité complet du projet Nook, focalisé sur les vulnérabilités OWASP Top 10, la gestion des secrets, le chiffrement de bout en bout (E2EE), l'authentification, la sécurité WebRTC, CORS et les headers de sécurité.

**Score de sécurité global : 82/100** (+4 points depuis le 2026-04-28)

### Progression
| Date | Score | Évolution |
|------|-------|-----------|
| 2026-04-09 | 79/100 | — |
| 2026-04-28 | 78/100 | -1 (nouvelles vulnérabilités dépendances) |
| 2026-05-03 | 82/100 | +4 (HSTS ajouté, rate limiting auth, corrections) |

---

## 🎯 Audit OWASP Top 10 (2021)

### A01: Broken Access Control — Score: 88/100 ⬆️
**Statut**: Bien implémenté

**Points forts**:
- Vérification d'autorisation via `user.id` et `user.role == "admin"`
- Middleware `require_auth` et `require_admin` bien implémentés (auth.rs:368-433)
- Vérification d'appartenance aux conversations avant accès aux messages
- Protection des routes admin via `require_admin` (main.rs:440)

**Points à améliorer**:
- Pas de middleware d'autorisation centralisé (vérifications dispersées)
- Ajouter des tests d'intégration pour les tentatives de contournement

**Recommandations**:
- Implémenter un middleware d'autorisation centralisé
- Ajouter des tests pour les contrôles d'accès

---

### A02: Cryptographic Failures — Score: 92/100 ⬆️
**Statut**: Excellente implémentation

**Points forts**:
- **Argon2** pour le hachage des mots de passe (auth.rs:66-73) ✓
- **XChaCha20-Poly1305** pour le chiffrement des fichiers (webrtc.rs:57-68) ✓
- **X25519** pour E2EE (e2ee.rs utilise des clés publiques X25519) ✓
- Génération de tokens avec `Uuid::new_v4()` (entropie suffisante) ✓
- Variables d'environnement pour les secrets (TURN_SECRET, VAPID keys) ✓

**Points à améliorer**:
- Rotation des clés de chiffrement non implémentée
- E2EE partiellement activé (frontend commenté)

**Recommandations**:
- Implémenter la rotation des clés pour les fichiers chiffrés
- Finaliser l'activation E2EE (résoudre le problème de chargement libsodium)

---

### A03: Injection — Score: 98/100
**Statut**: Excellent (pas de faille trouvée)

**Points forts**:
- Toutes les requêtes SQL utilisent des requêtes paramétrées avec `sqlx::query_as!`
- Pas de concaténation de chaînes dans les requêtes SQL
- Validation des entrées sur les endpoints API
- Pas d'exécution de commandes système avec entrée utilisateur

**Recommandations**:
- Aucune (implémentation exemplaire)

---

### A04: Insecure Design — Score: 82/100 ⬆️
**Statut**: Amélioration significative

**Points forts**:
- **Rate limiting par IP** implémenté (main.rs:362-370)
- **Rate limiting spécifique auth** : 5 tentatives/min par IP (main.rs:367-390) ✓ NOUVEAU
- CORS correctement configuré avec origines explicites
- Validation des entrées côté serveur (ex: longueur mot de passe ≥ 8)

**Points à améliorer**:
- Pas de verrouillage de compte après échecs répétés (par username)
- Limite de taille des uploads configurée mais dépend d'Axum (52MB)

**Recommandations**:
- Ajouter un verrouillage de compte après 5-10 tentatives échouées
- Implémenter un système de "account lockout"

---

### A05: Security Misconfiguration — Score: 88/100 ⬆️
**Statut**: Bien configuré

**Points forts**:
- **HSTS ajouté** : `Strict-Transport-Security: max-age=31536000; includeSubDomains` ✓ NOUVEAU
- X-Frame-Options: DENY ✓
- X-Content-Type-Options: nosniff ✓
- X-XSS-Protection: 1; mode=block ✓
- Content-Security-Policy restrictif avec `'unsafe-inline'` nécessaire pour Svelte ✓
- Referrer-Policy: strict-origin-when-cross-origin ✓
- Permissions-Policy: camera=(self), microphone=(self) ✓
- CORS credentials géré correctement (pas de wildcard avec credentials)

**Points à améliorer**:
- CSP utilise `'unsafe-inline'` (limitation SvelteKit)

**Recommandations**:
- Envisager nonce-based CSP si SvelteKit le supporte
- Audit régulier des configurations

---

### A06: Vulnerable and Outdated Components — Score: 65/100 ⬇️
**Statut**: Vulnérabilités trouvées (action requise)

### Frontend (npm audit):
| Sévérité | Count | Package | Advisory | CVSS |
|----------|-------|---------|----------|------|
| MODERATE | 4 | dompurify | ADD_TAGS bypass, FORBID_TAGS bypass, SAFE_FOR_TEMPLATES bypass, Prototype Pollution to XSS | 6.8-6.9 |
| MODERATE | 1 | uuid | Missing buffer bounds check in v3/v5/v6 when buf is provided | - |
| MODERATE | 1 | yaml | Stack Overflow via deeply nested YAML collections | 4.3 |

**Action immédiate**:
```bash
cd /opt/data/home/.hermes/Nook/frontend
npm update dompurify  # >= 3.4.0 required
npm update uuid      # >= 14.0.0 required
npm update yaml      # >= 2.8.3 required
```

### Backend (cargo audit):
- `cargo audit` non disponible (requiert Rust 1.86+)
- Dépendances principales vérifiées manuellement :
  - axum 0.8 - Récent, pas de CVE critiques connues
  - sqlx 0.8.6 - Récent
  - argon2 0.5 - Stable
  - ring 0.17 - Cryptographiquement sûr

**Recommandations**:
- Mettre à jour Rust vers 1.86+ et exécuter `cargo audit` dans la CI
- Corriger les vulnérabilités frontend immédiatement

---

### A07: Identification and Authentication Failures — Score: 87/100 ⬆️
**Statut**: Bien implémenté

**Points forts**:
- **Argon2** avec salt unique pour chaque mot de passe ✓
- **HttpOnly, SameSite cookies** (Lax en HTTP, None+Secure en HTTPS) ✓
- **Rate limiting auth** : 5 tentatives/min par IP ✓ NOUVEAU
- Force du mot de passe : minimum 8 caractères ✓
- `needs_password_change` pour les comptes créés par invitation ✓
- Détection HTTPS via `X-Forwarded-Proto` (Nginx) ✓

**Points à améliorer**:
- Pas de MFA (Multi-Factor Authentication)
- Pas de mécanisme de réinitialisation de mot de passe (seulement par invitation)
- Pas de verrouillage de compte par username

**Recommandations**:
- Ajouter TOTP pour les comptes admin
- Implémenter la réinitialisation de mot de passe par email
- Suivi des tentatives par username (pas seulement par IP)

---

### A08: Software and Data Integrity Failures — Score: 82/100
**Statut**: Bonnes pratiques

**Points forts**:
- Validation des magic bytes pour les uploads (upload.rs)
- Prévention du spoofing de type de fichier
- Validation des entrées API
- Pas de sous-ressources externes non sécurisées

**Points à améliorer**:
- Pas de Subresource Integrity (SRI) pour les ressources CDN
- Cookies de session non signés (bien que HttpOnly fournisse une protection)

**Recommandations**:
- Ajouter SRI si des ressources CDN sont utilisées
- Envisager la signature HMAC des cookies de session

---

### A09: Security Logging and Monitoring Failures — Score: 78/100 ⬆️
**Statut**: Amélioration

**Points forts**:
- Utilisation de `tracing` pour le logging structuré
- Événements de sécurité loggés (auth, actions admin)
- Rate limiting avec avertissements de sécurité

**Points à améliorer**:
- Pas de table d'audit log centralisée
- Pas d'alerting sur les patterns suspects
- Logs d'erreur ne fuitent pas d'infos sensibles

**Recommandations**:
- Créer une table `audit_logs` pour les actions critiques
- Ajouter des alertes pour les tentatives de force brute
- Intégrer avec une solution SIEM/monitoring

---

### A10: Server-Side Request Forgery (SSRF) — Score: 92/100
**Statut**: Risque faible

**Points forts**:
- Requêtes Giphy API utilisent l'URL configurée
- Pas d'URL fournie par l'utilisateur fetchée par le serveur
- Uploads de fichiers n'impliquent pas de fetch d'URL
- WebRTC utilise des serveurs TURN configurés

**Recommandations**:
- Maintenir la validation des URLs pour les futures fonctionnalités

---

## 🔐 Gestion des Secrets (Hardcoded Secrets)

**Score: 90/100** ⬇️ (baisse due aux passwords dans les tests)

### Points forts:
- ✅ Aucun secret en dur dans le code de production
- ✅ `.env.example` contient des placeholders (pas de vrais secrets)
- ✅ Variables d'environnement pour TURN_SECRET, VAPID keys, ADMIN_INITIAL_PASSWORD
- ✅ Génération aléatoire du mot de passe admin si non défini
- ✅ Fichiers `.env`, `*.key`, `*.pem` dans `.gitignore`

### Points à corriger:
- ⚠️ **Mots de passe en dur dans les tests E2E** (frontend/tests/*.spec.ts):
  - 'Hermes2026!' utilisé dans plusieurs tests
  - 'AdminCI2026!' dans admin-ui.spec.ts
  - 'E2eTest123!' dans api-sanity.spec.ts
- ⚠️ **MediaRecorder.svelte** utilise `prompt()` pour le mot de passe (ligne 133, 215)
  - Problème : le mot de passe est visible à l'écran
  - Problème : pas de stockage sécurisé (devrait utiliser cryptoStore)

### Recommandations:
1. **Immédiat**: Utiliser des variables d'environnement pour les tests E2E
   ```bash
   # Dans les tests Playwright
   const password = process.env.E2E_USER_PASSWORD || 'Hermes2026!';
   ```
2. **Correction MediaRecorder.svelte**:
   - Utiliser `cryptoStore` pour récupérer/récupérer le mot de passe
   - Ne pas utiliser `prompt()` en production
3. Ajouter un hook pre-commit avec `gitleaks` ou `trufflehog`

---

## 🔒 E2EE (End-to-End Encryption)

**Statut**: Partiellement implémenté — Score: 75/100

### Backend (e2ee.rs):
- ✅ Endpoints d'échange de clés publiques X25519 (POST /api/auth/public-key)
- ✅ Récupération des clés publiques par conversation (GET /api/auth/public-keys)
- ✅ Stockage des clés chiffrées par message (message_keys table)
- ✅ Validation de la taille des clés (32 bytes)
- ✅ Vérification d'appartenance à la conversation

### Frontend (e2ee.ts):
- ✅ Classe E2EE avec génération de paires de clés
- ✅ Stockage des clés privées dans IndexedDB (pas localStorage) ✓
- ✅ Chiffrement/déchiffrement XChaCha20-Poly1305
- ⚠️ Implémentation dans `lib/e2ee.ts` mais pas activée partout
- ⚠️ `MediaRecorder.svelte` utilise encore un prompt de mot de passe au lieu d'E2EE

### Point critique:
- 🔴 **TODO dans MediaRecorder.svelte** (lignes 133, 215, 382):
  ```typescript
  // TODO: récupérer les vraies clés publiques
  // TODO: implémenter avec cryptoStore.decryptMessage()
  ```

### Recommandations:
1. Activer E2EE dans tous les composants de messagerie
2. Remplacer le `prompt()` password par E2EE dans MediaRecorder.svelte
3. Implémenter le chargement dynamique de libsodium (éviter le blocage LCP)
4. Tests E2EE dans Playwright (E2E_SETUP=1)

---

## 🎥 WebRTC Security

**Score: 88/100**

### Points forts:
- ✅ **Authentification WebSocket** : cookie auth_token vérifié dès la connexion WS (webrtc.rs:234-262)
- ✅ **Limite de taille des messages WS** : 64 KB max (webrtc.rs:419)
- ✅ **Gestion des erreurs** : pas de crash sur message invalide
- ✅ **TURN/STUN configurés** via variables d'environnement (config.rs:55-70)
- ✅ **Routage sécurisé** des signaux WebRTC par user_id
- ✅ **Nettoyage** des connexions WS à la déconnexion

### Configuration TURN:
```yaml
# docker-compose.yml
environment:
  - TURN_SECRET=${TURN_SECRET:?must be set}
  - TURN_PORT=${TURN_PORT:-3478}
```

### Points à améliorer:
- WebRTC routes dans `webrtc_routes()` sont publiques (main.rs:559) — devraient être dans `protected_routes`
- Pas de limite sur le nombre de connexions WebSocket par utilisateur

### Recommandations:
1. Déplacer les routes `/api/webrtc/*` dans `protected_routes`
2. Ajouter une limite de connexions WS par utilisateur
3. Chiffrement des messages de signaling (mentionné dans security-crypto.md mais pas vérifié)

---

## 🌐 CORS (Cross-Origin Resource Sharing)

**Score: 95/100**

### Configuration (main.rs:529-544):
```rust
let cors_layer = CorsLayer::new()
    .allow_origin(allowed_origins)  // Origines explicites depuis env
    .allow_methods([GET, POST, PUT, DELETE, OPTIONS])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT, COOKIE])
    .allow_credentials(true);
```

### Points forts:
- ✅ Pas de `allow_any_origin()` avec `allow_credentials(true)` (panique Rust)
- ✅ Origines explicitement listées via `ALLOWED_ORIGINS` et `PUBLIC_SITE_URL`
- ✅ localhost uniquement en mode debug (config.rs:37-41)
- ✅ Headers autorisés limités au nécessaire

### Recommandations:
- Aucune (configuration exemplaire)

---

## 🛡️ Security Headers

**Score: 95/100** ⬆️ (HSTS ajouté)

### Headers configurés (main.rs:564-576):
| Header | Valeur | Statut |
|--------|--------|--------|
| X-Frame-Options | DENY | ✓ |
| X-Content-Type-Options | nosniff | ✓ |
| X-XSS-Protection | 1; mode=block | ✓ |
| Referrer-Policy | strict-origin-when-cross-origin | ✓ |
| Permissions-Policy | camera=(self), microphone=(self), geolocation=(), payment=() | ✓ |
| Content-Security-Policy | default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; ... | ✓ |
| **Strict-Transport-Security** | **max-age=31536000; includeSubDomains** | **✓ NOUVEAU** |

### Points forts:
- ✅ HSTS maintenant configuré (protection contre downgrade attacks)
- ✅ CSP restrictif (limitations SvelteKit acceptées)
- ✅ frame-ancestors 'none' dans CSP (protection clickjacking)

### Recommandations:
- Monitorer les violations CSP dans la console navigateur
- Envisager nonce-based CSP pour éliminer 'unsafe-inline'

---

## 📋 Résumé des Scores par Catégorie

| Catégorie | Score | Max | Évolution |
|-----------|-------|-----|-----------|
| A01: Broken Access Control | 88 | 100 | +3 |
| A02: Cryptographic Failures | 92 | 100 | +2 |
| A03: Injection (SQL) | 98 | 100 | +3 |
| A04: Insecure Design | 82 | 100 | +7 |
| A05: Security Misconfiguration | 88 | 100 | +8 |
| A06: Vulnerable Components | 65 | 100 | -5 |
| A07: Authentication Failures | 87 | 100 | +2 |
| A08: Integrity Failures | 82 | 100 | +2 |
| A09: Logging Failures | 78 | 100 | +3 |
| A10: SSRF | 92 | 100 | +2 |
| Hardcoded Secrets | 90 | 100 | -5 |
| XSS Protection | 88 | 100 | - |
| CSRF Protection | 90 | 100 | - |
| Path Traversal | 95 | 100 | - |
| Rate Limiting | 88 | 100 | +3 |
| E2EE Implementation | 75 | 100 | Nouveau |
| WebRTC Security | 88 | 100 | Nouveau |
| CORS | 95 | 100 | Nouveau |
| Security Headers | 95 | 100 | +15 (HSTS) |
| **OVERALL SCORE** | **82** | **100** | **+4** |

---

## 🚨 Vulnérabilités par Sévérité

### CRITICAL (0 issues)
Aucune vulnérabilité critique trouvée.

### HIGH (0 issues)
Aucune vulnérabilité haute trouvée.

### MODERATE (6 issues)
1. **dompurify < 3.4.0** — XSS via plusieurs vecteurs (frontend)
   - CVSS: 6.8-6.9
   - Fix: `npm update dompurify`

2. **uuid < 14.0.0** — Buffer overflow potentiel (frontend)
   - Fix: `npm update uuid`

3. **yaml < 2.8.3** — Stack Overflow (frontend)
   - CVSS: 4.3
   - Fix: `npm update yaml`

4. **Mots de passe en dur dans tests E2E** (frontend/tests/*.spec.ts)
   - Fix: Utiliser variables d'environnement

5. **MediaRecorder.svelte password prompt** (frontend)
   - Fix: Utiliser cryptoStore / E2EE

6. **Rate limiting auth par username manquant**
   - Fix: Implémenter un suivi par username

### LOW (3 issues)
1. **Pas de MFA pour admin**
   - Fix: Ajouter TOTP

2. **Pas de account lockout**
   - Fix: Verrouiller après 5-10 échecs

3. **Cargo audit non exécutable** (Rust 1.85.0 < 1.86+ requis)
   - Fix: Upgrader Rust en CI

---

## ✅ Actions Prioritaires

### Immédiat (1 semaine):
1. ⚠️ **Mettre à jour les dépendances frontend vulnérables** :
   ```bash
   cd /opt/data/home/.hermes/Nook/frontend
   npm update dompurify uuid yaml
   ```

2. ⚠️ **Corriger les mots de passe en dur dans les tests E2E**:
   - Remplacer par des variables d'environnement

3. ⚠️ **Corriger MediaRecorder.svelte**:
   - Remplacer `prompt()` par E2EE/cryptoStore

### Court terme (1 mois):
4. Déplacer les routes WebRTC dans `protected_routes`
5. Ajouter un verrouillage de compte (account lockout)
6. Upgrader Rust vers 1.86+ et exécuter `cargo audit`
7. Activer E2EE dans tous les composants

### Long terme (3 mois):
8. Implémenter une table `audit_logs`
9. Ajouter MFA (TOTP) pour les admins
10. Implémenter la réinitialisation de mot de passe
11. Ajouter alerting sur patterns suspects

---

## 📊 Conformité

| Standard | Conformité | Notes |
|----------|-----------|-------|
| OWASP Top 10 2021 | 82% | Cible: 85%+ |
| GDPR | Non évalué | Focus technique dans cet audit |
| SOC 2 | ~75% | Logs d'audit manquants |

---

## 🛠️ Outils Utilisés

- ✅ **npm audit** (frontend) — 3 vulnérabilités modérées trouvées
- ❌ **cargo audit** (backend) — Non exécutable (Rust 1.85.0 < 1.86+)
- ✅ **Analyse statique** (grep, search_files) — secrets, patterns
- ✅ **Review manuel** — code Rust, TypeScript, Svelte, YAML
- ✅ **Vérification configurations** — CORS, Headers, WebRTC, E2EE

---

## 📝 Conclusion

Le projet Nook démontre d'excellentes pratiques de sécurité avec une amélioration significative depuis le dernier audit (+4 points). L'ajout du rate limiting spécifique pour l'authentification et du header HSTS renforce considérablement la posture de sécurité.

**Points forts**:
- Protection excellente contre l'injection SQL (sqlx macros)
- Chiffrement fort (Argon2, XChaCha20-Poly1305, X25519)
- Gestion des secrets exemplaire (aucun en dur en production)
- CORS et Headers de sécurité bien configurés

**Points d'attention**:
- Vulnérabilités dans les dépendances frontend (action immédiate requise)
- E2EE partiellement implémenté (à finaliser)
- Quelques mots de passe en dur dans les tests

**Temps estimé pour atteindre 85/100**: 2-3 jours développeur

---

*Rapport généré par Hermes Agent — Security Auditor Skill*  
*Basé sur le skill nook-security-auditor et les règles dans .hermes/rules/*
