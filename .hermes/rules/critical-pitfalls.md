# Pièges critiques du projet Nook

- `rand 0.9` : utiliser `rng()` (NE PAS utiliser `thread_rng()` — supprimé en rand 0.9)
- `rand 0.9` : imports = `use rand::{{rng, distr::Alphanumeric, Rng}}` (NE PAS utiliser `distributions` — déplacé vers `distr`)
- `rand_core 0.6` forcé explicitement pour argon2 — ne JAMAIS importer `rand::rngs::OsRng`
- Routes Axum 0.8 : `{param}` au lieu de `:param`
- `$state` Svelte 5 → utiliser `Object.assign()` ou `$effect`
- CORS + credentials → origins explicites uniquement
- sqlx : éviter les macros quand `queries.json` est vide
- Ne jamais utiliser `?` dans les queries SQLx sans `query!` macro
- `tokio::spawn` sans `move` sur les closures qui capturent des variables
- Oublier de mettre à jour `Cargo.lock` après un changement de dépendance
---
### Tests E2E — regles de validation (2026-04-03)
1. **TOUJOURS** faire `npx playwright test --list` avant push pour valider la syntaxe
2. Chaque `test()` utilise `page` DOIT avoir `async ({ page }) =>` (pas `async () =>`)
3. Chaque `describe` qui utilise `adminPage` DOIT avoir son propre `let adminPage: Page;` + `test.beforeAll`
4. Les titres de test doivent etre UNIQUES dans la meme scope describe — pas de doublons
5. Chaque test doit se terminer par `});` — verifier les blocs try/catch ne laissent pas de test ouvert
6. Quand le backend renvoie 201 au lieu de 200, inclure 201 dans les assertions `.toContain([200, 201, 409])`
---
### 🧪 E2E Tests (2026-04-03)
1. **TOUJOURS** `npx playwright test --list` local avant push — jamais pusher sans validation
2. Si test utilise `page` ⇒ signature DOIT etre `async ({ page }) => {` pas `async () => {`
3. Si describe utilise `adminPage` ⇒ chaque describe besoin de son `let adminPage: Page;` + `test.beforeAll`
4. Titres de test UNIQUES dans chaque describe scope — pas de doublons
5. CHAQUE test doit se fermer avec `});` — verifier try/catch ne laisse pas test ouvert
6. Backend peut retourner 201 (Created) pas juste 200 — inclure dans assertions: `.toContain([200, 201, 409])`
7. `test.describe.serial` pour tests avec state share — `test.describe` standard sinon
8. Helpers partages dans `tests/helpers.ts` — loginAs, loginAsAdmin, clearSession, waitForAppReady


## 🌐 SFU — Pièges critiques

### SFU-P1: APIs rustrtc ne correspondent pas aux exemples Pion/SFU génériques
- **Piège:** Les subagents ont inventé des APIs rustrtc qui n'existent pas (PeerConnection::new(&config) au lieu de PeerConnection::new(config))
- **Bonne pratique:** TOUJOURS verifier les APIs sur https://github.com/restsend/rustrtc/blob/main/src/peer_connection.rs
- **APIs vérifiées:**
  - `PeerConnection::new(config)` — prend par valeur, pas par ref
  - `pc.set_remote_description(desc: SessionDescription)` — pas (SdpType, &str)
  - `pc.create_answer()` — retourne directement SessionDescription
  - `pc.add_ice_candidate(candidate: IceCandidate)` — synchrone, pas async
  - `pc.recv().await` — retourne `Option<PeerConnectionEvent>`
  - `PeerConnectionEvent::Track(transceiver)` — pas "TrackAdded"
  - `MediaRelay::with_capacity(track, capacity)` — pas new()

### SFU-P2: Structure if/else dans webrtc.rs
- **Piège:** Le bloc SFU doit être dans la même chaîne if/else que webrtc_types, PAS imbriqué dedans
- **Correct:** `if webrtc_types.contains() { ... } else if msg_type == "sfu_join" { ... } else if ... { ... }`
- **Incorrect:** `if webrtc_types.contains() { ... } else { /* SFU block ici */ }`

### SFU-P3: MediaRelay et added_sources
- **Piège:** Ajouter la même track deux fois au même peer → SDP reject
- **Solution:** HashSet<String> added_sources avec clé "{user_id}:{peer_id}:{kind:?}"
- **Vérification:** `if added.contains(&source_key) { continue; }` avant add_track

### SFU-P4: Negotiation et signaling_state
- **Piège:** Appeler create_offer() quand signaling_state n'est pas Stable
- **Solution:** Utiliser negotiation_pending: Arc<AtomicBool> pour différer
- **Pattern:** `if state != SignalingState::Stable { pending.store(true); return; }`

### SFU-P5: PLI forwarding RTCP
- **Piège:** Ne pas forward les PLI/FIR des peers → video freeze
- **Solution:** `sender.subscribe_rtcp()` → spawn task → `remote_track.request_key_frame().await`
- **Bonus:** PLI périodique toutes les 3s pour forcer les keyframes

## Svelte 5 Syntax (Session 48)
- **NE PAS** utiliser `onclick|stopPropagation` (syntaxe Svelte 4)
- Utiliser `onclick={(e) => { e.stopPropagation(); ... }}` (Svelte 5)
- Les `$derived` sont des propriétés, PAS des fonctions — `chessStore.board` pas `chessStore.board()`

## Docker & CI (2026-05-12)
- Alpine APK: `sqlite-libs` PAS `libsqlite3` (c'est le nom Debian)
- Backend.yml: noms d'artefacts = `nook-backend-x86_64-unknown-linux-musl` / `nook-backend-aarch64-unknown-linux-musl` (triplet Rust)
- Binaire dans l'artefact = `nook-backend-amd64` / `nook-backend-arm64` (nom cp séparé)
- docker.yml: chercher les noms d'artefacts GitHub, Verify context cherche les binaires
- Turn.yml: matrix `amd64`/`arm64` → artefacts `nook-turn-server-amd64` / `nook-turn-server-arm64`
- Ne JAMAIS utiliser `cargo zigcheck` ou `cargo zigclippy` (n'existent pas)
- `cargo-zigbuild` + cargo zigbuild = incompatible Rust 1.95 / Zig 0.13 sur GitHub runners
- Backend CI: natif + `musl-tools` sur ubuntu-latest / ubuntu-24.04-arm (pas zig ni cross)
- TOUS les containers doivent utiliser UID/GID 1000 (match host casaos user)
- turn-server : `--config /etc/turn-server/config.toml` obligatoire (TOML format, pas coturn)
- Volume turn-config doit être `:rw` (pas `:ro`) pour l'init automatique
- Template dans `/opt/turn-server/` (survit au volume mount)

## Notifications
- Système in-app fonctionne sur HTTP/LAN via AudioContext
- Web Push nécessite HTTPS — non disponible en LAN HTTP
- notificationStore.svelte.ts : store central, NotificationToast.svelte : composant
- Chaque module importe ses helpers: notifyMessage, notifyChess, notifyPoll, etc.

## 🔄 Redéploiement
- ⚠️ **Seul Mo Ju peut redéployer** — pas d'auto-redéploiement ni de scripts de déploiement non supervisés
- Le redéploiement doit toujours passer par Mo Ju pour éviter les incohérences de version et les pertes de données

## 🏗️ Orchestrator vs Front-end
- ⚠️ **L'orchestrateur ne code pas de front-end** — les interfaces Svelte 5 requièrent une connaissance approfondie des runes `$state`, `$derived.by`, `$props()` et de la syntaxe Svelte 5 spécifique
- Les tentatives d'orchestrator pour modifier le frontabout aboutissent souvent à une syntaxe Svelte 4 invalide ou à des `$state` mal placés

## 🎭 Playwright & Symlinks
- ⚠️ **Playwright + chmod+symlinks** : Lorsque Playwright teste des fichiers via des symlinks, les permissions `chmod` peuvent ne pas se propager correctement
- Vérifier que les chemins de test pointent vers des réels ou utiliser `test.fixme()` pour contourner les problèmes de permissions
- Toujours valider `npx playwright test --list` avant push (règle #1)

## 🔗 PLUR symlink
- ⚠️ **PLUR symlink ~/.plur** : Le répertoire `~/.plur` doit être en symlink vers le repo Nook `.herms/plur/` pour que les engrammes soient partagés entre sessions
- Sans ce symlink, chaque session PLUR a sa propre copie locale, causant des incohérences de mémoire partagée

# Svelte 5 Syntax
