# 🎨 Contexte Frontend — Nook

> Mis à jour : 2026-05-16

## Stack Technique
- **Framework** : SvelteKit 5 (Runes mode)
- **Langage** : TypeScript
- **État** : Boutique (Pinia-like stores via `$state` dans Svelte 5)
- **Build** : Vite + Svelte compiler via adapter-auto → output `/build`

## Stores Principaux

### authStore (`src/lib/authStore.ts`)
- Gère `currentUser` et `currentRoom` (objet ou null pour rooms globales)
- JWT + cookie session (`nook_session` httpOnly côté serveur)
- `loading`, `error` states
- Après login frais (non "se souvenir") : appelle `loadConversations()` puis navige vers room par défaut
- Après login "se souvenir" : navigate directement vers room mémorisée (_NAMESPACE/IDENTIFIER ou implicite)

### cryptoStore (`src/lib/cryptoStore.svelte.ts`)
- **X25519 keypair** : clé privée 32 bytes stockée dans IndexedDB, mot de passe en mémoire volatile seulement
- **Clé de session** (pour partage de message) : `crypto_box_key_pair()` par message, privée chiffrée par clé publique de chaque destinataire
- **sessionStorage** : `nook_privkey`, `nook_pubkey`, `nook_crypto_key`, `nook_userid` (tab volatile, survive page reloads)
- **Bug racine (36eefe5c)** : `await registerPublicKeyOnServer()` était fire-and-forget → clé publique pas synchronisée au moment où `ready=true`
- **Fix E2EE retail (f0a8c8d1)** : `encryptForRecipients` → try/catch par destinataire → `console.warn` par échec individué

```ts
// encryptForRecipients (après f0a8c8d1) — pattern:
for (const [userId, pubKeyB64] of Object.entries(recipientPubkeys)) {
  try {
    const recipientKey = base64ToUint8Array(pubKeyB64, 32);
    const enc = crypto_box_seal(sessionKeyPair.publicKey, recipientKey, sessionKeyBox);
    encryptedKeys[userId] = uint8ToBase64(enc);
  } catch (e) {
    console.warn(`[encryptForRecipients] participant ${userId} failed:`, e);
    // continue — autres destinataires peuvent encore être chiffrés
  }
}
```

### chatStore (`src/lib/chatStore.svelte.ts`)
- `sendMessage` : appelle `encryptForRecipients` avant POST serveur
- `loadMessages` / `loadMoreMessages` : après fetch, appelle `_decryptAllIfReady()`
- `_decryptAllIfReady()` : tentatives multiples de déchiffrement (polling)
- `_FAILED_DECRYPT_IDS` : Set d'IDs de messages qui n'ont pas pu être déchiffrés — après fix 36eefe5c, ne mutile plus les champs E2EE

## Règles Svelte 5 (CRITIQUES)

### Event binding — éviter form onsubmit
```svelte
<!-- ❌ NE FONCTIONNE PAS dans Svelte 5 -->
<form onsubmit={handleSubmit}>
  <button type="submit">Envoyer</button>
</form>

<!-- ✅ FONCTIONNE -->
<button type="button" onclick={handleSubmit}>Envoyer</button>
```
Quirk : événement submit n'est pas fiable quand bouton à l'intérieur du form. Workaround : utiliser button onclick uniquement.

### Proxies Svelte 5
- `$derived.by(fn)` : pas de réassignation
- Pour référence mutable partagée : utiliser `unsafeWindow` pattern (accès window direct, hors reactive tracking)

### Named Arguments dans .svelte
- `function foo({a, b}: {a: string, b: number}) {}` — toujours typer les objets

## Patterns récurrents

### Fetch membre E2EE
```ts
const response = await fetch(`/api/rooms/${roomId}/members`);
const members = await response.json();
const pubkeys = Object.fromEntries(members.map(m => [m.id, m.public_key]));
```

### sodium-wrappers base64
- `sodium.from_base64(s)` retourne Uint8Array
- `sodium.to_base64(bytes)` → string base64
- X25519 pubkey : 32 bytes → 44 chars base64

### Message structure (contrat API)
```ts
interface Message {
  id: string
  content: string          // texte chiffré base64 (encrypted)
  encrypted: boolean       // toujours true si E2EE activé
  nonce: string           // Uint8Array 24 bytes → base64
  sender_public_key: string // 44 chars base64
  encrypted_keys: Record<string, string> // par user_id: encrypted session key base64
}
```

### Erreurs E2EE connues
- `"incorrect key pair for the given ciphertext"` = clé de session chiffrée avec mauvaise clé publique
- `"incomplete input"` = taille entrée < 24 bytes (nonce malformé)
- `"invalid input"` = boxed < MACBYTES (données corrompues ou mauvaise clé)

## Commandes
```bash
npm ci --legacy-peer-deps
npm run build
```
