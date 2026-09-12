# 🔎 Rapport d'Audit de Sécurité — Nook

> **Profil**: security-auditor
> **Date**: 2026-09-11
> **Branche**: develop
> **Scope**: E2EE, Crypto, Rate Limiting (DT-04), Dependencies, Threat Model

> **Pour Hermes**: Utiliser le skill `plan` pour l'exécution détaillée. Ce document est le plan du rapport d'audit.

---

## Objectif

Produire un rapport d'audit de sécurité exhaustif couvrant l'état actuel de l'implémentation E2EE, du rate limiting (DT-04), des vulnérabilités de dépendances, et du modèle de menace STRIDE du projet Nook.

## Contexte

Le projet Nook est une plateforme de messagerie privée auto-hébergée avec E2EE (X25519 + XChaCha20-Poly1305), WebRTC, et SQLite. L'audit précédent remonte au 2026-06-04. Depuis, plusieurs évolutions ont eu lieu:

- **DT-04 résolu partiellement**: `governor` 0.10 intégré avec `redis_rate_limiter.rs` (StateStore custom Redis) — mais `DT-04` reste marqué ouvert
- **E2EE**: architecture "clé de session par message" avec rotation X25519 (DT-05 toujours ouvert — old messages undecryptable après rotation)
- **Cargo audit**: 4 vulnérabilités trouvées (h2, quinn-proto, rsa, proc-macro-error2, anyhow, event-listener)
- **E2E**: 69 tests échoués sur les routes de sécurité auth (voir `.hermes/E2E-TARGETED-REPORT.md`)
- **libsodium-wrappers**: 938 kB WASM (DT-01 toujours ouvert)
- **69 échecs Playwright** sur les routes non-authentiquées → possible faille d'auth

## Plan d'Audit — 5 Domaines

### Domaine 1: E2EE & Cryptographie (Poids: 🔴 CRITICAL)

**Fichiers à auditer**:
- `backend/src/e2ee.rs` (2393 lignes)
- `backend/src/auth.rs` (Argon2id)
- `frontend/src/lib/sodium.svelte.js` (libsodium dynamic import)
- `frontend/src/lib/cryptoStore.svelte.ts`
- `frontend/src/lib/storage.ts`

**Points d'audit**:

1. **Crypto Hygiene**:
   - [ ] Vérifier `unwrap()`/`expect()` dans les chemins crypto (`e2ee.rs` contient ~49 `.unwrap()`)
   - [ ] Vérifier constant-time comparisons (`subtle::ConstantTimeEq`) pour comparaisons de tokens/mots de passe
   - [ ] Vérifier nonces XChaCha20 (192-bit) jamais réutilisés
   - [ ] Vérifier `zeroize` sur drop des clés
   - [ ] Vérifier `rand_core 0.6` (pas `rand::thread_rng`)
   - [ ] Vérifier Argon2id params: `m=65536, t=3, p=4`

2. **Key Rotation (DT-05)**:
   - [ ] Analyser le mécanisme de rotation X25519 (`rotate-key`, `key-history`)
   - [ ] Vérifier si les messages chiffrés avec l'ancienne clé restent déchiffrables
   - [ ] Évaluer l'impact de la perte des messages anciens après rotation
   - [ ] Vérifier le chiffrement de la clé privée dans `encrypted_private_key`

3. **Double Ratchet**:
   - [ ] Statut: partiel (🟡) — vérifier l'implémentation dans `e2ee.rs` + `e2ee.ts`
   - [ ] Vérifier la dérivation des clés de session par message

4. **Frontend Crypto**:
   - [ ] `sodium.svelte.js`: import dynamique (DT-01 — 938 kB WASM)
   - [ ] Vérifier que libsodium est initialisé avant usage
   - [ ] Vérifier la gestion des erreurs crypto côté client

### Domaine 2: Rate Limiting (DT-04) (Poids: 🟡 HIGH)

**Fichiers à auditer**:
- `backend/src/main.rs` (lignes 402-430)
- `backend/src/redis_rate_limiter.rs` (204 lignes)
- `backend/Cargo.toml` (governor = "0.10")

**Points d'audit**:

1. **Implémentation `governor`**:
   - [ ] Vérifier `IpRateLimiter::build(redis_url, rate_limit)` dans main.rs
   - [ ] Vérifier les limites: IP (60/min défaut), auth, e2ee endpoints
   - [ ] Vérifier le StateStore custom Redis (CAS Lua script)
   - [ ] Vérifier le fallback mémoire quand REDIS_URL non défini
   - [ ] Tester si le rate limiter protège réellement contre le brute-force

2. **Configuration Nginx**:
   - [ ] Vérifier `X-Forwarded-For` injecté par Nginx (pour extraction IP correcte)
   - [ ] Vérifier `X-Forwarded-Proto` pour le switch LAN/WAN

3. **Vérification fonctionnelle**:
   - [ ] Simuler une attaque par broute-force sur `/api/auth/login`
   - [ ] Vérifier que les limites par endpoint sont bien appliquées

### Domaine 3: Vulnérabilités Dépendances (Poids: 🟡 HIGH)

**Outils**: `cargo audit`, `pnpm audit`, `osv-scanner`

**Vulnérabilités connues** (cargo audit):

| Crate | Version | CVE/ID | Severity | Fix |
|-------|---------|--------|----------|-----|
| h2 | 0.3.27, 0.4.14 | RUSTSEC-2026-0258 | High | Upgrade ≥0.4.16 |
| quinn-proto | 0.11.14 | RUSTSEC-2026-0185 | 7.5 | Upgrade ≥0.11.15 |
| rsa | 0.9.10 | RUSTSEC-2023-0071 | 5.9 | No fixed upgrade! |
| proc-macro-error2 | 2.0.1 | RUSTSEC-2026-0173 | Unmaintained | — |
| anyhow | 1.0.102 | RUSTSEC-2026-0190 | Unsound | — |
| event-listener | 5.4.1 | RUSTSEC-2026-0221 | Unsound | — |

**Points d'audit**:
1. [ ] Prioriser `h2` et `quinn-proto` (WebRTC concerné)
2. [ ] Évaluer `rsa` — utilisé où? Risque de timing side-channel
3. [ ] Vérifier `pnpm audit` (pnpm non installé — utiliser npm audit)
4. [ ] Vérifier `osv-scanner` (non installé — à installer)
5. [ ] Vérifier `rand_core` pinned à 0.6 (compatibilité argon2)

### Domaine 4: Modèle de Menace STRIDE (Poids: 🔴 CRITICAL)

**Fichier**: `docs/security/threat-model.md` (N'EXISTE PAS ENCORE — à créer)

**Points d'audit**:

1. **Spoofing**:
   - [ ] Auth cookie: HttpOnly, SameSite, token révocable
   - [ ] X25519 key binding (est-ce que la clé publique est liée à l'identité?)

2. **Tampering**:
   - [ ] Messages E2EE: XChaCha20-Poly1305 (authentifié)
   - [ ] Ed25519 signatures pour la non-répudiation

3. **Repudiation**:
   - [ ] Signatures Ed25519 sur les messages — qui vérifie?

4. **Info Disclosure**:
   - [ ] SQLite: pas de chiffrement au repos (file perms only)
   - [ ] Logs: ne jamais logger les clés ou tokens
   - [ ] DB backup: chiffré?

5. **DoS (DT-04)**:
   - [ ] **Risque résiduel HIGH** — rate limiting partiellement implémenté
   - [ ] WebRTC SFU: MediaRelay capacity bornée?
   - [ ] Nginx: connection limits?

6. **Elevation**:
   - [ ] Pas d'admin panel — bon
   - [ ] Mais: `/api/admin/*` routes existent? Vérifier `admin.rs`

### Domaine 5: E2E & Tests Sécurité (Poids: 🟡 HIGH)

**Fichiers**: `.hermes/E2E-TARGETED-REPORT.md`, tests Playwright

**Points d'audit**:

1. **69 échecs E2E** sur les routes auth:
   - [ ] Analyser les échecs: est-ce un problème de test ou une faille réelle?
   - [ ] Vérifier que les routes non-authentiquées retournent bien 401
   - [ ] Tester la modification de mot de passe d'un autre utilisateur (403)

2. **Playwright security tests**:
   - [ ] `api-sanity.spec.ts` lignes 392+: Security section
   - [ ] Coverage des tests de sécurité

## Livrables

1. **`.hermes/SECURITY-AUDIT-LATEST.md`** — Rapport mis à jour
2. **`docs/security/threat-model.md`** — Modèle de menace STRIDE complet (création)
3. **`docs/security/incident-response.md`** — Plan IR (création si absent)
4. **Fichier d'engram PLUR** — type=architectural, domain=nook.security.audit

## Priorisation

```
Phase 1 (URGENT): Domaine 1 + 3 — Crypto + Dépendances vulnérables
Phase 2 (HIGH):   Domaine 2 + 5 — Rate Limiting + E2E tests
Phase 3 (MEDIUM): Domaine 4 — Threat Model (documentation)
```

## Vérification Finale

- [ ] `cargo audit` clean (ou exceptions documentées)
- [ ] `pnpm audit` clean (ou npm audit équivalent)
- [ ] `osv-scanner` clean
- [ ] Zero `unwrap()` dans les chemins crypto critiques
- [ ] Constant-time comparisons vérifiées
- [ ] Threat model documenté
- [ ] Rate limiting fonctionnel (DT-04 marked resolved or documented residual)
- [ ] E2E security tests passent
- [ ] Engram PLUR écrit
