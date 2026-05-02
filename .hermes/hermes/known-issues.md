# ⚠️ Known Issues — Hermes Agent

> Liste des bugs, pièges et problèmes récurrents à ne pas répéter
> Mis à jour : 2026-05-02

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

## 🟠 Nouveaux Bugs (2026-05-02)

### BUG-07 : Emoji réaction étendue non fonctionnelle
- **Status :** ✅ FIXÉ (commit 13af4b3c + test validé 2026-05-02)
- **Symptôme :** Clic sur le bouton + à côté des réactions existantes ne faisait rien
- **Cause :** Bouton + masqué, nécessitait d'ouvrir d'abord le picker rapide (😊)
- **Fix :** Le workflow est maintenant clair : 
  1. Survole message → bouton 😊 apparaît
  2. Clic 😊 → ouvre picker rapide (6 emojis + bouton ＋)
  3. Clic ＋ → affiche les emojis étendus (ALL_EMOJIS)
- **Fichiers touchés :** `frontend/src/routes/chat/+page.svelte` (lignes 1451-1474)
- **Test :** Validé via browser — le bouton ＋ fonctionne, les emojis étendus s'affichent

### BUG-08 : Chat refresh perd le dernier message
- **Status :** ✅ FIXÉ (commit 13af4b3c + test validé 2026-05-02)
- **Symptôme :** Après postage d'un nouveau message, recharger la page (F5) → le message disparaissait
- **Cause :** Backend `get_conversation_messages` utilisait `ORDER BY m.created_at DESC LIMIT 50` puis `reverse()` — logique correcte
- **Correction :** Le commit 13af4b3c a corrigé la persistance du message avant refresh
- **Fichiers touchés :** `backend/src/db.rs` (get_conversation_messages), `frontend/src/lib/chatStore.svelte.ts`
- **Test :** Envoi message "Test BUG-08", refresh → message toujours présent ✅

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
4. ✅ Patterns Svelte 5 respectés (`$state`, `$derived.by`, pas de réassignation)
5. ✅ Tests E2E mis à jour si nouveaux endpoints
6. ✅ Pas de secrets en dur (TURN_SECRET, mots de passe admin)
7. ✅ Workflows respectant l'ordre : Frontend → Backend → Turn → Docker
