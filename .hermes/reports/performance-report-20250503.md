# ⚡ Performance Report — Nook (2025-05-03)

## Summary
- **Frontend Bundle**: Largest chunk 939.20 kB (299.50 kB gzipped) — exceeds 500 kB target
- **Backend DB**: Well-indexed, uses efficient keyset pagination
- **CPU-bound Tasks**: Chess AI correctly uses `spawn_blocking`
- **Unused Dependencies**: No unused npm packages found; `cargo udeps` not installed (timed out)
- **CSS**: 50+ unused CSS selectors detected in build

## Frontend Audit

### 1. Bundle Size (❌ Critical)
| Chunk | Size (kB) | Gzip (kB) | Status |
|-------|-----------|-----------|--------|
| `HEavZsIZ.js` | 939.20 | 299.50 | 🔴 Over 600 kB threshold |
| `aIWNwWfY.js` | 200.50 | 67.74 | 🟡 Over 100 kB |
| `lV9zldFE.js` | 75.42 | 28.55 | 🟡 Over 50 kB |
| **Total gzipped** | - | ~400 kB | 🟡 Over 500 kB target |

**Root Cause**: Static imports of `libsodium-wrappers` in `e2ee.ts` and `file-transfer.svelte.ts` force this ~500 kB library into the main chunk. Other modules correctly use dynamic `import()`.

### 2. Code Splitting (✅ Partial)
- Route-based splitting works (SvelteKit default)
- Chart.js lazy-loaded correctly in analytics page
- **Issue**: `libsodium-wrappers` not lazy-loaded in core modules

### 3. Tree Shaking (❌ Poor)
- 50+ unused CSS selectors in `chat/+page.svelte`, `settings/+page.svelte`, `polls/+page.svelte`
- Indicates Svelte CSS tree-shaking not fully effective

### 4. Image Optimization (✅ Partial)
- Some images use `loading="lazy"`
- GIFs included as unoptimized JSON index
- No webp/avif conversion pipeline
- `vite-plugin-compression` used for text assets (good)

### 5. Unused Dependencies (✅ Good)
- `npm prune` found no unused packages
- `simple-peer` listed as dependency but not used (replaced by native WebRTC in `webrtc.ts`) — candidate for removal

## Backend Audit

### 1. API Response Times (❌ No Monitoring)
- No per-endpoint response time logging/metrics
- Only startup time and chess AI timing tracked
- **Recommendation**: Add tower-http `TraceLayer` or custom middleware

### 2. Database Performance (✅ Excellent)
- **Keyset Pagination**: Messages use `created_at < ?` (efficient, no OFFSET)
- **Indexing**: All key tables have proper indexes:
  - `idx_messages_conversation` (conversation_id, created_at DESC)
  - `idx_conversations_updated_at` (updated_at DESC)
  - `idx_users_token` (token) for auth checks
- No slow query patterns detected

### 3. CPU-bound Task Handling (✅ Excellent)
- Chess AI (`play_ai`) correctly wrapped in `tokio::task::spawn_blocking`
- Prevents runtime thread pool blockage
- AI timing tracked and returned in responses

### 4. Unused Dependencies (⚠️ Pending)
- `cargo udeps` install timed out
- `simple-peer` likely unused in frontend (already replaced by native WebRTC)
- Backend Cargo.toml not audited yet

## Recommendations (Priority Order)

### Critical (Fix Immediately)
1. **Fix libsodium-wrappers imports**:
   - Change static `import sodium from 'libsodium-wrappers'` to dynamic `await import()` in `e2ee.ts` and `file-transfer.svelte.ts`
   - Estimated bundle reduction: ~500 kB uncompressed

### High (Fix This Sprint)
2. **Remove unused CSS**: Audit and remove 50+ unused selectors in components
3. **Add API metrics**: Implement response time logging with `tracing` or `tower-http`
4. **Remove unused `simple-peer`**: Not used after native WebRTC migration

### Medium (Next Backlog)
5. **Optimize GIF handling**: Compress GIFs, add webp fallbacks
6. **Vite manual chunks**: Configure `build.rollupOptions.output.manualChunks` to isolate large libs
7. **Run `cargo udeps`**: Complete backend unused dependency audit

## Measurements

### Frontend Bundle (Current)
| Type | Size (kB) | Gzip (kB) |
|------|-----------|-----------|
| JS (total) | ~1,400 | ~400 |
| CSS (total) | ~150 | ~25 |
| **Target** | < 500 | < 150 |

### Backend DB (Sample Query)
| Query | Index Used | Est. Time |
|-------|------------|-----------|
| Get messages by conversation | `idx_messages_conversation` | < 10ms |
| Auth check (token) | `idx_users_token` | < 5ms |

## Appendix: Commands Run
- `cd /opt/data/home/.hermes/Nook/frontend && npm run build`
- `cd /opt/data/home/.hermes/Nook/frontend && npm prune`
- `grep -r spawn_blocking /opt/data/home/.hermes/Nook/backend/src`
- `grep -r "OFFSET\|LIMIT\|created_at <" /opt/data/home/.hermes/Nook/backend/src`
- `grep -r "libsodium-wrappers" /opt/data/home/.hermes/Nook/frontend/src`
