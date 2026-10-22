# RFC-016 : E2EE groupe global — Migration vers Hybrid Sender Keys (Option D)

**Auteur** : architect (session 492)  
**Date** : 2026-08-24  
**Statut** : Proposition — en attente d'acceptation  
**Concerne** : `frontend/src/lib/e2ee.ts`, `frontend/src/lib/crypto.ts`, `frontend/src/lib/cryptoStore.svelte.ts`, `backend/src/e2ee.rs`, `backend/src/db.rs`, `backend/migrations/019_conversation_keys.sql` (déjà déployé)

---

## 1. Problème

Le groupe global Nook (`default_global`, `is_group=1`) chiffre tous les messages, mais l'architecture E2EE actuelle — **enveloppe par destinataire** (crypto.ts `encryptForRecipients` + `message_keys`) — est incompatible avec un groupe ouvert :

1. **Membres sans clé publique** : `fetchMemberPubkeys` (backend `get_member_public_keys`) filtre `WHERE u.public_key IS NOT NULL`. Un utilisateur sans clé enregistrée est exclu de la réponse → `encryptForRecipients` ne peut pas lui chiffrer → le message reste inaccessible pour lui.
2. **BUG-005** : `registerPublicKeyOnServer` est un **fire-and-forget** (pas de `await` dans `cryptoStore.svelte.ts` ligne 338-340). La clé publique peut ne pas être enregistrée côté serveur lors de l'envoi suivant, donc elle n'apparaît pas dans `fetchMemberPubkeys`, donc les destinataires reçoivent une 404 (`/api/conversations/{conv_id}/my-encrypted-key/{msg_id}`) — le bug "404 my-encrypted-key" rapporté.
3. **BUG-006** : `encryptForRecipients` catch silencieux (ligne 149-151 crypto.ts) : si un destinataire échoue, on skip sans erreur, sans fallback, sans notification.
4. **Arrivée post-coup** : un utilisateur qui rejoint le groupe après que des messages ont été envoyés ne peut pas déchiffrer l'historique — chaque ancien message devrait lui être ré-chiffré (impossible en E2EE par définition).
5. **Bots et comptes sans clé** : le groupe global permet des participants qui n'ont pas de clé E2EE (comptes désactivés, bots). Avec l'enveloppe par destinataire, ces participants sont systématiquement exclus.

**En résumé** : le modèle "un message, N enveloppes à N clés publiques" est le bon choix pour les DMs et groupes privés fermés, mais il échoue pour un groupe grand ouvert, dynamique, où les membresarrivent et partent.

---

## 2. Options considérées

### Option A — Garder l'enveloppe par destinataire + fallback clair

On laisse le modèle actuel, mais on affiche une erreur explicite quand un destinataire n'a pas de clé. Pas de changement d'architecture. **Rejeté** : ne résout pas le problème fondamental (historique inaccessible pour nouveaux membres, impossible d'échelonner).

### Option B — Chiffrement par groupe avec une seule clé symétrique (Global Group Key)

Une seule clé symétrique (XChaCha20-Poly1305 ou AES-256-GCM) est partagée par tous les membres actuels. Les messages sont chiffrés avec cette clé. Quand un membre arrive, le créateur lui envoie la clé (chiffrée avec sa clé publique via `crypto_box_seal`). Quand un membre part, la clé est rotée.

Cette option existe **déjà partiellement** dans Nook :

- `backend/migrations/019_conversation_keys.sql` a déjà les tables `conversation_keys` + `conversation_key_recipients`.
- `backend/src/e2ee.rs` a déjà les routes `distribute_group_keys`, `get_my_group_key`, `add_member_key`.
- `frontend/src/lib/e2ee.ts` a déjà `E2EE.distributeGroupKey()`, `addMemberToConversation()`, `rotateGroupKey()`, `loadGroupKey()`.

**Ce qui manque** : le groupe `default_global` n'utilise **pas** cette infrastructure — il utilise toujours `crypto.ts` enveloppe par destinataire.

**Avantages** : compatible avec Signal/Matrix/WhatsApp ; permet l'arrivée de nouveaux membres (ils reçoivent la clé courante) ; rotation explicite ; messagerie de groupe cohérente.

**Inconvénients** : la rotation casse l'accès à l'historique pour les membres qui ont quitté avant la rotation (ce qui est le comportement désiré pour la confidentialité). Nécessite une politique de rotation claire.

### Option C — Double enveloppe (group key + enveloppe par destinataire)

Chaque message est chiffré deux fois : une fois avec la Global Group Key (pour tous les membres actuels), une fois avec l'enveloppe par destinataire (pour les DMs privés). **Rejeté** : complexité doublée, stockage doublé dans `message_keys`, pas de benefice clair.

### Option D — Hybrid Sender Keys (choisie)

Modèle de type Signal/Matrix/WhatsApp :

1. **DMs et groupes privés** (`is_group=0` ou `is_group=1` avec liste restreinte) : garder l'enveloppe par destinataire actuelle. C'est le modèle optimal pour un petit nombre de destinataires connus.
2. **Groupe global (`default_global`)** : utiliser une **Global Group Key** (clef symétrique 32 bytes, rotable). Les messages sont chiffrés avec cette clé (XChaCha20-Poly1305, identique à `crypto_secretbox_easy` mais avec la group key). La clé est distribuée à chaque membre via `crypto_box_seal` utilisant la clé publique X25519 du membre.
3. **Rotation** : 
   - **Time-based** : toutes les 7 jours (configurable).
   - **Event-based** : ajout ou retrait d'un membre.
   - La version de la clé est stockée dans `conversation_keys.version`. L'historique des versions précédentes est conservé (pour le déchiffrement des anciens messages par les membres qui ont rejoint avant la rotation).
4. **Membres sans clé publique** : si un membre n'a pas encore de clé E2EE (nouvel utilisateur, bot), la distribution de la group key est **différée** — on stocke la clé de groupe pour ce membre avec un statut "pending" et on lui affiche un message "Configurez E2EE pour accéder aux messages chiffrés". Le template d'onboarding guide l'utilisateur vers le setup E2EE, puis il peut réclamer sa clé.

---

## 3. Décision

**Option D retenue** : Hybrid Sender Keys.

**Justification** :

- Nook a déjà les fondations (tables, routes, `e2ee.ts`) donc l'effort d'implémentation est faible.
- Le groupe global est par définition un groupe grand ouvert avec arrivées/départs fréquents — le modèle d'enveloppe par destinataire est structurellement inadapté.
- La compatibilité avec les DMs/groupes privés est préservée (enveloppe par destinataire).
- La rotation de clé de groupe permet une forward secrecy partielle (nouveaux membres après rotation n'ont pas accès à l'historique pré-rotation, ce qui est souhaitable).
- Les bots et comptes sans clé peuvent être gérés explicitement (différé + message explicite).

---

## 4. Conséquences

### + Ce que ça améliore

- Messages du groupe global accessibles à tous les membres actuels (pas de 404 my-encrypted-key).
- Nouveaux membres peuvent recevoir la clé de groupe et déchiffrer les messages futurs (et l'historique si la clé n'a pas été rotée depuis leur arrivée).
- Bots et comptes sans clé : comportement explicite au lieu de silencieux.
- Rotation automatique avec politique 7j + événementielle.

### - Ce que ça complique

- La rotation de clé casse l'accès à l'historique pour les membres qui ont quitté avant la rotation — c'est voulu, mais doit être documenté.
- La migration du groupe global vers la group key nécessite un point de migration : les messages existants chiffrés avec l'enveloppe par destinataire ne peuvent pas être ré-chiffrés avec la group key (les anciens messages restent dans leur format). Le frontend doit supporter deux formats de messages chiffrés : l'ancien (par destinataire) et le nouveau (group key).
- Nécessite une nouvelle route API pour distribuer la group key au membre "pending" après setup E2EE.

### → Agents impactés

- **coder** : implémentation frontend (e2ee.ts déjà prêt, cryptoStore.svelte.ts à adapter pour utiliser la group key pour default_global, chatStore.svelte.ts pour le déchiffrement des deux formats).
- **security-auditor** : revue des routes E2EE group key, validation de la politique de rotation, audit du modèle de diffusion.
- **researcher** : benchmark du modèle Signal/Matrix pour confirmer les paramètres de rotation.

---

## 5. Plan d'implémentation (hors code — architecture)

### 5.1 Schéma SQL (déjà présent — à valider)

Tables existantes dans `migrations/019_conversation_keys.sql` :

```sql
CREATE TABLE conversation_keys (
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    creator_id      TEXT NOT NULL REFERENCES users(id),
    created_at      INTEGER NOT NULL,
    PRIMARY KEY (conversation_id, version)
);

CREATE TABLE conversation_key_recipients (
    conversation_id     TEXT NOT NULL,
    version             INTEGER NOT NULL,
    user_id             TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    encrypted_key       TEXT NOT NULL,  -- base64 sealed box
    user_key_version    INTEGER NOT NULL,
    FOREIGN KEY (conversation_id, version) REFERENCES conversation_keys(conversation_id, version) ON DELETE CASCADE,
    PRIMARY KEY (conversation_id, version, user_id)
);
```

**Proposition d'extension** (pour supporter les membres "pending") :

```sql
-- Statut de distribution de la clé de groupe pour un membre
ALTER TABLE conversation_key_recipients ADD COLUMN distribution_status TEXT NOT NULL DEFAULT 'delivered';
-- valeurs : 'delivered' | 'pending' | 'failed'
```

Cette colonne permet de tracker si la clé a été distribuée avec succès, différée (membre sans clé publique), ou en échec.

### 5.2 Routes API

Routes existantes (déjà dans `e2ee.rs`) :

| Méthode | Route | Rôle |
|---------|-------|------|
| POST | `/api/conversations/{conv_id}/keys` | Distribuer une nouvelle group key aux membres |
| GET | `/api/conversations/{conv_id}/my-key` | Récupérer la group key chiffrée pour le membre courant |
| POST | `/api/conversations/{conv_id}/add-member-key` | Ajouter un membre à une group key existante |

**Nouvelles routes proposées** :

| Méthode | Route | Rôle |
|---------|-------|------|
| GET | `/api/conversations/{conv_id}/my-key-status` | Statut de distribution de la clé pour le membre courant (delivered/pending) |
| POST | `/api/conversations/{conv_id}/claim-key` | Réclamer sa clé de groupe après setup E2EE (si status = pending) |
| GET | `/api/conversations/{conv_id}/group-key-metadata` | Metadata de la group key : version actuelle, date de dernière rotation, politique de rotation |

### 5.3 Camps frontend à modifier

**cryptoStore.svelte.ts** :

- Ajouter un champ `groupKeyReady: boolean` et `groupKeyVersion: number` dans `CryptoStoreState`.
- Ajouter `loadGroupKey(convId)` et `distributeGroupKey(convId, memberPubkeys)` appelant `e2ee.ts`.
- Détecter si la conversation est le groupe global (`convId === 'default_global'`) et utiliser la group key au lieu de `encryptForRecipients`.

**chatStore.svelte.ts** :

- Le déchiffrement `_decryptBatch` et `_injectMessage` doit supporter deux formats :
  1. Messages chiffrés avec enveloppe par destinataire (ancien format : `nonce` + `encrypted_keys` dans le message) — déchiffrement via `decryptMessage` actuel.
  2. Messages chiffrés avec group key (nouveau format : le contenu est `crypto_secretbox_easy` avec la group key, `nonce` présent, pas d'`encrypted_keys`) — déchiffrement via `decryptGroupMessage(content, nonce, groupKey)`.

Le frontend doit déterminer le format du message en fonction de son arrivée : si le message a un champ `encrypted_keys` → ancien format ; si le message a un champ `group_key_version` → nouveau format. La distinction est faite côté backend lors de l'envoi.

**e2ee.ts** :

- Déjà prêt. Vérifier que `distributeGroupKey` gère le cas "membre sans clé publique" (le catch actuel fait un console.warn — à changer en retour d'erreur explicite).

### 5.4 Politique de rotation

- **Time-based** : toutes les 7 jours (configurable dans `config.rs` via `group_key_rotation_days`).
- **Event-based** : ajout ou retrait d'un membre via `POST /api/conversations/{id}/join` ou suppression.
- La rotation est déclenchée par le créateur/admin du groupe. Pour le groupe global, seul l'admin peut déclencher la rotation.
- Après rotation, les membres qui ont quitté avant la rotation ne peuvent plus déchiffrer les nouveaux messages (forward secrecy).

---

## 6. Questions ouvertes

1. **Historique post-rotation** : après une rotation, les nouveaux membres (ayant rejoint après la rotation) ne peuvent pas déchiffrer l'historique pré-rotation. Est-ce acceptable ? Réponse attendue : oui, c'est le comportement voulu pour la confidentialité.
2. **Expiry des anciennes clés** : combien de versions précédentes garder ? Proposition : garder les 3 dernières versions (rotation 7j → 21j d'historique déchiffrable). Au-delà, les anciennes clés sont expirées côté serveur.
3. **Group key pour les DMs** : les DMs pourraient aussi bénéficier d'une group key (une clé par DM). Mais cela complique le modèle et n'apporte pas de benefice clair pour un DM à 2 participants. Laisser de côté pour maintenant.

---

## 7. Prochaines étapes

1. **Acceptation du RFC** par l'architect (ou rejet avec commentaires).
2. **Création de l'ADR-016** (voir fichier contexte séparé) — ce RFC sert de base.
3. **Implémentation** par coder selon le plan ci-dessus.
4. **Review security** par security-auditor sur les routes E2EE group key.
5. **Tests E2E** pour valider : un nouvel utilisateur peut envoyer/recevoir des messages chiffrés dans le groupe global après setup E2EE.

---

*Document généré par architect, session 492, 2026-08-24. Conçu pour être converti en ADR-016 si accepté.*
