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
| [ADR-016](ADR-016-e2ee-group-hybrid.md) | `ADR-016-e2ee-group-hybrid.md` | E2EE groupe global — Hybrid Sender Keys (Option D) | **Accepté avec réserves** (implémenté ; 2 réserves non levées) | 2026-08-24 | `backend/src/e2ee.rs:590`, `frontend/src/lib/e2ee.ts:189` |
| [ADR-017](ADR-017-reply-to-feature.md) | `ADR-017-reply-to-feature.md` | Reply-to (citer / répondre à un message) | **Accepté** | 2026-10 | `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415` |
| [ADR-018](ADR-018-agent-memory-storage.md) | `ADR-018-agent-memory-storage.md` | Stockage de la mémoire par agent — MEMORY.md + PLUR vs Mem0 vs Supermemory | **Proposé** (en attente décision Mo Ju) | 2026-10-22 | `.hermes/profiles/*/memories/`, `.hermes/plur/` |
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
| `ADR-016-e2ee-group-hybrid.md` | `016-e2ee-group-hybrid.md` | à renommer |
| `ADR-017-reply-to-feature.md` | `017-reply-to-feature.md` | à renommer |
| `ADR-018-agent-memory-storage.md` | `018-agent-memory-storage.md` | à renommer |
| `0013-dual-message-store-architecture.md` | `013-dual-message-store-architecture.md` | orphelin, §4 |
| `0014-rand_core-argon2-migration.md` | `014-rand-core-argon2-migration.md` | orphelin, §4 |
| `0015-sqlx-08-to-09-migration.md` | `015-sqlx-08-to-09-migration.md` | orphelin, §4 |
| `0016-chacha20poly1305-010-to-011-migration.md` | — | doublon de collision, §4.4 |

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

Le 2026-08-07, l'Architect a créé trois ADR de migration de dépendances **en partant du numéro 013** (carte `t_dd44fc37`), en supposant — à tort — que 011 et 012 étaient les deux seuls ADR existants et que 013 était le prochain numéro libre. Résultat :

| Numéro | Attribué à | Fichier (jamais commité) |
|---|---|---|
| 013 | rand_core 0.6→0.10 + argon2 0.5→0.6 | `0013-…` puis `014-rand_core-argon2-migration.md` (renommé) |
| 014 | sqlx 0.8→0.9 | `015-sqlx-08-to-09-migration.md` |
| 015 | chacha20poly1305 0.10→0.11 | `0016-chacha20poly1305-010-to-011-migration.md` |
| 016 | E2EE groupe global (Option D) | `ADR-016-e2ee-group-hybrid.md` |

**Un même numéro a failli désigner deux décisions différentes** (016 = migration de dépendance *ou* architecture E2EE). C'est la cause directe de l'incohérence actuelle.

**Sort décidé** : les trois ADR de migration (013, 014, 015) sont **orphelins** : écrits mais jamais commités, leurs trois dépendances sont **encore épinglées aux versions anciennes** (`backend/Cargo.toml` : `rand_core 0.6`, `argon2 0.5`, `sqlx 0.8.6`, `chacha20poly1305 0.10.1`) — donc la décision n'a pas abouti. Leurs numéros restent réservés dans ce tableau (§6) et ne seront pas réattribués.

Le numéro **016** appartient à l'ADR E2EE (option D), seul ADR réellement implémenté.

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
| 016 | E2EE groupe global — Hybrid Sender Keys | `ADR-016-e2ee-group-hybrid.md` | **Accepté avec réserves** | `backend/src/e2ee.rs:590-1010`, `frontend/src/lib/e2ee.ts:189` |
| 017 | Reply-to (citer/répondre) | `ADR-017-reply-to-feature.md` | **Accepté** | `backend/migrations/024_reply_to.sql`, `backend/src/db.rs:415` |
| 018 | Mémoire par agent (MEMORY.md vs Mem0 vs Supermemory) | `ADR-018-agent-memory-storage.md` | **Proposé** | `.hermes/profiles/*/memories/`, `.hermes/plur/` |

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

---

## 6. Numéros réservés

| Numéro | Réservé à | Raison |
|---|---|---|
| 001-010 | — | décisions pré-ADR, **ne pas recréer** (§4.3) · **001 et 007 supersédés par ADR-019** (§4.6) |
| 013, 014, 015 | ADR de migration de dépendances | orphelins, décision non aboutie (§4.4) |
| 016 | E2EE groupe global (Option D) | occupé |
| 017 | Reply-to | occupé |
| 018 | Mémoire par agent | occupé |
| 019 | Topologie de déploiement (supersede 001, 007) | occupé (§4.6) |

**Prochain numéro libre : 020.**

---

## 7. Actions restantes (hors périmètre de cette carte)

| # | Action | Profil | Bloquant ? |
|---|--------|--------|------------|
| 1 | Renommer `ADR-016/017/018-*.md` → `016/017/018-*.md` (convention §3) | `@architect` | Non |
| 2 | Corriger `architect/SOUL.md:36-52` : marquer 001-010 et 013-015 `Non documenté` / `Orphelin`, retirer le statut `Accepted` | `@architect` | Non |
| 3 | Corriger `architect/SOUL.md:80` (DT-13 « DONE ») : les ADR 013-015 n'ont jamais été commités | `@architect` | Non |
| 4 | ~~Décider de ADR-001 (conteneur unique)~~ → **FAIT le 2026-10-22** : `Superseded` → [ADR-019](019-deployment-topology.md) (carte `t_107d5ece`) | `@architect` | Non |
| 5 | ~~Décider de ADR-007 (Nginx Proxy Manager)~~ → **FAIT le 2026-10-22** : `Superseded` → [ADR-019](019-deployment-topology.md) (carte `t_107d5ece`) | `@architect` | Non |
| 6 | Rouvrir ou clore les ADR 013/014/015 (migrations de deps encore bloquées) | `@architect` + `@security-auditor` | Non |
| 7 | Levée des 2 réserves d'ADR-016 (rotation de clé, purge des anciennes versions) | `@architect` + `@security-auditor` | Non |
| 8 | Récupérer le texte d'ADR-017 (rédigé dans le workspace `t_37a99925`, jamais commité) et le commiter | `@architect` | Non |
| 9 | Aligner les doc qui présentent NPM comme **le** proxy de Nook (LAN compris) → LAN = `nginx-local`, WAN = reverse proxy externe : `AGENTS.md:47`, `.hermes/rules/architecture.md:163`, `.hermes/rules/workflows.md:39`, `.hermes/rules/frontend-and-business.md:176`, `.hermes/rules/debug-and-glossary.md:209`, `.hermes/templates/profile-template/SOUL.md:48` | `@docs-writer` | Non |

---

## 8. Graphe des liens

```
README.md ──► docs/adr/README.md (cet index)
                  │
                  ├──► 011-team-structure-decisions.md ──► .hermes/profiles/
                  ├──► 012-sfu-relay-capacity.md       ──► backend/src/config.rs:22
                  ├──► 016-e2ee-group-hybrid.md        ──► docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md
                  ├──► 017-reply-to-feature.md         ──► backend/migrations/024_reply_to.sql
                  ├──► 018-agent-memory-storage.md     ──► .hermes/plur/, .hermes/profiles/*/memories/
                  └──► 019-deployment-topology.md       ──► docker-compose.yml:2,58,88 · nginx-local.conf

.hermes/profiles/architect/SOUL.md:36  ──► cet index (source of truth)
.hermes/profiles/architect/SOUL.md:177 ──► « Add microservice → Rejected. ADR-001 » → désormais ADR-019 §2 (budget, pas décompte)
.hermes/roles/architect.md:25          ──► processus ADR (template §5)
```

---

*Maintenu par `@docs-writer` · Validé par `@architect` · Révision : à chaque nouvel ADR, ou tous les 6 mois.*
