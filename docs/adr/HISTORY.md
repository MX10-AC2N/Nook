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
| Déploiement **un seul conteneur Docker** | ← Code: `docker-compose.yml` · ← ADR: *non documenté (ADR-001)* — **décision aujourd'hui obsolète** : 3 services (`nook`, `nginx-local`, `turn`) |

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
| **TLS LAN/WAN par Nginx Proxy Manager** | ← ADR: *non documenté (ADR-007)* — **faux dans les faits** : aucun artefact ne référence NPM. Le proxy réel est `nginx-local` ← Code: `docker-compose.yml:58`, `docs/nginx-local.md` |

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

---

## 2026-08 — Dépendances & E2EE de groupe

| Décision | Preuve |
|---|---|
| ~~Migration `rand_core 0.6 → 0.10` + `argon2 0.5 → 0.6`~~ | ← ADR: *orphelin, jamais commité (ADR-013)* — **non abouti** : `backend/Cargo.toml:35,38` = encore `rand_core 0.6`, `argon2 0.5` |
| ~~Migration `sqlx 0.8 → 0.9` avec compat Axum 0.8~~ | ← ADR: *orphelin, jamais commité (ADR-014)* — **non abouti** : `backend/Cargo.toml:17` = encore `sqlx 0.8.6` |
| ~~Migration `chacha20poly1305 0.10 → 0.11` (chemin E2EE)~~ | ← ADR: *orphelin, jamais commité (ADR-015)* — **non abouti** : `backend/Cargo.toml:39` = encore `chacha20poly1305 0.10.1` |
| **E2EE groupe global — Hybrid Sender Keys (Option D)** : enveloppe par destinataire pour DMs/groupes privés, clé de groupe globale pour `default_global`, distribution par `crypto_box_seal` | ← ADR: `016-e2ee-group-hybrid.md` (statut **Accepté** depuis le 2026-10-22) · ← RFC: `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md` · ← Code: `backend/src/e2ee.rs:590-1010`, `frontend/src/lib/e2ee.ts:189` · ← Code: `backend/migrations/{019_conversation_keys,022_distribution_status,023_group_key_version}.sql` · commits `ad499a3a`, `3b24583f`, `7bf46454`, `79edfc94` |

**Réserves closes sur cette décision** (revue `security-auditor`, `ACCEPTÉ AVEC RÉSERVES MAJEURES`, carte `t_0fc7cc3f` — tranchées par mesure de code, `docs/adr/DECISIONS.md`, carte `t_08632b95`, 2026-10-22) :

| Réserve | Décision | Preuve |
|---|---|---|
| 1. Rotation automatique non implémentée | **Assumée** — rotation **manuelle** ; `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`) est du **dead code** (zéro appelant), aucun `group_key_rotation_days` (`backend/src/config.rs:5-23`) | ← Code: `frontend/src/lib/e2ee.ts:189`, `backend/src/config.rs:5-23`, jobs périodiques `backend/src/main.rs:443,449` |
| 2. Aucune purge des anciennes versions | **Assumée** — conserve **illimitée**, **aucun** expiry ; `get_my_group_key` accepte toute `version` (`backend/src/e2ee.rs:130`), `prune.rs` ne touche jamais les clés de groupe, et le client n'expire rien | ← Code: `backend/src/e2ee.rs:130`, `backend/src/prune.rs`, `frontend/src/lib/e2ee.ts` |
| 3. *(découverte)* `add_member_key` ne pose pas `distribution_status` | **Assumée** — l'entrée naît `delivered`, `claim_key` ne la sert jamais → le cas « membre sans clé » est hors d'atteinte par ce chemin | ← Code: `backend/src/e2ee.rs:780`, `backend/migrations/022_distribution_status.sql:4`, `backend/src/e2ee.rs:838-841` |

Les 3 CRITIQUES de la revue restent **levées en code** : `SEC-FIX-1` (`backend/src/e2ee.rs:759-767`), `SEC-FIX-2` (`backend/src/main.rs:501,612`), `SEC-FIX-3` (`backend/src/e2ee.rs:834,959`).

---

## 2026-10 — Messagerie & agents

| Décision | Preuve |
|---|---|
| **Reply-to** (citer / répondre à un message) — Option B : référence simple `reply_to_id`, pas de threads | ← ADR: `017-reply-to-feature.md` · ← Code: `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415`, `frontend/src/lib/chatStore.svelte.ts` · commit `eca8093e` |
| **Stockage de la mémoire par agent** — statu quo recommandé (MEMORY.md + PLUR + skills git), pas de Mem0/Supermemory externe, sur la base de mesures (corpus 38 Ko, 19 profils) | ← ADR: `018-agent-memory-storage.md` (statut `Proposé`, décision en attente) · ← Code: `.hermes/profiles/*/memories/`, `.hermes/plur/` |
| **Tranchage des réserves d'ADR-016** — les 2 réserves de la revue sécurité sont **assumées** (politique réelle écrite dans l'ADR) et une 3e est découverte ; ADR-016 repassé `Accepté` ; 2 cartes `@coder` de suivi créées (job de rotation + contrôle admin ; purge + correctif `distribution_status`) | ← Doc: `docs/adr/DECISIONS.md` · ← ADR: `016-e2ee-group-hybrid.md` · ← Index: `docs/adr/README.md` §4.5, §7 (#9, #10) · ← Dette: DT-05 |

---

## Récapitulatif des statuts

| Période | Décisions | Documentées ? |
|---|---|---|
| 2026-01 → 2026-05 | 10 | ❌ Aucune (ADR-001..010) |
| 2026-07 | 2 | ✅ ADR-011, ADR-012 |
| 2026-08 (deps) | 3 | ❌ Orphelins (ADR-013..015) |
| 2026-08 (E2EE) | 1 | ✅ ADR-016 (**Accepté**, réserves closes le 2026-10-22) |
| 2026-10 | 3 | ✅ ADR-017 (accepté), ADR-018 (proposé), tranchage ADR-016 (`DECISIONS.md`) |

**Total : 19 décisions recensées · 5 documentées en ADR · 14 non documentées ou orphelines.**

---

*Maintenu par `@docs-writer` · Décision de non-recréation des ADR 001-010/013-015 : `@architect` (validée par l'inventaire git, carte `t_db6f9481`) · Tranchage des réserves d'ADR-016 : `@architect` (mesure de code, carte `t_08632b95`)*
