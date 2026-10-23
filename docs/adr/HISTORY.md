# Historique des décisions Nook

> Journal **chronologique** des décisions ayant conduit à l'architecture actuelle.
> Complète `docs/adr/README.md` (inventaire par numéro). Ici, une ligne par décision, **avec sa preuve**.

**Principe** : quand un ADR est impossible à écrire (décision pré-ADR, contexte perdu), on consigne la décision ici avec la preuve par le code. Une ligne sans `← Code:` n'est pas une décision, c'est une rumeur.

---

## Légende

- `← Code:` = fichier:ligne qui atteste la décision
- `← ADR:` = ADR correspondant (si existant)
- `← RFC:` = RFC correspondant (si existant)

---

## 2026-01 — Fondations

| Décision | Preuve |
|---|---|
| Backend **Rust / Axum 0.8** | ← Code: `backend/Cargo.toml:10` · ← ADR: *non documenté (ADR-002)* |
| Frontend **SvelteKit 5** | ← Code: `frontend/package.json` · ← ADR: *non documenté (ADR-002)* |
| Base principale **SQLite** | ← Code: `backend/src/db.rs`, `backend/migrations/001_initial.sql` · ← ADR: *non documenté (ADR-003)* |
| Déploiement **un seul conteneur Docker** | ← Code: `docker-compose.yml:2,58,88` · ← ADR: *non documenté (ADR-001)* — **Superseded** par [ADR-019](019-deployment-topology.md) : **3 services** (`nook`, `nginx-local`, `turn`) + 1 réseau ; la contrainte « 1 conteneur » est remplacée par un **budget RAM/CPU** ← Code: `docker-compose.yml:39-46` |

---

## 2026-02 — Communication

| Décision | Preuve |
|---|---|
| **E2EE Double Ratchet** (Signal) — X25519 + XChaCha20-Poly1305 | ← Code: `backend/src/e2ee.rs`, `frontend/src/lib/e2ee.ts` · ← ADR: *non documenté (ADR-004)* |
| **WebRTC SFU via rustrtc** pour les appels | ← Code: `backend/src/sfu.rs`, `backend/src/webrtc.rs` · ← ADR: *non documenté (ADR-005)* |

---

## 2026-03 — Réseau & authentification

| Décision | Preuve |
|---|---|
| **Auth par cookie** `auth_token` — `HttpOnly`, `SameSite=None; Secure` (WAN) / `SameSite=Lax` (LAN) | ← Code: `backend/src/auth.rs:94-100` · ← ADR: *non documenté (ADR-006)* |
| **TLS LAN = `nginx-local`** · **TLS WAN = reverse proxy externe** (NPM, Caddy, Traefik — au choix de l'utilisateur) | ← ADR: *non documenté (ADR-007)* — **Superseded** par [ADR-019](019-deployment-topology.md) : la formulation « NPM pour LAN/WAN » nommait un composant **hors de tout artefact déployé** ; NPM reste une **recommandation WAN** (`README.md:108,188,199`) et n'est **pas** le proxy LAN ← Code: `nginx-local.conf` (`listen 443 ssl`, `proxy_pass http://nook:3000`), `docker-compose.yml:58`, `Dockerfile.nginx`, `gen-cert.sh` · WAN honoré via `X-Forwarded-Proto` ← Code: `backend/src/auth.rs:184-216`, `backend/src/main.rs:716,725` |

---

## 2026-04 — CI/CD

| Décision | Preuve |
|---|---|
| Images **multi-arch amd64 + arm64** publiées sur **GHCR**, artefacts récupérés via **dawidd6/action-download-artifact** | ← Code: `.github/workflows/Docker.yml:28` · ← ADR: *non documenté (ADR-008)* |
| **Ordre séquentiel de 5 workflows** : Frontend → Backend → Turn → Docker → Release | ← Code: `.github/workflows/{Frontend,Backend,turn,Docker,Release}.yml` · ← ADR: *non documenté (ADR-009)* |

---

## 2026-05 — Frontend

| Décision | Preuve |
|---|---|
| **Svelte 5 runes uniquement** — interdiction de la syntaxe Svelte 4 (`export let`, `$:`, `on:`) | ← Code: `$state`/`$derived` dans `frontend/src/lib/*.svelte.ts` · ← ADR: *non documenté (ADR-010)* |

---

## 2026-07 — Organisation & média

| Décision | Preuve |
|---|---|
| **Équipe Nook : 16 profils d'agents** + ownership de la dette technique DT-01..07 | ← ADR: `011-team-structure-decisions.md` · ← Code: `.hermes/profiles/` (17 profils réels) |
| **Capacité SFU configurable** via `SFU_RELAY_CAPACITY` (défaut 500) + correction de déduplication des tracks (`peer_id` inclus) | ← ADR: `012-sfu-relay-capacity.md` · ← Code: `backend/src/config.rs:22`, `backend/src/sfu.rs:342` |
| ~~**ADR-013** — Dual message store (`writable` + `$state`) : adopter l'Option 1 (source unique, runes)~~ | ← ADR: *orphelin, **clos** le 2026-10-22 (carte `t_566014c5`)* — **décision abandonnée** : appliquée puis **revertée** (cf. 2026-04) · le suivi est porté par **DT-12** |

---

## 2026-08 — Dépendances & E2EE de groupe

| Décision | Preuve |
|---|---|
| ~~**ADR-013** — Dual message store (`writable` + `$state`) : adopter l'Option 1 (source unique, runes)~~ | ← ADR: *orphelin, **clos** le 2026-10-22 (carte `t_566014c5`)* — **décision abandonnée** : appliquée puis **revertée** · ← Code: `frontend/src/lib/chatStore.svelte.ts:85` = encore `writable`, `frontend/src/routes/chat/+page.svelte:122` = encore `localMessages` · le suivi est porté par **DT-12** |
| **`rand_core 0.6 → 0.10` + `argon2 0.5 → 0.6`** — migration **bloquée** (KDF de mot de passe : revue `security-auditor` requise) | ← ADR: `014-rand-core-argon2-migration.md` (**Proposé**, recréé le 2026-10-22) — **non abouti** · ← Code: `backend/Cargo.toml:35,38` = encore `rand_core 0.6`, `argon2 0.5` · `backend/Cargo.lock` = 3 copies de `rand_core` (0.6.4, 0.9.5, 0.10.1) |
| **`sqlx 0.8 → 0.9`** avec compat Axum 0.8 — migration **bloquée** (spike coder requis) | ← ADR: `015-sqlx-08-to-09-migration.md` (**Proposé**, recréé le 2026-10-22) — **non abouti** · ← Code: `backend/Cargo.toml:17` = encore `sqlx 0.8.6` |
| ~~**ADR-016** (migration `chacha20poly1305 0.10 → 0.11`)~~ — **numéro attribué à l'E2EE** | ← ADR: *orphelin, **clos** le 2026-10-22 (carte `t_566014c5`)* — le n° 016 désigne désormais `016-e2ee-group-hybrid.md` · ← Code: usage réel du crate = `backend/src/webrtc.rs:15-18` (**pas** `e2ee.rs` comme le stipulait l'orphelin) · `backend/Cargo.toml:39` = encore `chacha20poly1305 0.10.1` |
| **E2EE groupe global — Hybrid Sender Keys (Option D)** : enveloppe par destinataire pour DMs/groupes privés, clé de groupe globale pour `default_global`, distribution par `crypto_box_seal` | ← ADR: `016-e2ee-group-hybrid.md` (statut **Accepté** depuis le 2026-10-22) · ← RFC: `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md` · ← Code: `backend/src/e2ee.rs:590-1010`, `frontend/src/lib/e2ee.ts:189` · ← Code: `backend/migrations/{019_conversation_keys,022_distribution_status,023_group_key_version}.sql` · commits `ad499a3a`, `3b24583f`, `7bf46454`, `79edfc94` |

**Réserves closes sur cette décision** (revue `security-auditor`, `ACCEPTÉ AVEC RÉSERVES MAJEURES`, carte `t_0fc7cc3f` — tranchées par mesure de code, `docs/adr/DECISIONS.md`, carte `t_08632b95`, 2026-10-22) :

| Réserve | Décision | Preuve |
|---|---|---|
| 1. Rotation automatique non implémentée | **Assumée** — rotation **manuelle** ; `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`) est du **dead code** (zéro appelant), aucun `group_key_rotation_days` (`backend/src/config.rs:5-23`) | ← Code: `frontend/src/lib/e2ee.ts:189`, `backend/src/config.rs:5-23`, jobs périodiques `backend/src/main.rs:443,449` |
| 2. Aucune purge des anciennes versions | **Assumée** — conserve **illimitée**, **aucun** expiry ; `get_my_group_key` accepte toute `version` (`backend/src/e2ee.rs:130`), `prune.rs` ne touche jamais les clés de groupe, et le client n'expire rien | ← Code: `backend/src/e2ee.rs:130`, `backend/src/prune.rs`, `frontend/src/lib/e2ee.ts` |
| 3. *(découverte)* `add_member_key` ne pose pas `distribution_status` | **Assumée** — l'entrée naît `delivered`, `claim_key` ne la sert jamais → le cas « membre sans clé » est hors d'atteinte par ce chemin | ← Code: `backend/src/e2ee.rs:780`, `backend/migrations/022_distribution_status.sql:4`, `backend/src/e2ee.rs:845` |

Les 3 CRITIQUES de la revue restent **levées en code** : `SEC-FIX-1` (`backend/src/e2ee.rs:759-776`), `SEC-FIX-2` (`backend/src/main.rs:501,612`), `SEC-FIX-3` (`backend/src/e2ee.rs:834,961`).

---

## 2026-10 — Messagerie & agents

| Décision | Preuve |
|---|---|
| **Reply-to** (citer / répondre à un message) — Option B : référence simple `reply_to_id`, pas de threads | ← ADR: `017-reply-to-feature.md` · ← Code: `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415`, `frontend/src/lib/chatStore.svelte.ts` · commit `eca8093e` |
| **Stockage de la mémoire par agent** — statu quo recommandé (MEMORY.md + PLUR + skills git), pas de Mem0/Supermemory externe, sur la base de mesures (corpus 38 Ko, 19 profils) | ← ADR: `018-agent-memory-storage.md` (statut `Proposé`, décision en attente) · ← Code: `.hermes/profiles/*/memories/`, `.hermes/plur/` |
| **Tranchage des réserves d'ADR-016** — les 2 réserves de la revue sécurité sont **assumées** (politique réelle écrite dans l'ADR) et une 3e est découverte ; ADR-016 repassé `Accepté` ; 2 cartes `@coder` de suivi créées (job de rotation + contrôle admin ; purge + correctif `distribution_status`) | ← Doc: `docs/adr/DECISIONS.md` · ← ADR: `016-e2ee-group-hybrid.md` · ← Index: `docs/adr/README.md` §4.5, §7 (#9, #10) · ← Dette: DT-05 |

---

## 2026-10 — Sort des orphelins ADR 013 / 014 / 015 (et du 016 en collision)

Carte `t_566014c5` (`@architect`). Les quatre fichiers orphelins `0013-…`, `0014-…`, `0015-…`, `0016-…`
(rédigés le 2026-07-11 et le 2026-08-07, **jamais commités**) sont tranchés. Numérotation **autoritative** =
la **chronologie** (ADR-011 le 2026-07-02, ADR-012 le 2026-07-11, donc l'orphelin du 2026-07-11 suit au n° 013)
et les **titres des fichiers** : **013 = dual message store** (préexistant, 2026-07-11), **014 = rand_core/argon2**,
**015 = sqlx**, **016 = chacha20poly1305**.

| ADR | Fichier orphelin | Sort | Justification (preuve) |
|---|---|---|---|
| **013** | `0013-dual-message-store-architecture.md` | **Clore** | Statut « Accepted » **faux** : la décision a été **appliquée puis revertée** · ← Code: commit `1b47e159` (2026-04-13 07:04, Option 1) puis `2601bcca` (2026-04-13 07:12, « Use Svelte writable store for messages cross-file reactivity ») · ← Code: `frontend/src/lib/chatStore.svelte.ts:85` = `writable`, `frontend/src/routes/chat/+page.svelte:122` = `localMessages` · le suivi est porté par **DT-12** |
| **014** | `0014-rand_core-argon2-migration.md` | **Recréer** → `014-rand-core-argon2-migration.md` | Statut « Proposed (blocked on security-auditor coordination) » **exact** · ← Code: `backend/Cargo.toml:35` = `rand_core 0.6`, `:38` = `argon2 0.5` · `backend/Cargo.lock` = 3 copies de `rand_core` · le contenu est réel et reste la référence du blocage |
| **015** | `0015-sqlx-08-to-09-migration.md` | **Recréer** → `015-sqlx-08-to-09-migration.md` | Statut « Proposed (blocked — architect review required) » **exact** · ← Code: `backend/Cargo.toml:17` = `sqlx 0.8.6` · le spike n'a jamais été lancé |
| **016** | `0016-chacha20poly1305-010-to-011-migration.md` | **Clore** | Le n° **016 est attribué à l'E2EE** (`016-e2ee-group-hybrid.md`, implémenté) — le numéro n'est pas disponible · de plus la prémisse de l'orphelin est **partiellement fausse** : le crate est utilisé dans `backend/src/webrtc.rs:15-18` (chiffrement de fichiers), **pas** dans `e2ee.rs` · ← Code: `backend/Cargo.toml:39` = encore `chacha20poly1305 0.10.1` |

**Règle de numérotation réaffirmée** : un numéro = une décision, jamais réutilisé. Les numéros **013** et **016**
(dans son acception « migration ») restent **réservés et non réattribués**. Le prochain numéro libre est **019**
(016, 017, 018 étant occupés).

> **Corrections d'écriture** (par `@architect`, `t_566014c5`) : l'index `docs/adr/README.md` §4.4/§4.5 et le présent
> historique numérotaient les migrations 013=rand_core, 014=sqlx, 015=chacha — **décalage d'un cran**. La **chronologie**
> des ADR-011 (2026-07-02) et ADR-012 (2026-07-11) et les **titres des fichiers** donnent : 013=dual store (2026-07-11),
> 014=rand_core/argon2, 015=sqlx, 016=chacha. Les sections §3, §4.4, §4.5 et §6 de `README.md` sont alignées sur cette numérotation.

---

## 2026-10 — Topologie de déploiement (ADR-019, supersede ADR-001 et ADR-007)

Carte `t_107d5ece` (`@architect`). Les deux décisions de déploiement du tableau
`.hermes/profiles/architect/SOUL.md:36-52` étaient **fausses dans les faits** et **inopposables**
comme arguments de rejet (`architect/SOUL.md:177`). Tranché : **`Reformulée`** — un ADR-019
supersède les deux formulations, sur la base de **mesures** et non de mémoire.

| Décision | Preuve |
|---|---|
| **Un seul hôte, 3 services conteneurisés** (`nook`, `nginx-local`, `turn`) + réseau `nook-network` — la règle « un seul conteneur » est **supprimée** | ← ADR: `019-deployment-topology.md` §1 · ← Code: `docker-compose.yml:2,58,88,115`, `docker-compose.ci.yml:2,58,86` |
| **La garde-fou de simplicité porte sur le budget RAM/CPU, pas sur le nombre de conteneurs** : `cpus` `${CPUS_LIMIT:-1.5}`, `memory` `${MEMORY_LIMIT:-1028M}`, réservations `0.5` / `256M` (service `nook` uniquement) | ← ADR: `019-deployment-topology.md` §2 · ← Code: `docker-compose.yml:39-46` |
| **TLS LAN = `nginx-local`** : HTTPS, certificat auto-signé auto-généré (10 ans, CN = IP, SAN = IP + `localhost` + `nook.local`), `proxy_pass http://nook:3000`, injection de `X-Forwarded-Proto`, redirection HTTP→HTTPS, CA publié sur `/ca` | ← ADR: `019-deployment-topology.md` §3 · ← Code: `nginx-local.conf`, `Dockerfile.nginx`, `gen-cert.sh`, `nginx-entrypoint.sh`, `docs/nginx-local.md` |
| **TLS WAN = reverse proxy externe, hors périmètre Nook** (NPM / Caddy / Traefik au choix de l'utilisateur) — Nook ne le déploie pas ; sa seule obligation est d'honorer `X-Forwarded-Proto` pour choisir `SameSite=None; Secure` | ← ADR: `019-deployment-topology.md` §4 · ← Code: `backend/src/auth.rs:184-216`, `backend/src/main.rs:716,725` · doc de recommandation : `README.md:108,188,199` |

**Sort des ADR remplacées** : **ADR-001 → `Superseded`** (formulation morte, contrainte reformulée
sur le budget) · **ADR-007 → `Superseded`** (nom de composant faux, topologie réelle documentée).

**Corrections d'écriture** : la formulation « aucun artefact ne référence Nginx Proxy Manager » est
**imprécise** — NPM apparaît dans `README.md`, `AGENTS.md`, `docs/nginx-local.md`,
`.hermes/roles/*.md` comme **recommandation WAN**. Ce qui est faux, c'est « NPM est **le** proxy de
Nook ». L'alignement des doc concernées est suivi en §7 (action 9 de `README.md`).

**Réserve** : Gate 4 (`Docker / CI / Déploiement`) — `@architect` ✅ · `@ci-monitor` / `@deployer`
**sans objet** : aucune modification de `Dockerfile*`, `docker-compose*` ni workflow, l'ADR enregistre
un état déjà en production.

---

## Récapitulatif des statuts

| Période | Décisions | Documentées ? |
|---|---|---|
| 2026-01 → 2026-05 | 10 | ❌ Aucune (ADR-001..010) · **001 et 007 supersédés par ADR-019** |
| 2026-07 | 2 | ✅ ADR-011, ADR-012 |
| 2026-08 (deps) | 3 | ❌ Orphelins (ADR-013..015) |
| 2026-08 (E2EE) | 1 | ✅ ADR-016 (**Accepté**, réserves closes le 2026-10-22) |
| 2026-10 | 2 | ✅ ADR-017 (accepté), ADR-018 (proposé) |
| 2026-10 (topologie) | 1 | ✅ **ADR-019** (supersede 001, 007) |

**Total : 19 décisions recensées · 6 documentées en ADR · 13 non documentées ou orphelines.**

---

*Maintenu par `@docs-writer` · Décision de non-recréation des ADR 001-010/013-015 : `@architect` (validée par l'inventaire git, carte `t_db6f9481`) · Reformulation de ADR-001/007 → ADR-019 : `@architect`, carte `t_107d5ece` (2026-10-22)*
