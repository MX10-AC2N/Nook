# ⚠️ Known Issues — Hermes Agent

> Liste des bugs, pièges et problèmes récurrents à ne pas répéter
> Mis à jour : 2026-05-16

## 🔴 Bugs critiques (RÉSOLUS)

### BUG-001 : Compilation backend échoue (admin.rs)
- **Status :** ✅ FIXÉ commit `327b08e6`
- **Symptôme :** `mismatched closing delimiter` sur `.map_err()`
- **Leçon :** Toujours vérifier la syntaxe Rust avant push

### BUG-002 : E2EE refresh bug — messages non déchiffrés après F5
- **Status :** ✅ FIXÉ commit `0219c73e`
- **Symptôme :** `cryptoStore.ready=false` après refresh, pas de déchiffrement auto
- **Fix :** Polling `_decryptAllIfReady()` dans `chatStore.svelte.ts`
- **Contrainte :** Mot de passe PAS stocké

### BUG-003 : P2P file transfer sécurité groupes
- **Status :** ✅ FIXÉ commit `e9b17418`
- **Fix :** `e2ee.loadGroupKey(convoId)` au lieu clé dérivée

### BUG-004 / BUG-005 : E2EE clé publique désynchronisée serveur
- **Status** : ✅ FIXÉ commit `36eefe5c`
- **Symptôme** : `"incorrect key pair for the given ciphertext"` dans `crypto_box_open_easy`
- **Cause** : `unlockCrypto` positionnait `ready=true` avant completion de `registerPublicKeyOnServer()` (fire-and-forget)
- **Fix** : `await registerPublicKeyOnServer(kp.publicKey)` avant `_keyPair=kp` et `ready=true`
- **Duplication** : `sessionStorage` SET + `await registerPublicKeyOnServer` maintenant groupés avant activation store, duplication ligne 148-185 corrigée

### BUG-006 : E2EE encryptForRecipients casse total si 1 destinataire invalide
- **Status :** ✅ FIXÉ commit `f0a8c8d1`
- **Symptôme :** `encrypted_keys:{}` (vide) sur serveur, messages marqués `encrypted:true` mais impossibles à déchiffrer
- **Cause** : Boucle `for (const [userId, pubKeyB64] of Object.entries(...))` sans try/catch → un seul destinataire avec clé malformée casse toute la boucle
- **Fix** : try/catch par destinataire + `console.warn` par participant échoué + `console.info` dans `encryptMessage` pour diagnostic
- **Impact backend** : `db.rs ligne 456` ne stocke pas `encrypted_keys` si vide → message arrive sans mécanisme de déchiffrement

### _FAILED_DECRYPT_IDS Fix
- **Commit** : `36eefe5c` (inclus dans BUG-005)
- **Avant** : En cas d'échec de déchiffrement, `_FAILED_DECRYPT_IDS` mutilait les champs E2EE du message (`delete nonce/encrypted/sender_public_key`) → re-déchiffrement futur impossible
- **Après** : Les champs E2EE sont conservés — seul l'ID est ajouté au Set → re-déchiffrement automatique possible quand la racine causale est corrigée

## 🟡 Pièges récurrents (attention !)

### PITFALL-001 : Modifier les versions des dépendances
- Règle : Un commit de fix ne touche QUE le bug signalé
- Jamais toucher à Cargo.toml dans un fix code

### PITFALL-002 : rand 0.9 API change
- `thread_rng()` → `rng()`
- `distributions::` → `distr::`

### PITFALL-003 : Axum 0.8 breaking changes
- `{param}` pas `:param`
- `Message::Text` → `Utf8Bytes`
- `axum::extract::Host` supprimé → HeaderMap

### PITFALL-004 : CORS + credentials PANIC
- `allow_origin(Any)` + `allow_credentials(true)` → PANIC
- Lister origines explicitement depuis config

### PITFALL-005 : Svelte 5 form onsubmit
- `<form onsubmit={handler}>` ne fire pas fiablement avec `<button type="submit">`
- Fix : `<button type="button" onclick={handler}>` — pas de form onsubmit

## 🟠 Problèmes connus non résolus

### E2EE : Anciens messages indéchiffrables après rotation de clé
- **Status** : ⚠️ STRUCTUREL — pas de fix code possible
- **Cause** : Rotation de clé X25519 côté serveur → clé de session des anciens messages chiffrée avec ancienne clé, n'a pas de paire privée actuelle
- **Workaround** : Nouveaux messages OK, anciens deviennent illisibles (attendu après reset/changement mot de passe)

## 📝 Checklists pre-commit

1. ✅ Syntaxe Rust correcte (`.map_err()`, parenthèses, accolades)
2. ✅ Pas de modification de versions dans Cargo.toml
3. ✅ `cargo clippy` clean
4. ✅ Patterns Svelte 5 respectés (`$state`, `$derived.by`)
5. ✅ Tests E2E mis à jour si nouveaux endpoints
6. ✅ Pas de secrets en dur
7. ✅ Workflows dans l'ordre : Frontend → Backend → Turn → Docker
8. ✅ `concurrent futures` nécessitent `join!` pas `spawn` dans les handlers Axum
9. ✅ X25519 keys : 32 bytes Uint8Array, base64 encode → 44 chars
10. ✅ `activate_no_keepalive` pour éviter connexion HTTP keep-alive sur serveur Nook
