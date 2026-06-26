# Session 56 — Events Router State Bug Fix

## Contexte
Bug critique : `/api/events` GET retourne 500 `Missing request extension: Extension of type alloc::sync::Arc<sqlx_core::pool::Pool<sqlx_sqlite::database::Sqlite>>` alors que `/api/polls` fonctionne correctement avec le même pattern.

---

## Diagnostic

### Ce qui ne marchait PAS (Session 55 fix incomplet)
```rust
// main.rs — tentative de fix via Extension
.nest("/api/events", events::router().layer(Extension(db.clone())))
```
→ Ne résout pas le problème car les handlers events utilisent `State<Arc<SharedState>>` pas `Extension<Pool<Sqlite>>`.

### Ce qui marchait pour polls (pattern correct)
```rust
// polls.rs — handlers utilisent State(state): State<Arc<SharedState>>
// polls_routes() retourne Router<Arc<SharedState>>
// Dans main.rs, polls est mergé dans protected_routes qui a .with_state(shared_state)
.merge(polls::polls_routes())  // fonctionne
```

---

## Cause racine

Le router `protected_routes` dans `main.rs` (lignes 489-527) **n'a pas** `.with_state(shared_state.clone())` déclaré explicitement.

```rust
// main.rs aktuell
let protected_routes = Router::new()
    .merge(admin_routes)
    .route("/auth/me", ...)
    // ... autres routes ...
    .merge(events::events_routes())    // Router<Arc<SharedState>>
    .merge(polls::polls_routes())      // Router<Arc<SharedState>>
    .merge(chess::chess_routes())      // Router<Arc<SharedState>>
    // ...
    .layer(middleware::from_fn_with_state(
        shared_state.clone(),
        auth::require_auth,
    ));
    // MANQUANT : .with_state(shared_state.clone())
```

Les `layer()` avec `from_fn_with_state` ne propagent **pas** le state aux sous-routers mergés. Il faut `.with_state()` sur le router parent.

---

## Fix

```rust
// main.rs — ajouter .with_state() AVANT le layer auth
let protected_routes = Router::new()
    .merge(admin_routes)
    .route("/auth/me", get(auth::me))
    // ... toutes les routes ...
    .merge(events::events_routes())
    .merge(polls::polls_routes())
    .merge(chess::chess_routes())
    .merge(e2ee::e2ee_routes())
    .merge(reactions::reactions_routes())
    .merge(webrtc::webrtc_routes())
    .merge(missed_calls::missed_calls_routes())
    .merge(search::search_routes())
    .merge(presence::presence_routes())
    .with_state(shared_state.clone())  // OBLIGATOIRE pour State extractor
    .layer(middleware::from_fn_with_state(
        shared_state.clone(),
        auth::require_auth,
    ));
```

---

## Vérification

Après fix :
1. `./events` (GET, POST, PATCH, DELETE) → 200 OK
2. `./polls` → continue de marcher
3. `./chess` → continue de marcher
4. Tous les autres endpoints protégés → OK

---

## Leçon générale

| Pattern | Fonctionne ? | Note |
|---------|--------------|------|
| `.layer(middleware::from_fn_with_state(state, mw))` + merge subs | Non | State non propagé aux sous-routers |
| `.with_state(state)` + `.layer(...)` + merge subs | Oui | State disponible pour tous les extracteurs `State<T>` |
| `.layer(Extension(db.clone()))` sur sous-router | Partiel | Marche seulement pour `Extension<Pool>` handlers |

**Règle** : Tout router parent qui merge des sous-routers attendant `State<T>` doit avoir `.with_state(state)` déclaré **avant** toute middleware layer.