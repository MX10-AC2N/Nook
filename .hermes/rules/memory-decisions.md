# 🧠 Décisions Architecturales — Nook

> Décisions passées avec leur justification. Consulter avant de proposer des changements.

---

## [D01] Deux fichiers rapport backend (amd64/arm64) — Session 20

**Décision** : Deux fichiers séparés `BACKEND-BUILD-REPORT-amd64.md` et `BACKEND-BUILD-REPORT-arm64.md`.  
**Pourquoi** : Matrix GitHub Actions compile les deux architectures en parallèle. Un seul fichier → deux jobs commitent simultanément → non-fast-forward git, l'un écrase l'autre.  
**Alternative rejetée** : Artifact zip combiné → complexité inutile.

---

## [D02] Cookie HttpOnly + token DB — Session 2

**Décision** : Auth via cookie `HttpOnly; SameSite=Lax` + token stocké en DB.  
**Pourquoi** : Token révocable côté serveur (logout = NULL en DB). Résistant XSS. Compatible LAN sans HTTPS.  
**Note WAN** : `SameSite=None; Secure` activé via `X-Forwarded-Proto: https` détecté par le backend.

---

## [D03] rand_core 0.6 forcé comme dep explicite — Session 2

**Décision** : `rand_core = "0.6"` ajouté explicitement dans `Cargo.toml`.  
**Pourquoi** : `argon2` dépend de `rand_core 0.6`. `rand 0.9` utilise `rand_core 0.9`. Sans dep explicite, Cargo résout en 0.9 → `OsRng` incompatible → erreur de type à la compilation.  
**Règle** : Toujours `use rand_core::OsRng` (jamais `rand::rngs::OsRng`).

---

## [D04] Dockerfile.release séparé — Session 3

**Décision** : Deux Dockerfiles — `Dockerfile` (build depuis sources) et `Dockerfile.release` (binaires pré-compilés).  
**Pourquoi** : Build Rust complet (~10min) inacceptable à chaque déploiement prod. `Dockerfile.release` copie les binaires pré-compilés par `Backend.yml` → déploiement en ~30s.  
**Règle** : `Dockerfile` utilisé par `test-nook.yml` (intégration), `Dockerfile.release` par `Docker.yml` (prod).

---

## [D05] distroless + init container — Session 4

**Décision** : Image finale `gcr.io/distroless/cc-debian12`, init container `alpine:3` pour chown volumes.  
**Pourquoi** : distroless = surface d'attaque minimale, pas de shell. Volumes Docker créés root → distroless user (65532) ne peut pas écrire → init container règle les permissions.  
**Alternative rejetée** : Image debian slim → trop lourde, surface d'attaque plus grande.

---

## [D06] E2E_SETUP=1 pour user CI — Session 7

**Décision** : Variable d'env `E2E_SETUP=1` → backend crée `e2e_ci` (approved=1, no pwd change) au démarrage.  
**Pourquoi** : Les tests E2E ont besoin d'un user pré-approuvé. Impossible d'approuver via UI en CI. `docker-compose.ci.yml` injecte la variable.  
**Sécurité** : Ne jamais mettre `E2E_SETUP=1` en prod (docker-compose.yml prod ne l'a pas).

---

## [D07] clearSession via API logout (pas goto) — Session 22

**Décision** : `clearSession()` fait `page.request.post(logout)` + `clearCookies()`. Pas de `goto('/')`.  
**Pourquoi** : `goto('/')` monte le layout → `authStore.init()` → `fetch('/api/auth/me')` avec cookie encore valide → 200 → `isAuthenticated=true` → redirect → timeout. L'API logout révoque le token en DB AVANT toute navigation.  
**Règle** : Ne jamais naviguer dans le browser avant d'avoir révoqué le token serveur.

---

## [D08] fullyParallel: false en CI Playwright — Session 21

**Décision** : `fullyParallel: false` dans `playwright.config.ts`.  
**Pourquoi** : Avec `fullyParallel: true` + `workers: 1`, tous les tests du même fichier partagent le browser context → même localStorage → pollution entre tests → timeouts.  
**Note** : Avec des browser contexts isolés par test (`test.use({ storageState: ... })`), on pourrait revenir à `true` — mais la complexité n'est pas justifiée pour ce projet.

---

## [D09] conversation_participants (pas conversation_members) — Session 5

**Décision** : Le nom réel de la table est `conversation_participants`.  
**Pourquoi** : Migration `001_initial.sql` crée `conversation_participants`. Une incohérence dans `db.rs` (qui utilisait `conversation_members`) causait des erreurs SQL 500. Corrigé et stabilisé.  
**Règle** : Ne jamais renommer sans migration SQL correspondante.

---

## [D10] Workflows GitHub Actions tous manuels — Session 3

**Décision** : Aucun workflow déclenché automatiquement (pas de `on: push`).  
**Pourquoi** : Projet familial homeserver — pas besoin de CI automatique. Les déclenchements manuels permettent de choisir quand compiler/tester. Économise les GitHub Actions minutes.  
**Exception possible** : Ajouter un check lint automatique sur PR si l'équipe grandit.


## 🌐 SFU (Selective Forwarding Unit) — LOT 6

### Décision: rustrtc comme SFU (pas coturn ou mediasoup)
- **Pourquoi:** rustrtc est une crate Rust pure, compatible avec notre stack Axum
- **Alternative rejetée:** mediasoup (Node.js, incompatible), coturn (seulement TURN, pas SFU)
- **Version:** rustrtc 0.3.40

### Décision: Signalisation via WebSocket existant (pas REST)
- **Pourquoi:** Le canal WS `/ws` existe déjà pour le signaling P2P
- **Avantage:** Pas besoin d'ouvrir de nouveaux endpoints HTTP
- **Types WS ajoutés:** sfu_join, sfu_answer, sfu_candidate, sfu_leave, sfu_renegotiate_offer, sfu_peers, sfu_error

### Décision: Auto-SFU pour 3+ participants
- **Seuil:** `participantIds.length >= 3` → bascule automatique en SFU
- **Raison:** Le mesh P2P fonctionne bien jusqu'à 2 participants, au-delà le SFU est plus efficace
- **Toggle manuel:** L'utilisateur peut basculer entre SFU et P2P via le bouton dans call-controls

### Décision: MediaRelay avec capacité 500
- **Pourquoi:** Assez grand pour éviter les pertes de samples RTCP
- **Pattern:** MediaRelay::with_capacity(track, 500) pour chaque track entrante

### Décision: PLI periodique toutes les 3s
- **Pourquoi:** Forcer les keyframes régulières pour les nouveaux participants
- **Implémentation:** tokio::time::interval(3s) → track.request_key_frame().await

### Décision: added_sources HashSet pour éviter les doublons
- **Clé:** "{user_id}:{peer_id}:{kind:?}" format de string
- **Pourquoi:** Eviter d'ajouter la même track plusieurs fois au même peer (génère des duplicates SDP rejects)

### Décision: Negotiation différée via negotiation_pending AtomicBool
- **Pourquoi:** Si signaling_state n'est pas Stable, on marque pending et on réessaie plus tard
- **Pattern:** peer.negotiation_pending.store(true) → drain_pending_offer() plus tard
