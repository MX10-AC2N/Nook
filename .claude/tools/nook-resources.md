# Nook - Ressources de Développement

Ce fichier recense les ressources spécifiques pour le développement et l'optimisation de **Nook** (messagerie familiale auto-hébergée).

## Stack Technique

| Composant | Technologies | Ressources |
|-----------|--------------|------------|
| **Backend** | Rust + Axum 0.8 | `rust-resources.md` |
| **Frontend** | SvelteKit 5 Runes | `svelte5-resources.md` |
| **Base de données** | SQLite | Section Base de données |
| **Conteneurisation** | Docker + distroless | Section Docker |
| **Testing** | Playwright | Section Testing |
| **Temps réel** | WebSocket | Section WebSocket |
| **Appels** | WebRTC | Section WebRTC |

---

## 🔐 Cryptographie & Sécurité

### E2EE (X25519)

**[dalek-cryptography/x25519-dalek](https://github.com/dalek-cryptography/x25519-dalek)** - Implémentation X25519

| Aspect | Détails |
|--------|---------|
| **Usage** | Échange de clés pour E2EE |
| **Constant-time** | Protégé contre timing attacks |
| **No_std** | Support embedded |

**[curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek)** - Opérations X25519

| Aspect | Détails |
|--------|---------|
| **Operations** | Arithmétique sur courbe25519 |
| **SIMD** | Support AVX2/AVX512 |

### Chiffrement Fichiers (XChaCha20)

**[sodiumoxide/sodiumoxide](https://github.com/sodiumoxide/sodiumoxide)** - Bindings libsodium

| Aspect | Détails |
|--------|---------|
| **XChaCha20** | Chiffrement de flux |
| **Poly1305** | Authentification |
| **Blake2** | Hashing |
| **Argon2id** | Hashage mots de passe |

**[orion-rs/orion](https://github.com/orion-rs/orion)** - Cryptographie auditee

| Aspect | Détails |
|--------|---------|
| **XChaCha20-Poly1305** | AEAD |
| **Auditée** | Audité par Trail of Bits |
| **No_std** | Sans dépendances |

**[rust-argon2](https://github.com/Emilos/argon2)** - Argon2id

| Aspect | Détails |
|--------|---------|
| **Password** | Hashage de mots de passe |
| **Memory-hard** | Résistant aux GPU |

### Sécurité Web

**[TLS Config](https://docs.rs/rustls/latest/rustls/)** - TLS moderne

| Aspect | Détails |
|--------|---------|
| **rustls** | TLS moderne en Rust |
| **No OpenSSL** | Alternative sécurisée |

**[CSP Header](https://developer.mozilla.org/fr/docs/Web/HTTP/CSP)** - Content Security Policy

| Aspect | Détails |
|--------|---------|
| **XSS** | Protection XSS |
| **Inline** | Contrôle scripts inline |

---

## 💬 WebSocket & Temps Réel

### Serveur WebSocket (Axum)

**[axum-websockets](https://docs.rs/axum/latest/axum/extract/struct.WebSocketUpgrade.html)** - WebSocket dans Axum

| Aspect | Détails |
|--------|---------|
| **axum 0.8** | Support natif |
| **Sink/Stream** | Bidirectional |
| **Protocol** | Protocol upgrade |

**[tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)** - WebSocket async

| Aspect | Détails |
|--------|---------|
| **tokio** | Intégration Tokio |
| **Async** | 100% async |
| **TLS** | WSS support |

### Protocoles de Chat

**[MessagePack](https://github.com/3Hren/msgpack-rust)** - Format de message binaire

| Aspect | Détails |
|--------|---------|
| **Compact** | Plus petit que JSON |
| **Fast** | Sérialization rapide |
| **Rust** | Implementation native |

**[serde_json](https://github.com/serde-rs/json)** - JSON serialization

| Aspect | Détails |
|--------|---------|
| **Standard** | JSON pour API REST |
| **Typed** | Deserialize vers structs |

---

## 📞 WebRTC (Appels Audio/Vidéo)

### WebRTC Rust

**[rustrtc](https://github.com/restsend/rustrtc)** - WebRTC haute performance

| Aspect | Détails |
|--------|---------|
| **Performance** | ~2.8x plus rapide que pion |
| **SFU** | Serveur visioconférence |
| **DataChannel** | Data channels |
| **Licence** | MIT |

**[rini](https://github.com/rtcp-livewebrtc/rini)** - WebRTC simple

| Aspect | Détails |
|--------|---------|
| **Simple** | API simple |
| **P2P** | Peer-to-peer |

### STUN/TURN (Traverse NAT)

**[turn-rs](https://github.com/mycrl/turn-rs)** - Serveur TURN/STUN

| Aspect | Détails |
|--------|---------|
| **STUN** | RFC 5389 |
| **TURN** | RFC 5766 |
| **Performance** | 40M msg/sec |
| **gRPC** | API control |

**[candid](https://github.com/rtcp-livewebrtc/candid)** - ICE candidate

| Aspect | Détails |
|--------|---------|
| **ICE** | Interactive Connectivity Establishment |
| **Candidate** | Gestion candidats |

### WebRTC Frontend

**[adapter.js](https://github.com/webrtc/adapter)** - Polyfill WebRTC

| Aspect | Détails |
|--------|---------|
| **Browser** | Compatibilité navigateurs |
| **Shim** | Normalise API |
| **Chrome** | Support Chrome/Safari |

**[simple-peer](https://github.com/feross/simple-peer)** - WebRTC peer-to-peer

| Aspect | Détails |
|--------|---------|
| **JS** | Pour frontend |
| **Signaling** | Nécessite serveur signalisation |
| **API** | Simple à utiliser |

---

## 🗄️ Base de Données (SQLite)

### ORM & Drivers

**[rusqlite](https://github.com/rusqlite/rusqlite)** - Driver SQLite

| Aspect | Détails |
|--------|---------|
| **Sync** | API synchrone |
| **Async** | Via sqlite-pool |
| **Bundled** | SQLite embarqué |

**[sqlx](https://github.com/build-trust/codebase)** - ORM async

| Aspect | Détails |
|--------|---------|
| **Compile-time** | Requêtes vérifiées |
| **SQLite** | Support SQLite |
| **Async** | Tokio support |

**[sea-orm](https://github.com/SeaQL/sea-orm)** - ORM async

| Aspect | Détails |
|--------|---------|
| **Active Record** | Pattern |
| **Migration** | Migrations automatiques |
| **Async** | Tokio/async-std |

### Migrations

**[sqlx-cli](https://github.com/build-trust/codebase)** - Outil de migrations

| Aspect | Détails |
|--------|---------|
| **CLI** | Commandes terminal |
| **Migrate** | Créer/appliquer migrations |

### Exploration

**[sqlite-browser](https://github.com/oseiskar/sqlite-web)** - Interface web SQLite

| Aspect | Détails |
|--------|---------|
| **Web** | Admin interface |
| **Read-only** | Visualisation |
| **Rust** | Implémentation Rust |

---

## 🔗 Networking P2P

**[rust-libp2p](https://github.com/libp2p/rust-libp2p)** - Framework réseau peer-to-peer

| Aspect | Détails |
|--------|---------|
| **Réseau** | Stack réseau P2P complet |
| **Modular** | Transport, muxers, protocoles séparables |
| **WASM** | Support navigateur |
| **Stars** | 5.5k |
| **Usage** | ~23,000 repositories |

**Protocoles supportés :**

| Protocole | Usage pour Nook |
|-----------|----------------|
| **mDNS** | Découverte locale appareils familiaux |
| **Gossipsub** | Messages temps réel sans serveur central |
| **Kademlia** | Partage clés E2EE distribué |
| **Noise** | Chiffrement connexions P2P |
| **WebRTC** | Alternative aux appels existants |

**Cas d'usage concrets :**
- Appareils se découvrent automatiquement sur le LAN (mDNS)
- Messages diffusés sans passer par le serveur (Gossipsub)
- Clés de chiffrement partagées de façon décentralisée (Kademlia)

---

## 🧪 Testing

### Playwright (E2E)

**[Playwright](https://playwright.dev/)** - Tests E2E

| Aspect | Détails |
|--------|---------|
| **SvelteKit** | Support natif |
| **Cross-browser** | Chrome, Firefox, Safari |
| **API** | API moderne |
| **Trace** | Debugging visuel |

**[playwright-rs](https://github.com/microsoft/playwright-rs)** - Playwright pour Rust

| Aspect | Détails |
|--------|---------|
| **Rust** | API Rust |
| **E2E** | Tests browser |
| **API** | Backend testing |

### Tests Backend

**[rstest](https://github.com/la10736/rstest)** - Tests paramétrés

**[proptest](https://github.com/proptest-rs/proptest)** - Property-based testing

**[wiremock](https://github.com/MatejLach/wiremock-rs)** - Mock HTTP

### Tests WebSocket

**[tokio-test](https://github.com/tokio-rs/tokio)** - Tests async

**[websocket-client](https://github.com/食指16/websocket-client-rs)** - Client WebSocket test

---

## 🐳 Docker & Déploiement

### Optimisation Image

**[distroless](https://github.com/GoogleContainerTools/distroless)** - Images minimalistes

| Aspect | Détails |
|--------|---------|
| **No shell** | Pas de shell |
| **No pkg manager** | Surface d'attaque réduite |
| **Multi-arch** | amd64, arm64 |

**[docker-buildx](https://github.com/docker/buildx)** - Build multi-plateforme

| Aspect | Détails |
|--------|---------|
| **Multi-arch** | amd64 + arm64 |
| **Cache** | Build cache |
| **Push** | GitHub Packages |

### Monitoring & Observabilité

> **📖 Voir fichier dédié :** `monitoring-ressources.md`

| Outil | Type | Interface | Best for |
|-------|------|-----------|----------|
| **Beszel** | Dashboard | Web | Multi-serveurs (déjà utilisé) |
| **rustmon** | Dashboard | Web (React) | Dashboard admin |
| **monitor-rs** | TUI | Terminal | Debug SSH |
| **sysinfo** | Library | N/A | Intégration custom |

**Intégration suggérée pour Nook :**
- Beszel pour monitoring global
- Intégrer `/api/metrics` via sysinfo dans Nook
- Alertes via notifications push existantes

### Reverse Proxy

**[nginx-proxy/acme-companion](https://github.com/nginx-proxy/acme-companion)** - Nginx + Let's Encrypt

**[Traefik](https://github.com/traefik/traefik)** - Reverse proxy moderne

**[Caddy](https://github.com/caddyserver/caddy)** - HTTPS automatique

---

## 📱 Notifications Push

### web-push

**[web-push](https://github.com/Minishell/node-web-push)** - Notifications push

| Aspect | Détails |
|--------|---------|
| **VAPID** | Authentification VAPID |
| **FCM** | Firebase Cloud Messaging |
| **Rust** | Package Rust disponible |

**[actix-web-push](https://github.com/Stebalien/actix-web-push)** - Push dans Actix

**[axum-web-push](https://github.com/zRooTzN/axum-web-push)** - Push dans Axum

### Services

**[OneSignal](https://onesignal.com/)** - Alternative gratuite

**[Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging)** - Google FCM

**[Pusher](https://pusher.com/)** - Service de push

---

## 🎨 UI/UX

### Thèmes & Styling

**[Tailwind CSS](https://tailwindcss.com/)** - CSS utilitaire

| Aspect | Détails |
|--------|---------|
| **SvelteKit** | Support natif |
| **Custom** | Thèmes personnalisables |
| **Dark mode** | Mode sombre intégré |

**[Radix Colors](https://www.radix-ui.com/colors)** - Système de couleurs

| Aspect | Détails |
|--------|---------|
| **Accessible** | Contraste WCAG |
| **Palette** | 24 couleurs |
| **CSS** | Variables CSS |

### Animations

**[AutoAnimate](https://auto-animate.formkit.com/)** - Animations listes

**[svelte-motion](https://github.com/Psychopatoecci/svelte-motion)** - Animations Svelte

---

## 📊 Calendrier

### Bibliothèques

**[V Calendar](https://vcalendar.io/)** - Calendrier pour Vue/Svelte

**[FullCalendar](https://fullcalendar.io/)** - Calendrier complet

| Aspect | Détails |
|--------|---------|
| **Events** | Gestion d'événements |
| **iCal** | Support iCal |
| **Drag & Drop** | Interface intuitive |

**[Svelte Calendar](https://github.com/bsides/svelte-calendar)** - Calendrier Svelte

---

## ♟️ Échecs

### Engines UCI

**[Stockfish](https://github.com/official-stockfish/Stockfish)** - Moteur d'échecs

| Aspect | Détails |
|--------|---------|
| **UCI** | Protocol UCI |
| **Strong** | Top engine |
| **WASM** | Support WebAssembly |

**[Walleye](https://github.com/MitchelPaulin/Walleye)** - Moteur Rust UCI

| Aspect | Détails |
|--------|---------|
| **Rust** | Écrit en Rust |
| **UCI** | Protocol UCI |
| **MIT** | Licence MIT |

### Composants UI

**[chessboard.js](https://github.com/oakmac/chessboardjs)** - Échiquier interactif

**[react-chessboard](https://github.com/Clariity/react-chessboard)** - React/Svelte compatible

### Libraries

**[chess.js](https://github.com/jhlywa/chess.js)** - Logique d'échecs

**[shakmaty](https://github.com/nicktasche/shakmaty)** - Bibliothèque Rust pour règles

| Aspect | Détails |
|--------|---------|
| **Legal** | Mouvements légaux |
| **UCI** | Support UCI |
| **Moves** | Parsing moves |

**[chess-tui](https://github.com/thomas-mauran/chess-tui)** - Jeu d'échecs TUI (référence)

---

## 📊 Sondages

### Libraries

**[chart.js](https://www.chartjs.org/)** - Graphiques

**[svelte-chartjs](https://github.com/SauravKanchan/svelte-chartjs)** - Chart.js pour Svelte

---

## 📁 Upload & Médias

### Upload

**[axum-upload](https://docs.rs/axum/latest/axum/multipart/index.html)** - Multipart upload

**[bytes](https://github.com/tokio-rs/bytes)** - Gestion de bytes

**[mime](https://github.com/hyperium/mime)** - Détection MIME

### Images

**[image](https://github.com/image-rs/image)** - Processing image Rust

| Aspect | Détails |
|--------|---------|
| **Resize** | Redimensionnement |
| **Format** | JPEG, PNG, WebP |
| **Thumbnails** | Génération miniatures |

**[squoosh](https://github.com/GoogleChromeLabs/squoosh)** - Compression image

**[sharp](https://github.com/lovell/sharp)** - Node.js image processing

### Stockage

**[local-file-storage](https://github.com/axum-schema-org/local-file-storage)** - Stockage local

**[S3](https://github.com/aws/aws-sdk-rust)** - Compatible S3 (MinIO)

---

## 🔧 Dev Tools

### SvelteKit

**[SvelteKit](https://kit.svelte.dev/)** - Framework officiel

**[svelte-adapter](https://github.com/nicktasche/svelte-adapter)** - Adapters deployment

### Rust Backend

**[cargo-watch](https://github.com/watchexec/cargo-watch)** - Auto-reload

**[cargo-expand](https://github.com/dtolnay/cargo-expand)** - Macro expansion

**[cargo-dist](https://github.com/axodotdev/cargo-dist)** - Distribution

### IDE

**[rust-analyzer](https://rust-analyzer.github.io/)** - LSP pour Rust

**[Svelte for VS Code](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)** - Extension VS Code

---

## 📚 Documentation

### Books

**[Programming Svelte 5](https://svelte-5.book/)** - Livre officiel

**[Svelte Handbook](https://svelte-handbook.com/)** - Guide complet

**[Rust for Rustaceans](https://nostarch.com/rust-rustaceans)** - Livre Rust intermédiaire

### Blogs

**[Svelte Blog](https://svelte.dev/blog)** - Blog officiel

**[This Week in Svelte](https://this-week-in-svelte.org/)** - Newsletter

**[Rust Blog](https://blog.rust-lang.org/)** - Blog officiel

---

## 🎯 Ressources Spécifiques Nook

### Équivalents de Référence

**[rustchat](https://github.com/rustchatio/rustchat)** - Chat Rust + Vue (Axum, WebSocket, PostgreSQL)

**[videocall-rs](https://github.com/security-union/videocall-rs)** - Visioconférence Rust

### Articles Utiles

**[Secure WebSocket](https://developer.mozilla.org/fr/docs/Web/API/WebSockets_API/Writing_WebSocket_server)** - Serveur WebSocket sécurisé

**[WebRTC Architecture](https://developer.mozilla.org/fr/docs/Web/API/WebRTC_API/Architecture)** - Architecture WebRTC

**[SQLite vs PostgreSQL](https://www.sqlite.org/whentouse.html)** - Quand utiliser SQLite

**[E2EE Best Practices](https://signal.org/docs/specifications/session/)** - Signal E2EE specs

---

## 📝 Checklist Optimisation

### Backend (Rust)

- [ ] Connection pooling SQLite
- [ ] Rate limiting sur endpoints
- [ ] Cache responses frequentes
- [ ] Compression responses (gzip/brotli)
- [ ] Logging structuré (tracing)
- [ ] Health checks endpoint
- [ ] Exposer métriques système (sysinfo + rustmon)

### Frontend (SvelteKit)

- [ ] Lazy loading routes
- [ ] Image optimization
- [ ] Code splitting
- [ ] Prefetch links
- [ ] SSR pour SEO
- [ ] PWA (offline support)
- [ ] Dashboard admin état serveur

### Infrastructure

- [ ] Multi-arch Docker build
- [ ] Health check endpoint
- [ ] Graceful shutdown
- [ ] Log rotation
- [ ] Backup SQLite automatique
- [ ] Monitoring (Beszel / rustmon / monitor-rs)
- [ ] CI/CD optimisé

---

## Notes

- **Last updated**: 2026-03-29
- **Focus**: Ressource spécifiques pour Nook v0.4.0+
- **Priorité**: Cryptographie, WebRTC, WebSocket

## Fichiers Associés

| Fichier | Description |
|---------|-------------|
| `monitoring-ressources.md` | Outils de monitoring et observabilité |
| `rust-resources.md` | Ressources Rust backend |
| `svelte5-resources.md` | Ressources Svelte 5 frontend |
| `github-resources.md` | Projets de référence |
