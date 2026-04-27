# ⚠️ Known Issues — Hermes Agent

> Liste des bugs, pièges et problèmes récurrents à ne pas répéter
> Mis à jour : 2026-04-27

## 🔴 Bugs critiques (à corriger en priorité)

### BUG-001 : Compilation backend échoue (admin.rs)
- **Status :** EN COURS (commit 327b08e6 poussé, CI à vérifier)
- **Symptôme :** `mismatched closing delimiter` sur `.map_err()`
- **Cause :** Parenthèses mal fermées dans les closures
- **Fix :** Utiliser bloc `{ }` dans `.map_err(|err| { (...) })?`
- **Leçon :** Toujours vérifier la syntaxe Rust avant push

### BUG-002 : E2EE refresh bug
- **Status :** ✅ FIXÉ (commit 0219c73e)
- **Symptôme :** Messages chiffrés visibles après refresh page
- **Cause :** `cryptoStore.ready = false` après refresh, pas de déchiffrement auto
- **Fix :** Polling dans `chatStore.svelte.ts` (ne s'arrête plus à 1ère tentative)
- **Code :** `_decryptAllIfReady()` appelé APRÈS chargement messages (loadMessages, loadMoreMessages)
- **Contrainte :** Mot de passe PAS stocké (sécurité), pas d'unlockCrypto() auto

### BUG-003 : P2P file transfer (sécurité)
- **Status :** ✅ CODE FIXÉ (commit e9b17418)
- **Symptôme :** Transfert >50 Mo uniquement 1-to-1 (pas de groupes)
- **Code :** `file-transfer.svelte.ts` + `e2ee.ts` (export e2ee instance)
- **Fix sécurité :** Utilise maintenant `e2ee.loadGroupKey(convoId)` au lieu d'une clé dérivée
- **À faire :** Tester sur conversation 1-to-1 réelle, créer tests E2E

## 🟡 Pièges récurrents (attention !)

### PITFALL-001 : Modifier les versions des dépendances
- **Règle :** Un commit de fix ne touche QUE le bug signalé
- **Erreur commise :** J'ai changé `rustrtc` 0.3.40 → 0.3.39 par erreur
- **Correction :** Restauré immédiatement sur demande utilisateur
- **À retenir :** Jamais toucher à Cargo.toml dans un commit de fix code

### PITFALL-002 : rand 0.9 API change
- **Erreur :** `rand::thread_rng()` n'existe plus
- **Correct :** `rand::rng()`
- **Distribution :** `distr::` pas `distributions::`
- **Import :** `use rand::Rng;` pour `sample_iter()`

### PITFALL-003 : Axum 0.8 breaking changes
- **Routes :** `{param}` pas `:param`
- **Message::Text :** `Utf8Bytes` pas `String`
- **Host :** `axum::extract::Host` supprimé → extraire du HeaderMap

### PITFALL-004 : CORS + credentials
- **Erreur :** `allow_origin(Any)` + `allow_credentials(true)` → PANIC
- **Correct :** Lister origines explicitement depuis config

## 🔵 Problèmes de build CI

### CI-001 : Rust nightly dans Backend.yml
- **Fichier :** `.github/workflows/Backend.yml` ligne 34
- **Toolchain :** `dtolnay/rust-toolchain@nightly`
- **Impact :** Comportement peut différer de stable
- **À vérifier :** Compatibilité `rustrtc` avec nightly

### CI-002 : simple-peer 9.11.1 non maintenu
- **Statut :** ✅ RÉSOLU (PR #28 mergé, commit 65386b88)
- **Risque :** Sécurité, bugs non corrigés
- **Action :** ✅ `webrtc.ts` réécrit avec API WebRTC native (RTCPeerConnection)
- **Commit :** 65386b88 - simple-peer complètement supprimé

## 📝 Ce que je dois vérifier avant chaque commit

1. ✅ Syntaxe Rust correcte (`.map_err()`, parenthèses, accolades)
2. ✅ Pas de modification de versions dans Cargo.toml
3. ✅ `cargo check` pass (si disponible, sinon déléguer à Claude Code)
4. ✅ patterns Svelte 5 respectés (`$state`, `$derived.by`, pas de réassignation)
5. ✅ Tests E2E mis à jour si nouveaux endpoints
6. ✅ Pas de secrets en dur (TURN_SECRET, mots de passe admin)
