# ADR-016 : E2EE groupe global — Hybrid Sender Keys (Option D)

**Date** : 2026-08-24 (réécrit le 2026-10-22 pour décrire la politique **réelle**)
**Session** : 492
**Status** : **Accepté** (les 2 réserves de la revue sont **assumées** et closes — voir §Écarts de spécification)
**Décideur** : architect
**Supersede** : — (premier ADR sur l'architecture E2EE de groupe)
**Réserve** : revue `security-auditor` `ACCEPTÉ AVEC RÉSERVES MAJEURES` (carte kanban `t_0fc7cc3f`, 2026-08-24) — **les 2 réserves sont closes** (tranchage : `docs/adr/DECISIONS.md`, carte `t_08632b95`, 2026-10-22)
**Traçabilité** : RFC-016 `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md` · tranchage `docs/adr/DECISIONS.md` · migrations `019_conversation_keys.sql`, `022_distribution_status.sql`, `023_group_key_version.sql` · commits `ad499a3a` (SEC-FIX-1), `3b24583f` (SEC-FIX-2), `7bf46454` (SEC-FIX-3), `79edfc94` (E2EE-IMPL) · bugs `BUG-005`, `BUG-006` · dette `DT-05`

> **Note de gouvernance (2026-10-22, `@architect`, carte `t_08632b95`)**
> Ce document a été relevé de `Accepté avec réserves` à **`Accepté`** le 2026-10-22. Les deux réserves de la revue `security-auditor` ont été **tranchées par mesure de code** (`docs/adr/DECISIONS.md`) : elles sont **assumées** — la politique **réelle** est écrite ici, à la place de la politique spécifiée.
>
> **Les 3 CRITIQUES de la revue sont levées en code** (vérifié, non supposé) :
> - `SEC-FIX-1` : `add_member_key` valide `body.userId` → 403 si non-membre (`backend/src/e2ee.rs:759-776`) ;
> - `SEC-FIX-2` : rate limiter dédié aux routes E2EE (`backend/src/main.rs:501`, `backend/src/main.rs:612`) ;
> - `SEC-FIX-3` : `claim-key` (`backend/src/e2ee.rs:834`), `my-key-status`, `group-key-metadata` (`backend/src/e2ee.rs:961`) + `distribution_status` (migration 022).
>
> **Ce qui change dans ce document** : §Politique de rotation (rotation **manuelle**, pas de job), §Écarts de spécification (3 écarts actés), §Questions ouvertes (expiry **absent**, pas « à confirmer »).

---

## Contexte

Le groupe global Nook (`default_global`, `is_group=1`, tous les utilisateurs approuvés sont participants) chiffre tous les messages. L'architecture E2EE actuelle utilise le modèle **enveloppe par destinataire** : chaque message est chiffré avec une clé de session éphémère (`crypto_secretbox_easy`), et la clé de session est chiffrée pour chaque destinataire avec sa clé publique X25519 (`crypto_box_easy`) → stockée dans `message_keys`.

Ce modèle, bien que correct pour les DMs et les groupes privés de petite taille, présente des problèmes structurels pour le groupe global :

- **BUG-005** : `registerPublicKeyOnServer` est fire-and-forget dans `cryptoStore.svelte.ts` (ligne 338-340) — la clé publique peut ne pas être enregistrée avant l'envoi suivant, excluant le membre de `fetchMemberPubkeys` → 404 `my-encrypted-key`.
- **BUG-006** : `encryptForRecipients` catch silencieux (crypto.ts lignes 149-151) — les destinataires sans clé sont skipés sans erreur ni notification.
- **Membres sans clé** (nouveaux utilisateurs, bots) sont systématiquement exclus du chiffrement.
- **Nouveaux membres** (arrivant après coup) ne peuvent pas déchiffrer l'historique.
- **Modèle incomplet** pour un groupe de taille dynamique.

L'enquête E2EE-ARCH (session 488) et RESEARCH-E2EE (session 491) ont identifié **Option D — Hybrid Sender Keys** (modèle Signal/Matrix/WhatsApp) comme solution, avec la validation que Nook possède déjà les fondations nécessaires (tables `conversation_keys` + `conversation_key_recipients` déployées via migration 019, routes E2EE group key dans `e2ee.rs`, `E2EE` class dans `e2ee.ts`).

## Options considérées

### A — Garder enveloppe par destinataire + erreurs explicites

Laisser le modèle actuel, afficher des erreurs quand un destinataire n'a pas de clé. Pas de changement d'architecture.

**Rejeté** : ne résout pas le problème fondamental — nouveaux membres sans accès à l'historique, escalabilité limitée, silence sur les membres sans clé.

### B — Global Group Key (une seule clé symétrique pour le groupe)

Une clé symétrique 32 bytes partagée par tous les membres. Chiffrement des messages avec cette clé. Distribution via `crypto_box_seal` à chaque nouveau membre. Rotation à chaque arrivée/départ.

**Avantages** : modèle simple, compatible Signal/Matrix, nouveaux membres peuvent recevoir la clé.

**Inconvénients** : seule la groupe global serait concernée, les DMs/groupes privés garderaient l'enveloppe par destinataire. Pas d'"hybridation".

**Rejeté au profit de l'Option D** qui généralise ce modèle avec une politique de rotation claire et un support explicite des membres sans clé.

### C — Double enveloppe (group key + enveloppe par destinataire)

Chaque message est chiffré deux fois : une fois avec la Global Group Key, une fois avec l'enveloppe par destinataire. Stockage doublé.

**Rejeté** : complexité inutile, stockage doublé, pas de benefice clair.

### D — Hybrid Sender Keys (choisie)

- **DMs et groupes privés** (`is_group=0` ou `is_group=1` non-global) : garder l'enveloppe par destinataire (`crypto.ts` `encryptForRecipients`).
- **Groupe global (`default_global`)** : utiliser une **Global Group Key** (XChaCha20-Poly1305 avec la clé de groupe). Distribution à chaque membre via `crypto_box_seal`. Rotation **manuelle** + event-based (ajout/retrait de membre).
- **Membres sans clé publique** : distribution différée (status `pending`) + message explicite + template "setup E2EE puis reclamer la clé".
- **Compatibilité** : les DMs/groupes privés existants conservent leur modèle actuel. Seul le groupe global migre.

## Décision

**Option D — Hybrid Sender Keys** retenue pour le groupe global Nook.

Justification :

1. **Adapté au cas d'usage** : le groupe global est par définition un groupe grand ouvert avec des arrivées/départs fréquents — le modèle d'enveloppe par destinataire est structurellement inadapté.
2. **Fondations déjà en place** : Nook a déjà les tables `conversation_keys` + `conversation_key_recipients` (migration 019), les routes `distribute_group_keys`, `get_my_group_key`, `add_member_key` dans `e2ee.rs`, et la classe `E2EE` dans `e2ee.ts` avec `distributeGroupKey()`, `rotateGroupKey()`, `addMemberToConversation()`.
3. **Compatibilité préservée** : les DMs et groupes privés conservent l'enveloppe par destinataire, modèle optimal pour un petit nombre de destinataires connus.
4. **Forward secrecy partielle** : la rotation de clé de groupe empêche les nouveaux membres (après rotation) d'accéder à l'historique pré-rotation, ce qui est souhaitable pour la confidentialité. *(Réserve : cf. §Écarts de spécification E2 — sans contrôle d'appartenance à la lecture, l'historique reste accessible après un départ.)*
5. **Gestion explicite des membres sans clé** : au lieu du silence actuel, les membres sans clé publique ont un statut `pending` et un chemin clair vers le setup E2EE.

## Conséquences

### +

- Messages du groupe global accessibles à tous les membres avec clé (pas de 404 `my-encrypted-key`).
- Nouveaux membres peuvent recevoir la clé de groupe et déchiffrer les messages futurs (et l'historique si la clé n'a pas été rotée depuis leur arrivée).
- Bots et comptes sans clé : comportement explicite (pending + message guidant vers setup E2EE).
- Rotation **manuelle** (déclenchée depuis l'UI par un membre) + event-based. Aucun déclenchement automatique n'existe.
- Les DMs/groupes privés ne sont pas impactés.

### -

- Migration du groupe global vers la group key : les messages existants (chiffrés avec l'enveloppe par destinataire) ne peuvent pas être ré-chiffrés. Le frontend doit supporter deux formats de messages chiffrés : ancien (par destinataire) et nouveau (group key).
- La rotation casse l'accès à l'historique pour les membres qui ont quitté avant la rotation (comportement voulu, mais à documenter).
- Nécessite une extension du schéma `conversation_key_recipients` avec une colonne `distribution_status` (`delivered` | `pending` | `failed`). **Fait** : migration 022.
- Nécessite de nouvelles routes API : `GET /api/conversations/{conv_id}/my-key-status`, `POST /api/conversations/{conv_id}/claim-key`, `GET /api/conversations/{conv_id}/group-key-metadata`. **Fait** : `backend/src/e2ee.rs:834` et `:961`.
- `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`) est aujourd'hui du **dead code** (zéro appelant) : soit elle est branchée à un déclencheur UI, soit elle est retirée. Voir `docs/adr/DECISIONS.md` §4.

### → Agents impactés

- **coder** : 2 cartes de suivi créées (`docs/adr/README.md` §7) — (1) job de rotation / dead code + contrôle de rôle admin ; (2) purge des anciennes versions + correctif `distribution_status` dans `add_member_key`.
- **security-auditor** : revue des routes E2EE group key (faite, `t_0fc7cc3f`) — les réserves sont closes ; ré-audit recommandé **après** livraison des 2 cartes ci-dessus.
- **researcher** : benchmark du modèle Signal/Matrix pour confirmer les paramètres de rotation (7j, version antiquity) — **non requis** tant que la rotation reste manuelle.

## Implémentation — plan détaillé

### Schéma SQL (extension) — **FAIT**

```sql
-- Migration 022 : conversation_key_recipients.distribution_status
--   TEXT NOT NULL DEFAULT 'delivered'
--   valeurs : 'delivered' | 'pending' | 'failed'
```

### Routes API (nouvelles) — **FAIT**

| Méthode | Route | Rôle | Preuve |
|---------|-------|------|--------|
| GET | `/api/conversations/{conv_id}/my-key-status` | Statut de distribution pour le membre courant | `backend/src/e2ee.rs` |
| POST | `/api/conversations/{conv_id}/claim-key` | Réclamer sa clé après setup E2EE (statut pending) | `backend/src/e2ee.rs:834` |
| GET | `/api/conversations/{conv_id}/group-key-metadata` | Metadata : version actuelle, dernière rotation, politique | `backend/src/e2ee.rs:961` |

### Frontend

**cryptoStore.svelte.ts** :

- Ajouter `groupKeyReady: boolean` et `groupKeyVersion: number` dans `CryptoStoreState`.
- `loadGroupKey(convId)` et `distributeGroupKey(convId, memberPubkeys)` appelant `e2ee.ts`.
- Pour `default_global` : utiliser la group key au lieu de `encryptForRecipients`.

**chatStore.svelte.ts** :

- `_decryptBatch` et `_injectMessage` supportent deux formats :
  1. Ancien format (enveloppe par destinataire) : `nonce` + `encrypted_keys` → `decryptMessage`.
  2. Nouveau format (group key) : `nonce` + `group_key_version` → déchiffrement avec la group key chargée.
- La distinction se fait sur les champs présents dans le message.

**e2ee.ts** :

- `distributeGroupKey` retourne une erreur explicite quand un membre n'a pas de clé publique (au lieu du console.warn silencieux).
- `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`) — **non branchée** (dead code), cf. §Écarts E1.

### Backend (`e2ee.rs`) — routes **FAITES**, rotation/purge **À FAIRE**

- `distribute_group_keys` : ajoute le champ `distribution_status` dans l'INSERT.
- Route `claim_key` : si le membre a un statut `pending`, distribuer la clé courante (ou la dernière version disponible). **Réserve** : `add_member_key` ne pose pas la colonne (§Écarts E3).
- Route `my_key_status` : retourne `delivered`/`pending`/`failed` pour le membre courant.
- **Absent** : job de rotation périodique ; contrôle de rôle admin ; purge des anciennes versions. → 2 cartes `@coder` (`docs/adr/README.md` §7).

### Politique de rotation — **réelle** (diffère de la spécification)

- **Time-based 7j : NON implémenté.** Aucun champ `group_key_rotation_days` (`backend/src/config.rs:5-23` — `Config` n'a pas ce champ), aucun job périodique (les seuls `tokio::spawn` périodiques sont `main.rs:443` fichiers et `main.rs:449` `prune_old_data`).
- **Rotation manuelle** : déclenchée par un appel à `rotateGroupKey()` (`frontend/src/lib/e2ee.ts:189`). Cette fonction est **dead code** aujourd'hui (zéro appelant) — la rotation n'est donc déclenchable par personne en l'état.
- **Event-based** : ajout/retrait de membre.
- **Déclencheur** : un membre authentifié. La règle RFC-016 §5.4 (« seul l'admin » pour `default_global`) **n'est pas appliquée** — cf. §Écarts E1.
- **Expiry des anciennes clés** : **aucun**. Les versions sont conservées indéfiniment et restent récupérables — cf. §Écarts E2.

## Écarts de spécification (actés le 2026-10-22)

Trois écarts entre cet ADR et le code ont été **mesurés** puis **assumés**. Détail et preuves : `docs/adr/DECISIONS.md`.

| # | Écart | Mesure | Décision |
|---|-------|--------|----------|
| **E1** | Pas de job de rotation, et **pas de contrôle de rôle** sur les routes E2EE | `rotateGroupKey()` = dead code (`frontend/src/lib/e2ee.ts:189`, zéro appelant) · aucun `group_key_rotation_days` (`backend/src/config.rs:5-23`) · **zéro** contrôle de rôle dans `backend/src/e2ee.rs` alors que le rôle existe (`backend/src/admin.rs:69` : `if user.role != "admin"`) | **Assumé** — rotation manuelle, n'importe quel membre peut distribuer · → carte `@coder` (brancher/retirer `rotateGroupKey()` + contrôle admin). **Portée corrigée le 2026-10-23** : le contrôle s'applique à `distribute_group_keys` et `add_member_key` **uniquement** — `get_my_group_key` est une **auto-lecture membre**, non concernée (tranchage `t_e8af977c`, `DECISIONS.md` §1-8) |
| **E2** | Aucune purge des anciennes versions ; `get_my_group_key` accepte **n'importe quelle** version | `MyKeyQuery { version: Option<i32> }` (`backend/src/e2ee.rs:130`) · `prune.rs` ne nettoie jamais `conversation_keys` / `conversation_key_recipients` · **aucune** expiration côté client (`frontend/src/lib/e2ee.ts`) | **Assumé** — conserve **illimitée**, **aucun** expiry (la revue §3.6 disait à tort « résolu côté client ») · → carte `@coder` (purge) |
| **E3** | `add_member_key` n'insère pas `distribution_status` → l'entrée naît `delivered` → `claim_key` ne la sert **jamais** | `INSERT` sans la colonne (`backend/src/e2ee.rs:780`) · défaut `'delivered'` (`backend/migrations/022_distribution_status.sql:4`) · `claim_key` ne sert que le `pending` (`backend/src/e2ee.rs:845`) | **Assumé** — cas « membre sans clé » structurellement hors d'atteinte par ce chemin · → carte `@coder` (correctif) |

**Conséquence de E1+E2 sur la confidentialité** : un membre **renvoyé** peut, tant que son compte existe, redemander n'importe quelle version antérieure de la clé et déchiffrer l'historique. C'est le contraire de la forward secrecy partielle revendiquée (RFC-016 §5.4:180). **Une purge seule ne ferme pas cet accès** — il faut vérifier l'appartenance **au moment de la lecture**. Point acté, hors périmètre docs.

## Questions ouvertes

1. **Rotation** : brancher `rotateGroupKey()` à un déclencheur UI, ou la retirer comme dead code ? → **Carte `@coder` #1** (`docs/adr/README.md` §7). Le contrôle de rôle admin (RFC-016 §5.4) est dans la même carte.
2. **Expiry** : garder les 3 dernières versions (21j), comme proposé en 2026-08 ? → **Non tranché** ; la politique réelle est « conserve illimitée » (E2). Toute purge doit être accompagnée d'un contrôle d'appartenance à la lecture, sinon elle ne réduit pas l'exposition.
3. **`add_member_key`** : poser `distribution_status = 'pending'` quand le membre n'a pas de clé publique ? → **Carte `@coder` #2** (E3).
4. **Group key pour les DMs** : de côté pour maintenant (pas de benefice clair pour un DM à 2 participants).

## Voir aussi

- **Tranchage des réserves** : `docs/adr/DECISIONS.md` (carte `t_08632b95`)
- RFC-016 : [URL relative : `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md`]
- Revue de sécurité : carte `t_0fc7cc3f` (`ACCEPTÉ AVEC RÉSERVES MAJEURES`)
- Migration 019 : `backend/migrations/019_conversation_keys.sql`
- Migration 022 : `backend/migrations/022_distribution_status.sql`
- BUG-005 : `registerPublicKeyOnServer` fire-and-forget
- BUG-006 : `encryptForRecipients` catch silencieux
- DT-05 : E2EE partiellement implémenté

---

*ADR-016 — architect, session 492, 2026-08-24. Réécrit le 2026-10-22 (carte `t_08632b95`) pour décrire la politique réelle et acter les 3 écarts mesurés — statut : **Accepté**.*
