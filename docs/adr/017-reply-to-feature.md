# ADR-017: Reply-to (citer/répondre à un message)

**Date**: 2026-08-27
**Status**: Accepté
**Décideur**: architect
**Supersede**: — (première implémentation de la fonction reply/quote)

---

## Contexte

La fonction « répondre » (reply/quote un message) est **absente** du code Nook :
- Zéro occurrence frontend de `reply` ou `quote`
- Zéro colonne `reply_to` dans les migrations backend (`001_initial.sql` jusqu'à `023_group_key_version.sql`)
- Le rapport global tests (t_741a980b) confirme BUG-2 ELEVEE

Pourtant, c'est une fonction critique de tout messagerie moderne : les utilisateurs ont besoin de répondre à un message spécifique pour maintenir le contexte de la conversation, surtout dans le groupe global où plusieurs sujets se mélangent.

### Modèle actuel (constat)

```
Message (db.rs):
  id, conversation_id, sender_id, content, message_type, file_id,
  encrypted, timestamp, created_at, edited_at

SendMessageRequest (db.rs):
  content, encrypted, nonce, encrypted_keys, sender_key_version, group_key_version

ChatMessage (chatStore.svelte.ts):
  mêmes champs + sender_name, sender_avatar_style, sender_avatar_seed,
  sender_public_key, sender_key_version, group_key_version
```

Aucun champ `reply_to_id` ni `reply_to` n'existe.

---

## Options considérées

### A — Threads imbriqués (Slack/Discord style)

Chaque message peut lancer un thread (sous-conversation) avec ses propres messages. Modèle complexe nécessitant une table `threads` et une UI dédiée.

**Rejeté** : complexité excessive pour Nook (famille/privé). Pas besoin de threads parallèles. Contrainte ADR-001 (simplicité Docker).

### B — Reply-to simple avec référence (Signal/WhatsApp style)

Chaque message peut référencer un autre message via `reply_to_id`. La citation est affichée au-dessus du message de réponse. Pas de threads — la conversation reste plate.

**Avantages** : simple, familier, compatible avec le modèle temporel de Nook, implémentation légère.

**Retenu** : correspond au besoin (BUG-2) sans ajout de complexité architecturale.

### C — Référence uniquement par ID (citation manuelle)

Le client envoie `[Réponse à msg_abc]` dans le contenu texte. Pas de structure dédiée.

**Rejeté** : fragile, pas de preview possible, pas de lien cliquable, impossible de résoudre côté serveur.

---

## Décision

**Option B — Reply-to simple avec référence** retenue.

---

## Spécification complète

### 1. Schéma SQL

```sql
-- Migration 024: reply-to support
ALTER TABLE messages ADD COLUMN reply_to_id TEXT;

-- FK optionnelle : si le message cité est supprimé, on garde la référence
-- mais reply_to_id devient NULL (le frontend affichera « message supprimé »)
-- Note: SQLite ne supporte pas ALTER TABLE ADD CONSTRAINT FK directement.
-- La FK est appliquée au niveau applicatif (send_message vérifie l'existence).
```

**Pourquoi pas de FK stricte ?** SQLite ne supporte pas `ALTER TABLE ADD CONSTRAINT FOREIGN KEY`. Une FK stricte empêcherait la suppression d'un message cité (ou supprimerait en cascade les réponses). Le choix : FK applicatif (vérif dans `send_message`), et `ON DELETE SET NULL` simulé dans `delete_message`.

### 2. Backend (`backend/src/db.rs`)

#### `Message` struct (ajout)

```rust
pub struct Message {
    // ... existant ...
    pub reply_to_id: Option<String>,  // NOUVEAU
}
```

#### `MessageWithSender` struct (ajout)

```rust
pub struct MessageWithSender {
    // ... existant ...
    pub reply_to_id: Option<String>,  // NOUVEAU
    // reply_to est un champ JSON calculé (pas stocké en DB)
}
```

#### `SendMessageRequest` (ajout)

```rust
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    // ... existant ...
    /// ID du message cité (optionnel)
    #[serde(default)]
    pub reply_to_id: Option<String>,
}
```

#### `send_message` handler (modification)

```
1. Vérifier que reply_to_id pointe vers un message dans MÊME conversation
   (sécurité : pas de citation cross-conversation)
2. INSERT INTO messages (..., reply_to_id) VALUES (..., ?)
3. Dans le msg_json retourné, inclure "reply_to" avec preview du message cité
```

#### `get_conversation_messages` (modification)

Le JOIN doit inclure `reply_to_id` et calculer le champ `reply_to` (preview du message cité) pour chaque message.

```sql
SELECT
  m.*,
  COALESCE(u.name, u.username) AS sender_name,
  u.avatar_style AS sender_avatar_style,
  u.avatar_seed AS sender_avatar_seed,
  u.public_key AS sender_public_key,
  -- Champs du message cité (si existe)
  rm.id AS reply_to_id,
  COALESCE(ru.name, ru.username) AS reply_to_sender_name,
  rm.content AS reply_to_content,
  rm.message_type AS reply_to_message_type,
  rm.file_id AS reply_to_file_id,
  rm.nonce AS reply_to_nonce
FROM messages m
JOIN users u ON m.sender_id = u.id
LEFT JOIN messages rm ON m.reply_to_id = rm.id
LEFT JOIN users ru ON rm.sender_id = ru.id
WHERE m.conversation_id = ?
ORDER BY m.created_at ASC
```

#### Champ `reply_to` dans le JSON servi

```json
{
  "id": "msg_xyz",
  "reply_to_id": "msg_abc",
  "reply_to": {
    "id": "msg_abc",
    "sender_name": "Alice",
    "sender_avatar_style": "adventurer",
    "sender_avatar_seed": "abc123",
    "content": "Le contenu du message cité...",
    "message_type": "text",
    "file_id": null,
    "encrypted": false,
    "nonce": null
  },
  "content": "Voici ma réponse !",
  ...
}
```

**Si le message cité a été supprimé** : `reply_to_id` devient NULL, `reply_to` est `null`. Le frontend affiche « Message supprimé ».

#### `delete_message` handler (modification)

Quand un message est supprimer, mettre à jour les messages qui le citent :
```sql
UPDATE messages SET reply_to_id = NULL WHERE reply_to_id = ?
```

**Alternative** : supprimer en cascade les réponses (trop agressif). Choix : SET NULL.

#### Validation `reply_to_id`

- `reply_to_id` doit référencer un message existant dans la même conversation
- Pas de récursion : si le message cité a lui-même un `reply_to_id`, on ne le suit PAS (le `reply_to` servi est toujours un message de base)
- Limite de profondeur : 1 niveau de citation (pas de « réponse à une réponse à une réponse »)

### 3. Frontend

#### `ChatMessage` interface (`chatStore.svelte.ts`)

```typescript
export interface ReplyToPreview {
  id: string;
  sender_name: string;
  sender_avatar_style: string | null;
  sender_avatar_seed: string | null;
  content: string;
  message_type: string;
  file_id: string | null;
  encrypted: boolean;
  nonce: string | null;
}

export interface ChatMessage {
  // ... existant ...
  reply_to_id: string | null;
  reply_to: ReplyToPreview | null;
}
```

#### `sendMessage` function (`chatStore.svelte.ts`)

```typescript
export async function sendMessage(
  content: string,
  conversationId: string,
  replyToId?: string  // NOUVEAU paramètre optionnel
): Promise<ChatMessage | null> {
  // ... existant ...
  if (replyToId) {
    body.reply_to_id = replyToId;
  }
  // ...
}
```

#### `chat/+page.svelte` — Bouton « Répondre »

Ajouter un bouton `↩️ Répondre` dans le menu contextuel (à côté de « Éditer » et « Supprimer ») :

```svelte
<button class="msg-menu-item reply"
        onclick={() => { startReply(msg); messageMenuMsgId = null; }}>
  ↩️ Répondre
</button>
```

#### `chat/+page.svelte` — Bandeau de citation

Quand `replyingToMsgId` est défini, afficher un bandeau au-dessus du champ de saisie :

```svelte
{#if replyingToMsg}
  <div class="reply-banner">
    <div class="reply-banner-text">
      ↩️ Réponse à <strong>{replyingToMsg.sender_name}</strong>:
      {replyingToMsg.content.substring(0, 60)}...
    </div>
    <button class="reply-banner-cancel" onclick={() => replyingToMsg = null}>✕</button>
  </div>
{/if}
```

#### `chat/+page.svelte` — Rendu de la citation dans le message

Dans le `{#each reversedMessages as msg}`, avant le contenu du message :

```svelte
{#if msg.reply_to}
  <div class="reply-quote" onclick={() => scrollToMessage(msg.reply_to.id)}>
    <div class="reply-quote-sender">{msg.reply_to.sender_name}</div>
    <div class="reply-quote-content">
      {#if msg.reply_to.encrypted}
        🔒 Message chiffré
      {:else if msg.reply_to.message_type === 'image'}
        📷 Image
      {:else if msg.reply_to.message_type === 'audio'}
        🎙️ Audio
      {:else if msg.reply_to.message_type === 'video'}
        🎥 Vidéo
      {:else if msg.reply_to.message_type === 'file'}
        📎 Fichier
      {:else}
        {msg.reply_to.content.substring(0, 100)}
      {/if}
    </div>
  </div>
{:else if msg.reply_to_id}
  {#if !msg.reply_to}
    <div class="reply-quote deleted">🗑️ Message supprimé</div>
  {/if}
{/if}
```

**Note** : le `msg.reply_to_id` sans `reply_to` (deleted) nécessite une passe dans `loadMessages` pour afficher le fallback.

### 4. CSS (styles)

```css
.reply-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary, #f1f5f9);
  border-left: 3px solid var(--accent, #3b82f6);
  border-radius: 6px;
  margin-bottom: 8px;
}

.reply-banner-text {
  flex: 1;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.reply-banner-cancel {
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.6;
}

.reply-banner-cancel:hover { opacity: 1; }

.reply-quote {
  margin-bottom: 4px;
  padding: 6px 10px;
  background: var(--bg-secondary, #f1f5f9);
  border-left: 2px solid var(--accent, #3b82f6);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  max-width: 80%;
}

.reply-quote:hover {
  background: var(--bg-tertiary, #e2e8f0);
}

.reply-quote-sender {
  font-weight: 600;
  color: var(--accent, #3b82f6);
  margin-bottom: 2px;
}

.reply-quote-content {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.reply-quote.deleted {
  font-style: italic;
  opacity: 0.6;
}
```

### 5. Scroll à la citation

```typescript
function scrollToMessage(msgId: string) {
  const el = document.querySelector(`[data-msg-id="${msgId}"]`);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    el.classList.add('highlight-msg');
    setTimeout(() => el.classList.remove('highlight-msg'), 2000);
  }
}
```

---

## Conséquences

### +

- Fonction familière (Signal/WhatsApp/Slack) — zéro learning curve pour les utilisateurs
- Simple : un seul champ FK, pas de nouvelle table, pas de thread model
- Compatible avec E2EE : la citation est en clair (méta-données), le chiffrage du contenu répondant ne change pas
- Pas d'impact sur les messages existants (colonne NULL par défaut)
- Bandeau visuel clair : sait toujours à quel message on répond

### -

- Migration SQLite `ALTER TABLE ADD COLUMN` : pas de FK déclarée (contrainte applicative)
- Si le message cité est chiffré et que le destinataire n'a pas la clé, il voit « 🔒 Message chiffré » au lieu du contenu — comportement acceptable (même logique que le reste du chat)
- `reply_to` preview est servi par le serveur (charge légère JOIN supplémentaires par message)
- Limitation à 1 niveau de citation (pas de nested quotes) — assumé

### → Agents impactés

- **coder** : implémentation complète (backend + frontend + CSS)
- **ux-reviewer** : validation de l'UI (placement bandeau, rendu citation, responsiveness mobile)
- **tester** : tests E2E (citation, suppression message cité, scroll, E2EE + citation)

---

## Plan d'implémentation

### Phase 1 — Backend (migration + routes)

1. Créer `backend/migrations/024_reply_to.sql`
2. Modifier `Message` struct, `MessageWithSender` struct, `SendMessageRequest` struct
3. Modifier `send_message` handler : validation + INSERT
4. Modifier `get_conversation_messages` : JOIN avec `reply_to`
5. Modifier `delete_message` : SET NULL sur `reply_to_id`
6. Tester manuellement avec curl

### Phase 2 — Frontend (store + UI)

1. Étendre `ChatMessage` interface avec `reply_to_id` et `reply_to`
2. Modifier `sendMessage` pour accepter `replyToId?`
3. Ajouter état `replyingToMsg` dans `chat/+page.svelte`
4. Ajouter bouton « Répondre » dans menu contextuel
5. Ajouter bandeau de citation
6. Ajouter rendu de la citation dans le message
7. Ajouter `scrollToMessage`
8. Ajouter CSS

### Phase 3 — Tests

1. Test E2E : citer un message, vérifier le rendu
2. Test E2E : supprimer le message cité, vérifier « Message supprimé »
3. Test E2E : citation avec message chiffré
4. Test E2E : scroll vers la citation au clic

---

## Références

- BUG-2 ELEVEE (rapport global tests, t_741a980b)
- ADR-003 (SQLite — contraintes ALTER TABLE)
- ADR-004 (E2EE — citation en méta-données claires, pas de brèche)
- ADR-010 (Svelte 5 runes — état `replyingToMsg` en $state)
