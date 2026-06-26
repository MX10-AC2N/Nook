# Axum 0.8 Middleware State Propagation Pattern

## The Problem

When using `middleware::from_fn_with_state(state, handler)` to protect routes, sub-routers merged into the protected router that expect `State<Arc<SharedState>>` in their handlers **don't receive the state** if:

1. The parent router doesn't have `.with_state(state)` declared, AND
2. The middleware consumes the state without re-propagating it

## What DOESN'T Work

### Attempt 1: Just add `.with_state()` on parent router
```rust
// main.rs
let protected_routes = Router::new()
    .merge(events::events_routes())   // Router<Arc<SharedState>>
    .merge(polls::polls_routes())     // Router<Arc<SharedState>>
    .with_state(shared_state.clone())  // ← This alone is NOT enough
    .layer(middleware::from_fn_with_state(
        shared_state.clone(),
        auth::require_auth,
    ));
```
**Result**: Compiles but runtime 500 "Missing request extension: Pool<Sqlite>" — the middleware handler with `AxumState` extractor doesn't forward state to sub-routers.

### Attempt 2: Use closure with `from_fn_with_state`
```rust
.layer(middleware::from_fn_with_state(shared_state.clone(), |state, req, next| {
    auth::require_auth(state, req, next)
}))
```
**Result**: `cargo build --release` FAILS with LTO + clippy: `FromFn<{closure}...>: Service<...> not implemented`

## What WORKS — Canonical Pattern

### Step 1: Middleware handler uses `AxumState` extractor (not direct param)
```rust
// auth.rs
pub async fn require_auth(
    AxumState(state): AxumState<Arc<SharedState>>,  // ← Extractor pattern
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // ... validate session, get user ...
    
    if let Some(user) = user {
        // KEY: Manually re-insert state for downstream handlers
        req.extensions_mut().insert(state.clone());  // ← CRITICAL
        req.extensions_mut().insert(CurrentUser(user));
        return next.run(req).await;
    }
    
    // ... 401 response ...
}
```

### Step 2: Register middleware with function item (not closure)
```rust
// main.rs
let protected_routes = Router::new()
    .merge(events::events_routes())    // Router<Arc<SharedState>>
    .merge(polls::polls_routes())      // Router<Arc<SharedState>>
    .merge(chess::chess_routes())      // Router<Arc<SharedState>>
    .layer(middleware::from_fn_with_state(
        shared_state.clone(),
        auth::require_auth,  // ← Function item, NOT closure
    ));
    // NO .with_state() needed on parent — middleware injects it
```

### Step 3: Sub-router handlers use `State` extractor normally
```rust
// events.rs (and polls.rs, chess.rs, etc.)
pub async fn list_events(
    State(state): State<Arc<SharedState>>,  // ← Works!
    Extension(CurrentUser(user)): Extension<CurrentUser>,
    ...
) -> impl IntoResponse {
    // state.db is available ✅
}
```

## Why This Works

| Component | Role |
|-----------|------|
| `from_fn_with_state(state, fn_item)` | Injects `state` into middleware call via `AxumState` extractor |
| `AxumState(state)` in middleware | Pulls injected state from request extensions |
| `req.extensions_mut().insert(state.clone())` | Re-inserts state for downstream handlers/extractors |
| Function item (not closure) | Satisfies `tower::Service` bounds in release LTO |

## Verification Checklist

- [ ] `cargo build --release --target x86_64-unknown-linux-musl` ✅
- [ ] `cargo build --release --target aarch64-unknown-linux-musl` ✅  
- [ ] `cargo clippy` → 0 warnings
- [ ] Runtime: `GET /api/events` → 200 OK (not 500)
- [ ] Runtime: `GET /api/polls` → 200 OK
- [ ] Runtime: `GET /api/chess` → 200 OK
- [ ] All protected routes work

## Related Files Modified

- `backend/src/auth.rs` — middleware handler with state re-insertion
- `backend/src/main.rs` — middleware registration with function item

## Debugging Tip

If you see `Missing request extension: ... Pool<Sqlite>` or `Missing request extension: ... SharedState`:
1. Check middleware is registered with **function item** (not closure)
2. Check middleware handler uses **`AxumState` extractor**
3. Check middleware **re-inserts state** via `req.extensions_mut().insert(state.clone())`
4. Verify sub-router handlers use **`State<Arc<SharedState>>`** (not `Extension`)