# 📚 LEARNING.md — Mémoire technique du projet Nook

> Fichier vivant maintenu par Claude. Contient bugs résolus, décisions architecturales,
> patterns validés et tout ce qui évite de réapprendre les mêmes choses.  
> **Mise à jour à chaque session.**

---

## 🐛 BUGS ACTIFS (à résoudre)

### Bug #1 — BUILD FRONTEND CASSÉ ❌ (PRIORITÉ HAUTE)

**Erreur CI** :
```
[vite-plugin-svelte:compile-module] src/lib/conversationStore.svelte.ts
Cannot export state from a module if it is reassigned. → state_invalid_export
```

**Cause** : `conversationStore.svelte.ts` exporte des variables `$state` réassignées.

**Fix** :
```typescript
// ✅ Convertir en objet $state unique
export const conversationStore = $state<ConversationState>({
  conversations: [], activeConversationId: null, participants: [], availableUsers: []
});
// Puis partout : conversationStore.conversations = [...] au lieu de conversations = [...]
```

**Status** : 🔴 Non résolu — bloque le CI complet

---

### Bug #2 — IMPORTS CASSÉS dans authStore ❌

**Cause** : `authStore.svelte.js` refactorisé en classe mais n'exporte plus les noms attendus.

**Exports manquants** : `authUser`, `isAuthenticated`, `isAdmin`, `needsPasswordChange`,
`authLoading`, `initAuth()`, `setAuthenticated()`

**Fichiers impactés** : layout, login, chat, calendar, call, admin, change-password,
conversationStore, chatStore, webrtc-calls, crypto, mediaStore

**Status** : 🔴 Non résolu

---

### Bug #3 — `connectionError.set()` cassé ❌

**Cause** : `connectionError` n'est plus un writable store, c'est un champ de `chatStore`.

**Fix** :
```typescript
// ❌ import { connectionError } from './chatStore.svelte.ts';
// ✅ import { setConnectionError } from './chatStore.svelte.ts';
```

**Fichiers** : conversationStore, mediaStore, MediaPlayer, MediaRecorder

**Status** : 🔴 Non résolu

---

### Bug #4 — `sodiumLoading` / `sodiumError` cassés dans layout ❌

**Cause** : layout utilise la syntaxe store Svelte 4 (`sodiumLoading.subscribe()`, `$sodiumError`)
mais `sodium.svelte.js` n'exporte que `sodiumState` (objet `$state`).

**Fix** :
```svelte
// ✅ Utiliser waitForSodium() + sodiumState directement
import { sodiumState, waitForSodium } from '$lib/sodium.svelte.js';
await waitForSodium();
sodiumState.error // au lieu de get(sodiumError)
```

**Status** : 🔴 Non résolu

---

### Bug #5 — Incohérence schéma SQL ⚠️

**Cause** : `001_initial.sql` crée `conversation_members` mais `db.rs` utilise
`conversation_participants`.

**Fix** : Corriger `db.rs` pour utiliser `conversation_members` (évite migration destructive).

**Status** : 🟡 Non bloquant CI, mais crash runtime

---

## ✅ BUGS RÉSOLUS

### [Résolu] Diamond dependency rand_core 0.6/0.9

**Contexte** : Upgrade de rand 0.8 → 0.9 avec argon2 0.5 qui utilise rand_core 0.6.

**Erreur** :
```
error[E0277]: the trait `CryptoRngCore` is not implemented for `&mut rand::rngs::OsRng`
```

**Cause** : `rand 0.9` utilise `rand_core 0.9`, mais `argon2 0.5` / `password-hash 0.5`
attendent `rand_core 0.6`. Les deux `OsRng` sont des types incompatibles.

**Fix** :
```toml
# Cargo.toml — deux crates rand coexistent
rand = { version = "0.9", features = ["std", "std_rng", "os_rng"] }
rand_core = { version = "0.6", features = ["std", "getrandom"] }  # pour argon2
```
```rust
// auth.rs
use rand_core::OsRng;  // ← 0.6, compatible argon2
// webrtc.rs
use rand::RngCore;
rand::rng().fill_bytes(&mut buf);  // ← rand 0.9
```

---

### [Résolu] axum 0.7 → 0.8 breaking changes

**Changements** :
- `axum::extract::Host` supprimé → extraire depuis `HeaderMap` :
  ```rust
  let host = headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("localhost:3000");
  ```
- `Message::Text(String)` → `Message::Text(Utf8Bytes)` : utiliser `.into()` pour convertir
- `text.clone()` (Utf8Bytes) vers channel String → `.to_string()`
- Middleware `FromFn<(), ...>` → retirer `Host` extractor de la signature

---

### [Résolu] Cargo.lock désynchronisé après upgrade Cargo.toml

**Symptôme** : Docker compilait avec `axum 0.7.9` malgré `axum = "0.8"` dans Cargo.toml.
`async-trait` incompatible déclenché car axum 0.7 en dépend différemment.

**Cause** : `Cargo.lock` jamais régénéré après les modifications du `Cargo.toml`.

**Fix** :
```bash
cd backend
rm Cargo.lock
cargo update
git add Cargo.lock && git commit -m "chore(deps): regenerate Cargo.lock"
```

**Versions après fix** : axum 0.8.8, rand 0.9.2, reqwest 0.13.2, tower_governor 0.8.0

---

### [Résolu] `home@0.5.12 requires rustc 1.88`

**Symptôme** :
```
error: rustc 1.85.1 is not supported by the following package:
  home@0.5.12 requires rustc 1.88
```

**Fix** : `FROM rust:1.88-bookworm` dans tous les Dockerfiles.

---

### [Résolu] proc-macro async-trait incompatible en Docker

**Symptôme** (récurrent sur 5 tentatives !) :
```
error: cannot produce proc-macro for `async-trait v0.1.89` as the target
`x86_64-unknown-linux-gnu` does not support these crate types
```

**Causes identifiées et fixes** (par ordre de découverte) :

| # | Cause | Fix |
|---|-------|-----|
| 1 | `FROM --platform=$BUILDPLATFORM rust:...` | Supprimer `--platform=$BUILDPLATFORM` |
| 2 | `BUILDKIT_INLINE_CACHE: "1"` dans docker-compose | Supprimer ce build-arg |
| 3 | Technique "dummy fn main()" pour cache deps | Remplacer par `cargo-chef` |
| 4 | **Cargo.lock désynchronisé** (cause racine réelle) | `rm Cargo.lock && cargo update` |

**Cause racine finale** : Cargo.lock pointait vers axum 0.7 (qui dépend d'async-trait
d'une façon incompatible avec le contexte Docker) malgré Cargo.toml demandant axum 0.8.
Toutes les autres tentatives masquaient le vrai problème.

**Leçon clé** : Après tout upgrade de `Cargo.toml`, **toujours régénérer `Cargo.lock`**
localement et le commiter avant de pusher.

---

### [Résolu] ARG non interpolé dans COPY

**Symptôme** : `COPY ${BACKEND_PATH}/Cargo.toml ./` échoue car BuildKit n'interpole
pas les ARG utilisateur dans les chemins sources des COPY.

**Fix** : Hardcoder tous les chemins dans les COPY.

---

### [Résolu] npm_net externe crash CI

**Cause** : `docker-compose.yml` référençait `npm_net` comme réseau externe (pour
Nginx Proxy Manager en prod). Ce réseau n'existe pas sur les runners GitHub.

**Fix** : Un `docker-compose.yml` sans réseaux externes, compatible CI.

---

## ✅ DÉCISIONS ARCHITECTURALES

### Deux Dockerfiles distincts

- `Dockerfile` : compilation depuis sources avec `cargo-chef`. Utilisé par test-nook.yml et docker-compose.
- `Dockerfile.release` : binaires pré-compilés par Backend.yml. Utilisé par Docker.yml et ci-new2.yml.

**Raison** : le multi-arch Docker (linux/amd64 + linux/arm64) est incompatible avec la
compilation Rust dans le même Dockerfile — les proc-macros cassent. Solution : compiler
séparément dans Backend.yml (matrice), puis assembler dans Docker.yml.

---

### Versioning unifié via fichier `VERSION`

- **Source de vérité** : `VERSION` à la racine (`0.5.0`)
- `release.yml` bumpe `VERSION` + `backend/Cargo.toml` + `frontend/package.json` + tag git
- `Docker.yml` lit `VERSION` pour tagger l'image GHCR
- Badges README via `ghcr-badge.egpl.dev` → affichage automatique version + taille image

---

### Pattern stores Svelte 5 retenu

```typescript
// fichier: src/lib/monStore.svelte.ts
export const monStore = $state<MonState>(createInitialState());
export function setData(data: string[]): void { monStore.data = data; }
export function getData(): string[] { return monStore.data; }
export function reset(): void { Object.assign(monStore, createInitialState()); }
```

Stores conformes : `chatStore`, `callStore`, `sodiumState`, `recordingState`  
Stores à corriger : `conversationStore` (Bug #1), `authStore` (Bug #2)

---

### Auth : Cookie HttpOnly

Cookie `auth_token=userId:token` (HttpOnly, SameSite=Lax, Max-Age=86400).  
Token stocké en DB → révocable côté serveur. Changement de mdp → nouveau token.

---

### Docker : Image Distroless

`gcr.io/distroless/cc-debian12:nonroot` — ~8-15 MB, pas de shell, user nonroot (65532).
Libs copiées manuellement : libsqlite3, libsodium, libssl, libcrypto.

---

## 📝 SESSIONS DE TRAVAIL

### Session 1 — 2026-02-19
- Analyse complète du projet
- Identification des 5 bugs actifs (Svelte 5 runes)
- Création de CLAUDE.md et LEARNING.md

### Session 2 — 2026-02-21 (matin)
- Upgrade dépendances Rust : axum 0.7→0.8, rand 0.8→0.9, reqwest 0.12→0.13
- Fix diamond dependency rand_core 0.6/0.9
- Fix axum 0.8 breaking changes (Host, Message::Text, middleware)
- Fix GitHub Actions workflow test-nook.yml (Dockerfile ARG, docker-compose CI)

### Session 3 — 2026-02-21 (après-midi)
- Debugging récurrent erreur proc-macro async-trait (5 tentatives)
- Découverte cause racine : Cargo.lock désynchronisé
- Fix : `rm Cargo.lock && cargo update` → versions correctes dans le lock
- Refonte complète architecture workflows CI :
  - `Backend.yml` + `Frontend.yml` : workflows manuels standalone
  - `test-nook.yml` : intégration avec Dockerfile (cargo-chef)
  - `Docker.yml` : assemblage artifacts + Dockerfile.release
  - `release.yml` : nouveau workflow de versioning sémantique
- Ajout `VERSION`, `Dockerfile.release`
- Mise à jour `README.md` avec badges dynamiques GHCR
- Création de `DOCKER.md` (règles et pièges Docker)

### À faire — prochaine session
1. Corriger `conversationStore.svelte.ts` (Bug #1 — bloquant CI frontend)
2. Compléter `authStore.svelte.js` avec les exports manquants (Bug #2)
3. Corriger imports `connectionError` → `setConnectionError` (Bug #3)
4. Corriger layout pour `sodiumLoading`/`sodiumError` (Bug #4)
5. Corriger incohérence nom table SQL `conversation_members` vs `conversation_participants` (Bug #5)
