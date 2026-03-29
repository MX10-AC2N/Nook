# Rust - Ressources pour le Développement Web

Ce fichier recense les ressources utiles pour le développement d'applications web et de services en Rust.

## Frameworks Web

### Frameworks Principaux

**[Axum](https://github.com/tokio-rs/axum)** - Framework web créé par l'équipe de Tokio

| Aspect | Détails |
|--------|---------|
| **Créé par** | Équipe Tokio |
| **Performance** | Haute performance, async natif |
| **Ergonomie** | API moderne et intuitive |
| **Intégration** | Tower middleware, Tokio |
| **Typed** | Request/Response typesafe |
| **Documentation** | Excellente |

**[Actix Web](https://github.com/actix/actix-web)** - Framework haute performance

| Aspect | Détails |
|--------|---------|
| **Performance** | Leader des benchmarks |
| **Actix** | Basé sur le framework Actix |
| **Features** | WebSocket, HTTP/2, middleware |
| **Type-safe** | Extraction de données typée |
| **Écosystème** | Large communauté |

**[Rocket](https://github.com/rust-flash/rocket)** - Framework ergonomique

| Aspect | Détails |
|--------|---------|
| **Simplicité** | API simple et expressive |
| **Type-safe** | Routes typées |
| **Validation** | FromForm pour validation |
| **Testing** | Support de test intégré |
| **Stable** | API stable |

**[Warp](https://github.com/seanmonstar/warp)** - Framework filtré

| Aspect | Détails |
|--------|---------|
| **Filtres** | Système de filtres composables |
| **Hyper** | Basé sur Hyper |
| **Flexible** | Haute configurabilité |
| **Type-safe** | Filtres typés |

**[Poem](https://github.com/poem-web/poem)** - Framework full-featured

| Aspect | Détails |
|--------|---------|
| **OpenAPI** | Génération automatique OpenAPI |
| **gRPC** | Support gRPC natif |
| **WebSocket** | Support WebSocket |
| **JWT** | Authentification JWT |

## Runtime Asynchrone

**[Tokio](https://github.com/tokio-rs/tokio)** - Runtime async standard

| Aspect | Détails |
|--------|---------|
| **Status** | De facto standard async Rust |
| **I/O** | Async I/O non-bloquant |
| **Scheduling** | Multi-threaded scheduler |
| **Timers** | Timers asynchrones |
| **Tracing** | Intégration tracing |
| **Usage** | Axum, Hyper, etc. |

## Bases de Données & ORM

**[SQLx](https://github.com/build-trust/codebase)** - ORM async compile-time checked

| Aspect | Détails |
|--------|---------|
| **Compile-time** | Requêtes vérifiées à la compilation |
| **Async** | 100% async |
| **Pools** | Connection pooling |
| **Support** | PostgreSQL, MySQL, SQLite |

**[Diesel](https://github.com/diesel-rs/diesel)** - ORM établi

| Aspect | Détails |
|--------|---------|
| **Type-safe** | Queries typées |
| **Migrations** | Système de migrations |
| **Async** | Support async (async-trait) |

**[SeaORM](https://github.com/SeaQL/sea-orm)** - ORM async dynamique

| Aspect | Détails |
|--------|---------|
| **Async** | Entièrement async |
| **Active Record** | Pattern Active Record |
| **Migrations** | Migrations automatiques |

## HTTP & API

**[reqwest](https://github.com/seanmonstar/reqwest)** - Client HTTP

| Aspect | Détails |
|--------|---------|
| **Async** | API async |
| **TLS** | Support HTTPS |
| **JSON** | Désérialization automatique |

**[Hyper](https://github.com/hyperium/hyper)** - HTTP library basse niveau

| Aspect | Détails |
|--------|---------|
| **Low-level** | HTTP implementation |
| **HTTP/1** | HTTP/1.1 |
| **HTTP/2** | HTTP/2 |

**[GraphQL Rust](https://github.com/async-graphql/async-graphql)** - Implémentation GraphQL

| Aspect | Détails |
|--------|---------|
| **Schema** | Schema-first ou code-first |
| **Subscriptions** | Support WebSocket |
| **Introspection** | Introspection GraphQL |

**[jsonwebtoken](https://github.com/Keats/jsonwebtoken)** - JWT en Rust

| Aspect | Détails |
|--------|---------|
| **Claims** | Validation des claims |
| **Algorithms** | HS256, RS256, etc. |

## WebSocket & Temps Réel

**[tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)** - WebSocket pour Tokio

| Aspect | Détails |
|--------|---------|
| **Async** | Entièrement async |
| **TLS** | Support WSS |

**[rustrtc](https://github.com/restsend/rustrtc)** - WebRTC haute performance

| Aspect | Détails |
|--------|---------|
| **Performance** | ~2.8x plus rapide que pion |
| **SFU** | Serveur visioconférence |
| **STUN/TURN** | Support STUN/TURN |

**[turn-rs](https://github.com/mycrl/turn-rs)** - Serveur TURN/STUN

| Aspect | Détails |
|--------|---------|
| **STUN** | RFC 3489, 5389 |
| **TURN** | RFC 5766, 6062 |
| **Performance** | 40M msg/sec, <35µs latence |

## Frontend & WebAssembly

**[Leptos](https://github.com/leptos-rs/leptos)** - Framework frontend reactif

| Aspect | Détails |
|--------|---------|
| **SSR** | Server-side rendering |
| **SPA** | Single page app |
| **Fine-grained** | Réactivité fine |

**[Dioxus](https://github.com/DioxusLabs/dioxus)** - Framework UI moderne

| Aspect | Détails |
|--------|---------|
| **Multi-platform** | Web, Desktop, Mobile |
| **RSX** | Syntaxe JSX-like |
| **Popularité** | En croissance rapide |

**[Yew](https://github.com/yewstack/yew)** - Framework frontend

| Aspect | Détails |
|--------|---------|
| **Components** | Composants comme React |
| **WebAssembly** | Compile vers WASM |

## Testing

**[rstest](https://github.com/la10736/rstest)** - Tests paramétrés

| Aspect | Détails |
|--------|---------|
| **Parametric** | Tests paramétrés |
| **Fixtures** | Fixtures async |

**[proptest](https://github.com/proptest-rs/proptest)** - Property-based testing

| Aspect | Détails |
|--------|---------|
| **Property** | Property-based testing |
| **Arbitrary** | Génération de données |

**[wiremock](https://github.com/MatejLach/wiremock-rs)** - Mock HTTP server

| Aspect | Détails |
|--------|---------|
| **Mock** | Mock server |
| **Testing** | Tests d'intégration |

## Logging & Observability

**[tracing](https://github.com/tokio-rs/tracing)** - Framework de tracing

| Aspect | Détails |
|--------|---------|
| **Structured** | Logging structuré |
| **Spans** | Distributed tracing |
| **Metrics** | Instrumentation |

**[metrics](https://github.com/metrics-rs/metrics)** - Instrumentation metrics

| Aspect | Détails |
|--------|---------|
| **Counter** | Counters |
| **Gauge** | Gauges |
| **Histogram** | Histograms |
| **Prometheus** | Export Prometheus |

## CLI & Tools

**[Clap](https://github.com/clap-rs/clap)** - Parser d'arguments

| Aspect | Détails |
|--------|---------|
| **Derive** | Derive macros |
| **Typed** | Arguments typés |
| **Completions** | Shell completions |

## Learning Resources

**[The Rust Programming Language](https://doc.rust-lang.org/book/)** - Livre officiel

**[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** - Apprendre par l'exemple

**[Rustlings](https://github.com/rust-lang/rustlings)** - Exercices interactifs

## Listes de Ressources

**[awesome-rust](https://github.com/rust-unofficial/awesome-rust)** - Liste complète (25k+ stars)

**[Rust Frameworks List](https://github.com/venuswhispers/Rust-awesome-frameworks)** - Comparaison frameworks

**[Rust Security](https://github.com/osirislab/awesome-rust-security)** - Outils de sécurité

## Notes

- **Last updated**: 2026-03-29
- **Focus**: Web development, API, services
- **async-std**: Déprécié en 2025 - utiliser Tokio
- **À revisar**: Trimestriellement pour nouvelles ressources
