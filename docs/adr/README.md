# Index des ADR — Nook

> Source of truth de la gouvernance architecturale. Un ADR = une décision, pas un document de fond.
> **Propriétaire** : `@architect` · **Index maintenu par** : `@docs-writer`

---

## 1. Comment lire cet index

| Champ | Signification |
|---|---|
| **Statut** | `Accepté` · `Proposé` · `Rejeté` · `Superseded` · `Orphelin` (voir §4) |
| **Supersede** | Numéro de l'ADR qu'il remplace (sinon `—`) |
| **Superseded by** | Numéro de l'ADR qui le remplace (sinon `—`) |
| **Preuve** | Lien vers l'ADR **et** vers le code/artefact qui atteste que la décision est réelle |

**Règle de numérotation** : un numéro = une décision, jamais réutilisé. Un ADR «orphelin» garde son numéro dans le tableau pour ne pas casser les liens déjà publiés ; il n'est pas recréé.

---

## 2. ADR existants (sur disque)

| ADR | Fichier | Titre | Statut | Date | Preuve code |
|-----|---------|-------|--------|------|-------------|
| [ADR-011](011-team-structure-decisions.md) | `011-team-structure-decisions.md` | Team Nook — 16-Profile Agent Structure Decisions | **Accepté** | 2026-07-02 | `.hermes/profiles/` (17 profils) |
| [ADR-012](012-sfu-relay-capacity.md) | `012-sfu-relay-capacity.md` | Configurable SFU MediaRelay Capacity | **Accepté** | 2026-07-11 | `backend/src/config.rs:22`, `backend/src/sfu.rs:342` |
| [ADR-016](016-e2ee-group-hybrid.md) | `016-e2ee-group-hybrid.md` | E2EE groupe global — Hybrid Sender Keys (Option D) | **Accepté** (réserves closes le 2026-10-22 — voir `DECISIONS.md`) | 2026-08-24 | `backend/src/e2ee.rs:590`, `frontend/src/lib/e2ee.ts:189` · 3 écarts actés : `DECISIONS.md` |
| [ADR-017](017-reply-to-feature.md) | `017-reply-to-feature.md` | Reply-to (citer / répondre à un message) | **Accepté** | 2026-08-27 (récupéré 2026-10-22) | `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415`, commit `eca8093e` |
| [ADR-018](018-agent-memory-storage.md) | `018-agent-memory-storage.md` | Stockage de la mémoire par agent — MEMORY.md + PLUR vs Mem0 vs Supermemory | **Proposé** (en attente décision Mo Ju) | 2026-10-22 | `.hermes/profiles/*/memories/`, `.hermes/plur/` |
| [ADR-019](019-deployment-topology.md) | `019-deployment-topology.md` | Topologie de déploiement — services, budget RAM/CPU, ingress TLS | **Accepté** | 2026-10-22 | `docker-compose.yml:2,58,88,39-46`, `nginx-local.conf`, `Dockerfile.nginx` |

**ADR absents de ce tableau, cités mais jamais rédigés** : 001-010, 013-015. Voir §4.

---

## 3. Convention de nommage (norme unique)

Le repo mélangeait trois formes : `011-*.md`, `012-*.md`, `ADR-016-*.md`.

**Norme retenue** — appliquée désormais partout :

```
docs/adr/NNNN-title-kebab-case.md     NNNN = numéro 4 chiffres, zéro-padding
```

Exemples valides : `019-single-container-budget.md`, `020-postgres-migration.md`.

Règles :
1. Le préfixe `ADR-` est **facultatif** dans le nom de fichier, mais **obligatoire** dans le titre et les références (`# ADR-019 : …`).
2. Le numéro est unique et jamais recyclé, y compris pour un ADR orphelin (§4).
3. Le titre est en kebab-case ASCII, sans accents, sans espaces.
4. Un lien qui pointe vers un ADR utilise la forme `ADR-NNNN` (jamais `011`).
5. Un ADR qui remplit aussi un rôle de RFC vit dans `docs/rfc/RFC-NNNN-…`, avec un lien croisé dans les deux sens.

**Table de correspondance (historique)**

| Ancien nom sur disque | Nom normalisé | Note |
|---|---|---|
| `011-team-structure-decisions.md` | `011-team-structure-decisions.md` | conforme (inchangé) |
| `012-sfu-relay-capacity.md` | `012-sfu-relay-capacity.md` | conforme (inchangé) |
| `ADR-016-e2ee-group-hybrid.md` | `016-e2ee-group-hybrid.md` | ✅ renommé (commit `3c4e0ca8`) |
| `ADR-017-reply-to-feature.md` | `017-reply-to-feature.md` | ✅ commité au nom normalisé (carte `t_0b0decb6`) |
| `ADR-018-agent-memory-storage.md` | `018-agent-memory-storage.md` | ✅ renommé (commit `3c4e0ca8`) |
| `0013-dual-message-store-architecture.md` | — | ⛔ **clos** (décision appliquée puis revertée), carte `t_566014c5` |
| `0014-rand_core-argon2-migration.md` | `014-rand-core-argon2-migration.md` | ✅ **recréé** (Proposé, bloqué), carte `t_566014c5` |
| `0015-sqlx-08-to-09-migration.md` | `015-sqlx-08-to-09-migration.md` | ✅ **recréé** (Proposé, bloqué), carte `t_566014c5` |
| `0016-chacha20poly1305-010-to-011-migration.md` | — | ⛔ **clos** — doublon de collision, n° 016 attribué à l'ADR E2EE |

Les renommages sont listés en §7 (action en attente).

---

## 4. Sort des ADR 001-010 et 013-015 — décision documentée

### 4.1 Constat

Le tableau `.hermes/profiles/architect/SOUL.md:36-52` liste les ADR 001 à 015 comme `Accepted`.

**Aucun fichier ADR 001-010 n'existe**, ni sur disque, ni dans aucun objet git du dépôt. Preuve :

```
$ git ls-files docs/adr/
docs/adr/011-team-structure-decisions.md
docs/adr/012-sfu-relay-capacity.md
docs/adr/ADR-018-agent-memory-storage.md

$ # les 37 396 objets git (commits + blobs, atteignables ou non) contiennent exactement :
#   "# ADR-011: …", "# ADR-016 : …", "# ADR-018 : …"   — et rien d'autre en ADR-00x/01x
```

`git log --all --diff-filter=D` ne montre aucun ADR supprimé : **ces fichiers n'ont jamais été commités**.

### 4.2 Origine probable

Le tableau de `architect/SOUL.md` a été écrit en **une fois**, le 2026-08-16, par l'Architect (carte kanban `t_d0b262d1`, « Architectural State Report »). Les titres 001-015 y sont énumérés comme un inventaire de mémoire, pas comme un inventaire de fichiers lus. Le 2026-08-16, l'audit `.hermes/reports/documentation-audit-2026-05-04.md:21` établissait déjà que `docs/adr/` était **vide**. Les entrées 001-010 et 013-015 sont donc des **résumés de décisions prises hors ADR** (choix de stack, de conteneur, de DB, de crypto) — jamais rédigées, donc jamais commitées.

### 4.3 Décision : ne PAS recréer rétroactivement

**On ne recrée pas les ADR 001-010 ni 013-015.**

Raisons :
1. **Ne pas fabriquer une décision.** Rédiger aujourd'hui un « ADR-003 SQLite for primary DB » reviendrait à inventer un contexte, des options et des conséquences que personne n'a débattus en 2026-01. Un ADR faux est pire qu'un ADR absent : il **fige une justification inventée** et devient une référence citée par les revues de design.
2. **Le contexte d'origine est perdu.** Aucun commit, kanban, RFC ou archive ne contient la discussion d'origine de ces dix décisions (vérifié : recherche plein texte sur 16 125 fichiers Hermes + 37 396 objets git).
3. **Les décisions sont réelles, la documentation ne l'est pas.** Le code **atteste** chaque décision (cf. §4.5). La preuve de la décision, c'est le code, pas le document.

### 4.4 Collision de numérotation 013-016 (bug historique majeur)

Le 2026-08-07, l'Architect a créé trois ADR de migration de dépendances **en partant du numéro 013** (carte `t_dd44fc37`), en supposant — à tort — que 011 et 012 étaient les deux seuls ADR existants et que 013 était le prochain numéro libre. Or un ADR **préexistant** du 2026-07-11 occupait déjà ce numéro. Numérotation **autoritative** (chronologie + titres des fichiers) :

| Numéro | Attribué à | Fichier (jamais commité) |
|---|---|---|
| 013 | Dual message store (`writable` + `$state`) — **préexistant, 2026-07-11** | `0013-dual-message-store-architecture.md` |
| 014 | rand_core 0.6→0.10 + argon2 0.5→0.6 | `0014-rand_core-argon2-migration.md` |
| 015 | sqlx 0.8→0.9 | `0015-sqlx-08-to-09-migration.md` |
| 016 | chacha20poly1305 0.10→0.11 | `0016-chacha20poly1305-010-to-011-migration.md` |
| 016 | E2EE groupe global (Option D) | `016-e2ee-group-hybrid.md` (**implémenté**) |

**Un même numéro a failli désigner deux décisions différentes** (016 = migration de dépendance *ou* architecture E2EE). C'est la cause directe de l'incohérence actuelle.

**Sort décidé** (carte `t_566014c5`, `@architect`, 2026-10-22) :

- **ADR-013 (dual message store) → clos.** Son statut « Accepted » était **faux** : la décision a été **appliquée puis revertée**. Le commit `1b47e159` (2026-04-13) appliquait l'Option 1 (runes) ; `2601bcca` (2026-04-13, 8 min plus tard) rétablissait le `writable`. Le code atteste : `frontend/src/lib/chatStore.svelte.ts:85` = `writable`, `frontend/src/routes/chat/+page.svelte:122` = `localMessages`. Le suivi est porté par **DT-12**. Le numéro 013 reste **réservé, non réattribué**.
- **ADR-014 (rand_core/argon2) et ADR-015 (sqlx) → recréés** au nom normalisé (`014-rand-core-argon2-migration.md`, `015-sqlx-08-to-09-migration.md`). Leur statut réel — **Proposé, bloqué** — est exact pour **014** (revue `security-auditor` toujours ouverte ; `argon2 0.5` / `rand_core 0.6` toujours épinglés `backend/Cargo.toml:35,38`) ; **015 a été débloqué le 2026-10-22 par le spike coder (`t_60a24c82`) : migration sqlx 0.9 faisable en une passe avec Axum 0.8 → Accepté**, mais `sqlx 0.8.6` reste épinglé `backend/Cargo.toml:17` en attendant application de la migration.
- **ADR-016 (migration chacha20poly1305) → clos.** Le numéro **016 appartient à l'ADR E2EE** (option D, implémentée) — le numéro n'est pas disponible. De plus la prémisse de l'orphelin est **partiellement fausse** : le crate est utilisé dans `backend/src/webrtc.rs:15-18` (chiffrement de fichiers), **pas** dans `e2ee.rs`. La dépendance reste épinglée à `0.10.1` (`backend/Cargo.toml:39`).

Le numéro **016** appartient donc à l'ADR E2EE, seul ADR réellement implémenté.

### 4.5 Statut de chaque numéro cité — preuve par le code

| ADR | Décision prétendue | Fichier ADR | Statut réel | Preuve dans le code |
|-----|--------------------|-------------|-------------|---------------------|
| 001 | Single Docker container deployment | — | **Superseded** (→ [ADR-019](019-deployment-topology.md)) | `docker-compose.yml:2,58,88` = **3 services** (`nook`, `nginx-local`, `turn`) + 1 réseau (`:115`) · `docker-compose.ci.yml:2,58,86` idem · idem ADR-018:122 · la contrainte « 1 conteneur » n'a jamais été vraie ; remplacée par un **budget RAM/CPU** ← Code: `docker-compose.yml:39-46` (`MEMORY_LIMIT:-1028M`, `CPUS_LIMIT:-1.5`) |
| 002 | Rust/Axum + SvelteKit 5 stack | — | **Vrai, non documenté** | `backend/Cargo.toml:10` (`axum 0.8`), `frontend/package.json` (SvelteKit 5) |
| 003 | SQLite for primary DB | — | **Vrai, non documenté** | `backend/src/db.rs`, `backend/migrations/*.sql` |
| 004 | E2EE Double Ratchet (Signal) | — | **Vrai, non documenté** | `backend/src/e2ee.rs`, `frontend/src/lib/e2ee.ts` |
| 005 | WebRTC SFU (rustrtc) for calls | — | **Vrai, non documenté** | `backend/src/sfu.rs`, `backend/src/webrtc.rs` |
| 006 | Cookie-based auth (HttpOnly, SameSite) | — | **Vrai, non documenté** | `backend/src/auth.rs:94-100` (`auth_token`, `HttpOnly`, `SameSite=None/Lax`) |
| 007 | Nginx Proxy Manager for LAN/WAN TLS | — | **Superseded** (→ [ADR-019](019-deployment-topology.md)) | NPM n'est **référencé par aucun artefact déployé** (pas de service, image, workflow ni certificat) ; cité comme *recommandation WAN* seulement : `README.md:108,188,199`, `AGENTS.md:47`, `docs/nginx-local.md:57-69`, `.hermes/roles/architect.md:140` · le proxy LAN réel est `nginx-local` ← Code: `nginx-local.conf` (`listen 443 ssl`, `proxy_pass http://nook:3000`), `docker-compose.yml:58`, `Dockerfile.nginx` |
| 008 | GHCR multi-arch (dawidd6 action) | — | **Vrai, non documenté** | `.github/workflows/Docker.yml:28` (`dawidd6/action-download-artifact@v6`) |
| 009 | 5-workflow CI order (FE→BE→Turn→Docker→Release) | — | **Vrai, non documenté** | `.github/workflows/{Frontend,Backend,turn,Docker,Release}.yml` |
| 010 | Svelte 5 Runes only (no Svelte 4 syntax) | — | **Vrai, non documenté** | `$state`/`$derived` dans `frontend/src/lib/*.svelte.ts` |
| 011 | Team Nook — 16-Profile Agent Structure | `011-team-structure-decisions.md` | **Accepté** | `.hermes/profiles/` |
| 012 | SFU MediaRelay Capacity & Track Dedup | `012-sfu-relay-capacity.md` | **Accepté** | `backend/src/config.rs:22`, `backend/src/sfu.rs:342` |
| 013 | rand_core 0.6→0.10 + argon2 0.5→0.6 | orphelin | **Non abouti** | `backend/Cargo.toml:35,38` = encore `rand_core 0.6`, `argon2 0.5` |
| 014 | sqlx 0.8→0.9 + Axum 0.8 compat | orphelin | **Non abouti** | `backend/Cargo.toml:17` = encore `sqlx 0.8.6` |
| 015 | chacha20poly105 0.10→0.11 (E2EE path) | orphelin | **Non abouti** | `backend/Cargo.toml:39` = encore `chacha20poly1305 0.10.1` |
| 016 | E2EE groupe global — Hybrid Sender Keys | `016-e2ee-group-hybrid.md` | **Accepté** | routes `claim-key` (`backend/src/e2ee.rs:834`), `group-key-metadata` (`:961`) · rate limit E2EE (`backend/src/main.rs:501`) · **3 écarts assumés** : rotation manuelle (`frontend/src/lib/e2ee.ts:189` dead code), conserve illimitée (`backend/src/e2ee.rs:130`), `distribution_status` non posé (`:780`) — détail : `DECISIONS.md` |
| 017 | Reply-to (citer/répondre) | `017-reply-to-feature.md` | **Accepté** | `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415` · commit `eca8093e` (récupéré 2026-10-22) |
| 018 | Mémoire par agent (MEMORY.md vs Mem0 vs Supermemory) | `018-agent-memory-storage.md` | **Proposé** | `.hermes/profiles/*/memories/`, `.hermes/plur/` |

> **Conséquence pour les revues de design** : citer `ADR-003` ou `ADR-001` comme justification **ne fonctionne pas** — il n'y a pas de fichier à lier. La preuve admissible est le **code** (`← Code: backend/Cargo.toml:17`). Utiliser la colonne «Preuve dans le code».
>
> **Exception**: `ADR-019` **remplace** les décisions 001 et 007 — c'est désormais **lui** la référence pour la topologie de déploiement et l'ingress TLS (voir §4.6).

### 4.6 ADR-019 — reformulation de ADR-001 et ADR-007 (2026-10-22)

Décision de `@architect`, carte `t_107d5ece`. Les deux formulations étaient **fausses dans les faits**
(§4.5) et servaient d'arguments de rejet (`architect/SOUL.md:177`), ce qui les rendait inopposables.

**Tranché : `Reformulée`, pas `Obsolète`.** La garde-fou de simplicité n'est pas supprimée, elle est
**recadrée sur une contrainte mesurable** :

| ADR | Statut | Remplacé par |
|---|---|---|
| **001** — « Single Docker container deployment » | `Superseded` | ADR-019 §1 (3 services) + §2 (budget RAM/CPU, pas un décompte de conteneurs) |
| **007** — « Nginx Proxy Manager for LAN/WAN TLS » | `Superseded` | ADR-019 §3 (TLS LAN = `nginx-local`) + §4 (TLS WAN = reverse proxy externe, hors périmètre) |

Pourquoi pas « Obsolète » : la contrainte d'**installabilité** est toujours nécessaire ; la supprimer
laisserait le registre sans ancre. Pourquoi pas « recréé rétroactivement » : décision §4.3 (contexte
perdu). L'ADR-019 est une **nouvelle** décision, prise sur des **mesures**, qui supersède les deux
formulations — elle ne les recrée pas.

**Point de vigilance** : la formulation « aucun artefact ne référence NPM » est **imprécise** (NPM
apparaît dans `README.md`, `AGENTS.md`, `docs/nginx-local.md`, `.hermes/roles/*.md` comme
*recommandation WAN*). Ce qui est faux, c'est « NPM est le proxy de Nook » — pas « NPM est cité ».
L'alignement des doc concernées est suivi en §7 (action 9).

### 4.6 Récupération d'ADR-017 (2026-10-22) — provenance vérifiée

L'ADR-017 a été rédigée le **2026-08-27** dans le workspace kanban `t_37a99925`, puis **jamais commitée** ;
le workspace a été nettoyé et le fichier a disparu du disque (carte `t_db6f9481`). Le texte a été
**retrouvé à l'identique** dans un artefact daté, pas réécrit :

| Élément | Valeur |
|---|---|
| Source | dump de session du profil `coder` : `profiles/coder/sessions/request_dump_20260828_171413_7f00b8_20260828_171601_168402.json` |
| Date de la source | 2026-08-28 17:16 UTC (cohérent avec la date de l'ADR, 2026-08-27) |
| Forme | résultat brut d'un `read_file` (430 lignes) de l'ADR, tel qu'il existait alors sur disque |
| Différence avec le texte commité | **une seule ligne** : `Status: Proposé` → `Accepté` |
| Preuve de la décision | `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415` (validation même conversation), `:769` (SET NULL), `:812` (JOIN preview), `frontend/src/lib/chatStore.svelte.ts:558`, commit `eca8093e` |

**Revue Gate 2 (schéma DB) — approuvée** par `@architect` le 2026-10-22 : la décision est implémentée
de bout en bout (migration, handlers, store, UI), le statut passe donc de `Proposé` à `Accepté` sur
preuve de code, conformément à §5. Aucune réécriture du contenu : seul le statut a été acté.

**Écart d'implémentation consigné** (sans conséquence sur la décision) : la migration 024 ajoute un index
`idx_messages_reply_to` qui n'était pas prévu par la spécification §1 de l'ADR. Il s'agit d'une optimisation
de lecture (`get_conversation_messages`), cohérente avec l'Option B.

---

## 5. Format d'un ADR

Deux formats coexistent ; le second est le format de travail de l'Architect (`.hermes/roles/architect.md:33`).

**Format minimal (historique, ADR-011/012)**
```markdown
# ADR-NNNN: Titre
**Date**: YYYY-MM-DD
**Status**: Proposed | Accepted | Rejected | Superseded
**Owner**: architect
## Context
## Decision
## Consequences
## Usage / Verification
```

**Format de travail (architecte)**
```markdown
# ADR-NNNN : Titre
**Date** : YYYY-MM-DD · **Status** : Proposé / Accepté / Rejeté · **Supersede** : ADR-XXXX
## Contexte
## Options considérées  (A. / B. / C. / D. — avec rejet motivé)
## Décision
## Conséquences (+ / − / → agents impactés)
## Plan d'implémentation
## Voir aussi (RFC, migration, DT, bugs)
```

Exigences non négociables :
- Chaque ADR cite au moins **une ligne de code** (`← Code:`) et **un artefact** (migration, workflow, DT ou RFC).
- Le statut `Proposé` passe à `Accepté` **uniquement** après revue par les reviewers du Gate concerné.
- Un ADR `Accepté` n'est jamais réécrit : on écrit un nouvel ADR qui le `Supersede`.

**Dérogation actée (ADR-016, 2026-10-22)** : un ADR déjà `Accepté` **peut** être réécrit pour **aligner le texte sur le réel** quand des réserves de revue sont closes par **mesure de code** — à condition que la décision elle-même ne change pas et que le tranchage soit consigné dans `DECISIONS.md`. C'est le cas d'ADR-016 : la décision (Option D) est inchangée ; seules la politique de rotation et la purge décrites ont été corrigées.

---

## 6. Numéros réservés

| Numéro | Réservé à | Raison |
|---|---|---|
| 001-010 | — | décisions pré-ADR, **ne pas recréer** (§4.3) · **001 et 007 supersédés par ADR-019** (§4.6) |
| 013 | Dual message store (`writable` + `$state`) | **clos** : décision appliquée puis revertée (§4.4) |
| 014 | rand_core 0.6→0.10 + argon2 0.5→0.6 | **recréé** (Proposé, bloqué security-auditor) — §4.4 |
| 015 | sqlx 0.8→0.9 (compat Axum 0.8) | **Accepté** (spike 2026-10-22 : migration faisable 1 passe, Axum 0.8 confirmé) — §4.4 |
| 016 | E2EE groupe global (Option D) | occupé |
| 017 | Reply-to (citer / répondre) | occupé — texte récupéré du workspace `t_37a99925` (§4.6) |
| 018 | Mémoire par agent | occupé |
| 019 | Topologie de déploiement (supersede 001, 007) | occupé (§4.6) |

**Prochain numéro libre : 020.**

---

## 7. Actions restantes (hors périmètre de cette carte)

| # | Action | Profil | Bloquant ? |
|---|--------|--------|------------|
| 1 | ~~Renommer `ADR-016/017/018-*.md` → `016/017/018-*.md`~~ → **fait** (016/018 en `3c4e0ca8`, 017 avec la carte `t_0b0decb6`) | `@architect` | — |
| 2 | Corriger `architect/SOUL.md:36-52` : marquer 001-010 et 013-015 `Non documenté` / `Orphelin`, retirer le statut `Accepted` | `@architect` | Non |
| 3 | Corriger `architect/SOUL.md:80` (DT-13 « DONE ») : les ADR 013-015 n'ont jamais été commités | `@architect` | Non |
| 4 | ~~Décider de ADR-001 (conteneur unique)~~ → **FAIT le 2026-10-22** : `Superseded` → [ADR-019](019-deployment-topology.md) (carte `t_107d5ece`) | `@architect` | — |
| 5 | ~~Décider de ADR-007 (Nginx Proxy Manager)~~ → **FAIT le 2026-10-22** : `Superseded` → [ADR-019](019-deployment-topology.md) (carte `t_107d5ece`) | `@architect` | — |
| 6 | ~~Rouvrir ou clore les ADR 013/014/015~~ → **fait** (carte `t_566014c5`) : 013 clos, 014/015 recréés, migration chacha 016 clos | `@architect` | — |
| 7 | ~~Levée des 2 réserves d'ADR-016~~ → **faite** le 2026-10-22 (`t_08632b95`) : réserves **assumées** (mesure de code), ADR-016 repassé `Accepté`, tranchage dans `DECISIONS.md` | `@architect` | — |
| 8 | ~~Récupérer le texte d'ADR-017~~ → **fait** : texte authentique retrouvé dans le dump de session coder du 2026-08-28, commité sous `017-reply-to-feature.md` (§4.6) | `@architect` | — |
| 9 | Débloquer ADR-014 : revue `security-auditor` du passage argon2 0.5→0.6 | `@security-auditor` | Non |
| 10 | Débloquer ADR-015 : spike coder de compat sqlx 0.9 / Axum 0.8 | `@coder` ✔ `t_60a24c82` (2026-10-22) | Non |
| 11 | **Job de rotation** : brancher `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`) ou la retirer (dead code) + **contrôle de rôle admin** sur `distribute_group_keys` / `add_member_key` / `get_my_group_key` (RFC-016 §5.4 non appliqué) — écart **E1** de `DECISIONS.md` | `@coder` | Non |
| 12 | **Purge des anciennes versions** de clé de groupe + **correctif `distribution_status`** dans `add_member_key` (`backend/src/e2ee.rs:780`) — écarts **E2/E3** de `DECISIONS.md` · ⚠️ une purge seule ne ferme pas l'accès post-départ : vérifier l'appartenance **à la lecture** | `@coder` ✔ `07ac1ad0` (2026-10-22) | Non |
| 13 | Aligner les doc qui présentent NPM comme **le** proxy de Nook (LAN compris) → LAN = `nginx-local`, WAN = reverse proxy externe (`AGENTS.md`, `.hermes/rules/*`, template SOUL) | `@docs-writer` | Non |

---

## 8. Graphe des liens

```
README.md ──► docs/adr/README.md (cet index)
                  │
                  ├──► 011-team-structure-decisions.md ──► .hermes/profiles/
                  ├──► 012-sfu-relay-capacity.md       ──► backend/src/config.rs:22
                  ├──► 016-e2ee-group-hybrid.md        ──► docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md
                  │                    │
                  │                    └──► DECISIONS.md ──► 2 cartes @coder (§7 #9, #10)
                  ├──► 017-reply-to-feature.md         ──► backend/migrations/024_reply_to.sql, backend/src/db.rs:415
                  ├──► 018-agent-memory-storage.md     ──► .hermes/plur/, .hermes/profiles/*/memories/
                  └──► 019-deployment-topology.md       ──► docker-compose.yml:2,58,88 · nginx-local.conf

.hermes/profiles/architect/SOUL.md:36  ──► cet index (source of truth)
.hermes/profiles/architect/SOUL.md:177 ──► « Add microservice → Rejected. ADR-001 » → désormais ADR-019 §2 (budget, pas décompte)
.hermes/roles/architect.md:25          ──► processus ADR (template §5)
```

---

*Maintenu par `@docs-writer` · Validé par `@architect` · Révision : à chaque nouvel ADR, ou tous les 6 mois.*
