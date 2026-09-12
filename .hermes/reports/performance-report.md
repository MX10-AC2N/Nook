# ⚡ Performance Report — Nook (2026-05-04)

## Summary
- **Frontend Bundle**: Largest chunk 939kB (300kB gzipped) exceeds 600kB raw threshold
- **Backend API**: Benchmark skipped (backend build timed out)
- **Database**: Core queries properly indexed, no critical gaps found
- **Unused Dependencies**: 1 confirmed frontend unused dep, backend needs deeper check

## Audit Results

### 1. Frontend Bundle Size (Target: <500kB gzipped, <600kB raw)
| Chunk | Raw Size | Gzipped | Status |
|-------|----------|---------|--------|
| `HEavZsIZ.js` (libsodium-wrappers) | 939.20 kB | 299.50 kB | ❌ Over 600kB raw |
| `aIWNwWfY.js` | 200.50 kB | 67.74 kB | ✅ OK |
| `2X8YI2Tx.js` | 75.42 kB | 28.55 kB | ✅ OK |
| Other chunks | <100kB each | <30kB each | ✅ OK |

#### Root Cause (P1 Issue)
- `libsodium-wrappers` is statically imported in `src/lib/e2ee.ts` and `src/lib/file-transfer.svelte.ts`
- Should use dynamic imports (consistent with other modules like `sodium.svelte.js`)
- Causes oversized chunk despite gzipped size meeting target

### 2. Backend API Response Times (Target: TTFB <200ms, p95 <100ms)
- **Status**: Incomplete (backend `cargo build --release` timed out after 300s)
- **Attempted**: Installed `wrk` for load testing, but backend not started
- **Note**: Backend uses properly indexed SQLite queries (see Database section)

### 3. Database Query Optimization (SQLite)
| Query | Index Used | Status |
|-------|------------|--------|
| `SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 50` | `idx_messages_conversation (conversation_id=?)` | ✅ Optimal |
| Conversation join (participants) | Covering indexes on `conversation_participants` and `conversations` | ✅ Optimal |

- **Indexes Present**:
  - `idx_messages_conversation` on `(conversation_id, created_at DESC)`
  - `idx_conversations_updated_at` on `updated_at DESC`
  - `sqlite_autoindex_conversation_participants_1` (primary key covering)
- **No critical missing indexes found** in core queries

### 4. Unused Dependencies
#### Frontend (confirmed)
| Dependency | Status | Action |
|------------|--------|--------|
| `simple-peer` (^9.11.1) | ❌ Unused (replaced by native WebRTC per `src/lib/webrtc.ts`) | Remove from `package.json` (P2) |

#### Backend (needs `cargo-udeps` for full check)
- `cargo-udeps` install timed out; manual review found no obvious unused deps
- `reqwest` is used in `push.rs` and `gifs_updater.rs`; all listed deps appear used

## Issues by Priority

### P1 (High Priority)
1. **Oversized libsodium chunk**: Convert static imports of `libsodium-wrappers` to dynamic imports in `e2ee.ts` and `file-transfer.svelte.ts` to split the 939kB chunk. Expected reduction: ~600kB from main bundle.

### P2 (Medium Priority)
1. **Unused frontend dependency**: Remove `simple-peer` from `frontend/package.json` (not used, replaced by native WebRTC)
2. **Backend build timeout**: Investigate slow `cargo build` (possible network/resource issue) to enable API benchmarking
3. **Unused CSS warnings**: 50+ unused CSS selectors in `chat/+page.svelte` (not performance-critical but indicates dead code)

### P0 (Critical)
- None identified in this audit

## Files Modified
- `/opt/data/home/.hermes/Nook/frontend/vite.config.js`: Temporarily added `rollup-plugin-visualizer` for bundle analysis (reverted)
- Created `/opt/data/home/.hermes/Nook/backend/.env` for testing
- Created `/tmp/nook-test.db` for SQLite analysis

## Limitations
- Backend API benchmarking incomplete due to build timeout
- Full backend unused dependency check blocked by `cargo-udeps` install timeout
- Bundle analysis only covers production build, not runtime lazy loading efficiency

## Recommended Next Steps
1. Fix libsodium imports to use dynamic `import()` instead of static `import`
2. Remove `simple-peer` from frontend dependencies
3. Resolve backend build timeout to complete API benchmarking
4. Run `cargo-udeps` when build environment is stable to confirm backend unused deps
