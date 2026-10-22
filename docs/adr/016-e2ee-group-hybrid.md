# ADR-016 : E2EE groupe global — Hybrid Sender Keys (Option D)

**Date** : 2026-08-24  
**Session** : 492  
**Status** : **Accepté avec réserves** (implémenté ; 2 réserves non levées — voir note de gouvernance ci-dessous)  
**Décideur** : architect  
**Supersede** : — (premier ADR sur l'architecture E2EE de groupe)  
**Réserve** : revue `security-auditor` `ACCEPTÉ AVEC RÉSERVES MAJEURES` (carte kanban `t_0fc7cc3f`, 2026-08-24)  
**Traçabilité** : RFC-016 `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md` · migrations `019_conversation_keys.sql`, `022_distribution_status.sql`, `023_group_key_version.sql` · commits `ad499a3a` (SEC-FIX-1), `3b24583f` (SEC-FIX-2), `7bf46454` (SEC-FIX-3), `79edfc94` (E2EE-IMPL) · bugs `BUG-005`, `BUG-006` · dette `DT-05`

> **Note de gouvernance (2026-10-22, `@docs-writer`, carte `t_db6f9481`)**
> Le statut de cet ADR a été relevé de `Proposé → en attente d'acceptation` à `Accepté avec réserves` sur la base de **preuves de code**, pas sur une approbation formelle supplémentaire :
> - `backend/src/e2ee.rs:590-1010` implémente les 3 routes SEC-FIX-3 (`claim-key`, `my-key-status`, `group-key-metadata`) + `distribution_status` ;
> - `frontend/src/lib/e2ee.ts:189` implémente `rotateGroupKey()` ;
> - `backend/src/main.rs:498-502` applique un rate limiter dédié aux routes E2EE (SEC-FIX-2) ;
> - `ad499a3a` valide l'appartenance à la conversation avant insertion (SEC-FIX-1).
>
> **Réserves encore ouvertes** (aucun artefact ne les lève) :
> 1. **Rotation automatique** : `rotateGroupKey()` existe côté frontend mais aucun job backend ni config `group_key_rotation_days` n'existe — la rotation 7j reste manuelle.
> 2. **Purge des anciennes versions** : pas de mécanisme d'expiry ; `get_my_group_key` accepte n'importe quelle `version`, y compris expirée.
>
> Corrigé dans `docs/adr/README.md` §4.5 et §7 (actions 6-7).

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

**Inconvénients** : seule la groupe global serait concernée, les DMs/groupes privés garderaient l'enveloppe par destinataire. Pas de "hybridation".

**Rejeté au profit de l'Option D** qui généralise ce modèle avec une politique de rotation claire et un support explicite des membres sans clé.

### C — Double enveloppe (group key + enveloppe par destinataire)

Chaque message est chiffré deux fois : une fois avec la Global Group Key, une fois avec l'enveloppe par destinataire. Stockage doublé.

**Rejeté** : complexité inutile, stockage doublé, pas de benefice clair.

### D — Hybrid Sender Keys (choisie)

- **DMs et groupes privés** (`is_group=0` ou `is_group=1` non-global) : garder l'enveloppe par destinataire (`crypto.ts` `encryptForRecipients`).
- **Groupe global (`default_global`)** : utiliser une **Global Group Key** (XChaCha20-Poly1305 avec la clé de groupe). Distribution à chaque membre via `crypto_box_seal`. Rotation time-based (7j) + event-based (ajout/retrait de membre).
- **Membres sans clé publique** : distribution différée (status `pending`) + message explicite + template "setup E2EE puis reclamer la clé".
- **Compatibilité** : les DMs/groupes privés existants conservent leur modèle actuel. Seul le groupe global migre.

## Décision

**Option D — Hybrid Sender Keys** retenue pour le groupe global Nook.

Justification :

1. **Adapté au cas d'usage** : le groupe global est par définition un groupe grand ouvert avec des arrivées/départs fréquents — le modèle d'enveloppe par destinataire est structurellement inadapté.
2. **Fondations déjà en place** : Nook a déjà les tables `conversation_keys` + `conversation_key_recipients` (migration 019), les routes `distribute_group_keys`, `get_my_group_key`, `add_member_key` dans `e2ee.rs`, et la classe `E2EE` dans `e2ee.ts` avec `distributeGroupKey()`, `rotateGroupKey()`, `addMemberToConversation()`.
3. **Compatibilité préservée** : les DMs et groupes privés conservent l'enveloppe par destinataire, modèle optimal pour un petit nombre de destinataires connus.
4. **Forward secrecy partielle** : la rotation de clé de groupe empêche les nouveaux membres (après rotation) d'accéder à l'historique pré-rotation, ce qui est souhaitable pour la confidentialité.
5. **Gestion explicite des membres sans clé** : au lieu du silence actuel, les membres sans clé publique ont un statut `pending` et un chemin clair vers le setup E2EE.

## Conséquences

### +

- Messages du groupe global accessibles à tous les membres avec clé (pas de 404 `my-encrypted-key`).
- Nouveaux membres peuvent recevoir la clé de groupe et déchiffrer les messages futurs (et l'historique si la clé n'a pas été rotée depuis leur arrivée).
- Bots et comptes sans clé : comportement explicite (pending + message guidant vers setup E2EE).
- Rotation automatique avec politique 7j + événementielle.
- Les DMs/groupes privés ne sont pas impactés.

### -

- Migration du groupe global vers la group key : les messages existants (chiffrés avec l'enveloppe par destinataire) ne peuvent pas être ré-chiffrés. Le frontend doit supporter deux formats de messages chiffrés : ancien (par destinataire) et nouveau (group key).
- La rotation casse l'accès à l'historique pour les membres qui ont quitté avant la rotation (comportement voulu, mais à documenter).
- Nécessite une extension du schéma `conversation_key_recipients` avec une colonne `distribution_status` (`delivered` | `pending` | `failed`).
- Nécessite de nouvelles routes API : `GET /api/conversations/{conv_id}/my-key-status`, `POST /api/conversations/{conv_id}/claim-key`, `GET /api/conversations/{conv_id}/group-key-metadata`.

### → Agents impactés

- **coder** : implémentation frontend (`cryptoStore.svelte.ts` + `chatStore.svelte.ts` pour supporter les deux formats de messages chiffrés, `e2ee.ts` pour le cas membres sans clé).
- **security-auditor** : revue des routes E2EE group key, validation de la politique de rotation, audit du modèle de diffusion.
- **researcher** : benchmark du modèle Signal/Matrix pour confirmer les paramètres de rotation (7j, version antiquity).

## Implémentation — plan détaillé

### Schéma SQL (extension)

```sql
-- Dans conversation_key_recipients, ajouter :
--   distribution_status TEXT NOT NULL DEFAULT 'delivered'
--   valeurs : 'delivered' | 'pending' | 'failed'
```

### Routes API (nouvelles)

| Méthode | Route | Rôle |
|---------|-------|------|
| GET | `/api/conversations/{conv_id}/my-key-status` | Statut de distribution pour le membre courant |
| POST | `/api/conversations/{conv_id}/claim-key` | Réclamer sa clé après setup E2EE (statut pending) |
| GET | `/api/conversations/{conv_id}/group-key-metadata` | Metadata : version actuelle, dernière rotation, politique |

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

### Backend (`e2ee.rs`)

- `distribute_group_keys` : ajouter le champ `distribution_status` dans l'INSERT.
- Nouvelle route `claim_key` : si le membre a un statut `pending`, distribuer la clé courante (ou la dernière version disponible).
- Nouvelle route `my_key_status` : retourner `delivered`/`pending`/`failed` pour le membre courant.

### Politique de rotation

- **Time-based** : toutes les 7 jours (configurable `group_key_rotation_days` dans `config.rs`).
- **Event-based** : ajout/retrait de membre.
- Déclencheur : créateur/admin du groupe (pour `default_global`, seul l'admin).
- Expiry des anciennes clés : garder les 3 dernières versions (21j d'historique déchiffrable).

## Questions ouvertes

1. **Historique post-rotation** : les nouveaux membres (après rotation) ne peuvent pas déchiffrer l'historique pré-rotation. Acceptable ? → Oui (confidentialité).
2. **Expiry des anciennes versions** : 3 versions (21j) — à confirmer avec security-auditor.
3. **Group key pour les DMs** : de côté pour maintenant (pas de benefice clair pour un DM à 2 participants).

## Voir aussi

- RFC-016 : [URL relative : `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md`]
- Migration 019 : `backend/migrations/019_conversation_keys.sql`
- BUG-005 : `registerPublicKeyOnServer` fire-and-forget
- BUG-006 : `encryptForRecipients` catch silencieux
- DT-05 : E2EE partiellement implémenté

---

*ADR-016 — architect, session 492, 2026-08-24. À convertir en "Accepté" après revue security-auditor.*
