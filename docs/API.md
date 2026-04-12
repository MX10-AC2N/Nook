# API Reference — Nook v0.5.0

> Documentation auto-générée des endpoints REST.
> Base URL : `http://<host>:6300/api`

---

## 🔐 Authentification

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/auth/register` | Inscription (en attente approbation) | ❌ |
| POST | `/auth/login` | Connexion → cookie `auth_token` | ❌ |
| POST | `/auth/logout` | Déconnexion | ✅ |
| GET | `/auth/me` | Infos utilisateur courant | ✅ |
| POST | `/auth/change-password` | Changer mot de passe | ✅ |
| POST | `/auth/public-key` | Enregistrer clé publique X25519 | ✅ |
| GET | `/auth/public-keys` | Clés publiques des membres | ✅ |

### POST `/auth/register`
```json
{
  "username": "string",
  "password": "string (min 8 chars)",
  "email": "string",
  "name": "string",
  "invite_token": "string (optional)"
}
```

### POST `/auth/login`
```json
{
  "username": "string",
  "password": "string"
}
```
Response : Set-Cookie `auth_token=user_id:token`

---

## 💬 Conversations

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/conversations` | Liste des conversations | ✅ |
| POST | `/conversations` | Créer une conversation | ✅ |
| GET | `/conversations/{id}` | Détail d'une conversation | ✅ |
| PATCH | `/conversations/{id}/rename` | Renommer | ✅ |
| GET | `/conversations/{id}/participants` | Liste des participants | ✅ |
| POST | `/conversations/{id}/participants` | Ajouter un participant | ✅ |
| POST | `/conversations/{id}/leave` | Quitter la conversation | ✅ |

---

## 📨 Messages

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/conversations/{id}/messages` | Liste des messages (paginé) | ✅ |
| POST | `/conversations/{id}/messages` | Envoyer un message | ✅ |
| PUT | `/conversations/{conv_id}/messages/{msg_id}` | Éditer un message | ✅ |
| DELETE | `/conversations/{conv_id}/messages/{msg_id}` | Supprimer un message | ✅ |

### POST `/conversations/{id}/messages`
```json
{
  "content": "string",
  "encrypted": false,
  "file_id": "string (optional)",
  "encrypted_keys": {} (optional, E2EE)
}
```

---

## 😊 Réactions

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/conversations/{conv_id}/messages/{msg_id}/reactions` | Réactions agrégées | ✅ |
| POST | `/conversations/{conv_id}/messages/{msg_id}/reactions` | Ajouter/réagir | ✅ |
| DELETE | `/conversations/{conv_id}/messages/{msg_id}/reactions` | Supprimer sa réaction | ✅ |

Emojis autorisés : `👍 ❤️ 😂 😮 😢 😡`

### POST (body)
```json
{ "emoji": "👍" }
```

---

## 📅 Calendrier

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/events` | Liste des événements | ✅ |
| POST | `/events` | Créer un événement | ✅ |
| PUT | `/events/{id}` | Modifier un événement | ✅ |
| DELETE | `/events/{id}` | Supprimer un événement | ✅ |

### POST `/events`
```json
{
  "title": "string",
  "date": "YYYY-MM-DD",
  "time": "HH:MM (optional)",
  "description": "string (optional)"
}
```

---

## ♟️ Échecs

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/chess/list` | Liste des parties | ✅ |
| POST | `/chess/create` | Créer une partie | ✅ |
| GET | `/chess/{id}` | État d'une partie | ✅ |
| GET | `/chess/{id}/moves?from=e2` | Coups légaux depuis une case | ✅ |
| POST | `/chess/{id}/move` | Jouer un coup | ✅ |
| POST | `/chess/{id}/ai-move` | Demander un coup IA | ✅ |
| POST | `/chess/{id}/resign` | Abandonner | ✅ |
| POST | `/chess/{id}/join` | Rejoindre une partie | ✅ |

### POST `/chess/create`
```json
{
  "opponent": "human" | "easy" | "medium" | "hard" | "expert" | "godlike",
  "color": "white" | "black",
  "time_limit_secs": 0
}
```

### POST `/chess/{id}/move`
```json
{ "from": "e2", "to": "e4" }
```

---

## 📊 Sondages

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/polls` | Liste des sondages | ✅ |
| POST | `/polls` | Créer un sondage | ✅ |
| GET | `/polls/{id}` | Détail d'un sondage | ✅ |
| POST | `/polls/{id}/vote` | Voter | ✅ |
| POST | `/polls/{id}/close` | Fermer le sondage | ✅ |
| DELETE | `/polls/{id}` | Supprimer (admin) | ✅ |

---

## 📤 Upload / Download

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/upload/chat` | Upload fichier (multipart) | ✅ |
| GET | `/api/download/{id}` | Télécharger un fichier | ✅ |

---

## 🔔 Push Notifications

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/push/vapid-public-key` | Clé publique VAPID | ❌ |
| POST | `/push/subscribe` | S'abonner aux notifications | ✅ |
| GET | `/push/preferences` | Préférences de notification | ✅ |
| POST | `/push/preferences` | Mettre à jour les préférences | ✅ |

---

## 🎥 WebRTC / Appels

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/webrtc/ice-config` | Config ICE (TURN/STUN credentials) | ✅ |
| POST | `/webrtc/offer` | Envoyer une offre SDP | ✅ |
| POST | `/webrtc/answer` | Répondre à une offre | ✅ |
| WS | `/ws` | WebSocket signaling | ✅ (cookie) |

---

## 👤 Utilisateurs (Admin)

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/users` | Liste tous les utilisateurs | 🔒 Admin |
| GET | `/users/pending` | Utilisateurs en attente | 🔒 Admin |
| POST | `/users/approve` | Approuver un utilisateur | 🔒 Admin |
| DELETE | `/users/{id}` | Supprimer un utilisateur | 🔒 Admin |
| GET | `/users/available` | Utilisateurs disponibles | ✅ |

---

## 📨 Invitations

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/invites` | Liste des invitations | 🔒 Admin |
| POST | `/invites` | Créer un lien d'invitation | 🔒 Admin |
| POST | `/invites/delete` | Supprimer une invitation | 🔒 Admin |
| GET | `/invite/validate?token=xxx` | Valider un token | ❌ |
| POST | `/invite/accept` | Accepter une invitation | ❌ |

---

## 📈 Analytics (Admin)

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/analytics` | Statistiques complètes | 🔒 Admin |

---

## 🏥 Health

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/health` | Healthcheck | ❌ |

---

## WebSocket Events

Connect via `ws://<host>:6300/ws` with `auth_token` cookie.

| Type | Description |
|------|-------------|
| `new_message` | Nouveau message |
| `message_edited` | Message édité |
| `message_deleted` | Message supprimé |
| `reaction_updated` | Réaction modifiée |
| `new_poll` | Nouveau sondage |
| `poll_voted` | Vote sur un sondage |
| `poll_closed` | Sondage fermé |
| `calendar_event` | Événement calendrier |
| `chess_move` | Coup joué aux échecs |
| `chess_ai_move` | Coup de l'IA |
| `call_offer` | Offre d'appel |
| `call_answer` | Réponse d'appel |
| `call_ice` | Candidat ICE |
| `call_end` | Fin d'appel |
| `admin_event` | Événement admin |

---

*Documentation générée pour Nook v0.5.0*
