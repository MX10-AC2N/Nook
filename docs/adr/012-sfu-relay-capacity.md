# ADR-012: Configurable SFU MediaRelay Capacity

**Date**: 2026-07-11
**Status**: Accepted
**Owner**: coder
**Participants**: architect, coder

---

## Context

The SFU (Selective Forwarding Unit) component in `backend/src/sfu.rs` creates a `MediaRelay` for each incoming media track with a hardcoded subscriber capacity of 500:

```rust
let relay = MediaRelay::with_capacity(local_track.clone(), 500);
```

This limits the number of peers that can subscribe to a single track. In larger family group calls or community deployments, this limit may need adjustment without recompiling the binary.

## Decision

Make the relay capacity configurable via the `SFU_RELAY_CAPACITY` environment variable:

- **Default**: `500` (backwards compatible)
- **Env var**: `SFU_RELAY_CAPACITY`
- **Config field**: `Config.sfu_relay_capacity` (type `u32`)
- **SFU state**: `SfuState.relay_capacity` passed through to `setup_peer_events`

### Changes

1. **`backend/src/config.rs`**
   - Added `sfu_relay_capacity: u32` field to `Config`
   - Loaded from `SFU_RELAY_CAPACITY` env var, default `500`
   - Parse failures fall back to `500`

2. **`backend/src/sfu.rs`**
   - Added `relay_capacity: u32` field to `SfuState`
   - `SfuState::new()` takes `relay_capacity: u32`
   - `setup_peer_events()` receives `relay_capacity` parameter
   - Hardcoded `500` replaced with the parameter

3. **`backend/src/main.rs`**
   - `SfuState::new(config.sfu_relay_capacity)` at initialization

### Track deduplication fix (DT-11)

**Bug**: The SFU deduplication check at track reception filtered on `(user_id, kind)` only, omitting `peer_id`. If a user reconnected (new `peer_id`) before the old peer was fully cleaned up, the incoming track was incorrectly rejected as a duplicate.

**Fix**: Include `peer_id` in the dedup key. The check now uses `(user_id, peer_id, kind)`:

```rust
// Before:
tracks.iter().any(|t| t.user_id == uid && t.kind == kind)

// After:
tracks.iter().any(|t| t.user_id == uid && t.peer_id == peer_id && t.kind == kind)
```

**Verification**: The peer disconnect cleanup (event loop exit handler) already correctly removes tracks for `(user_id, peer_id)` — unchanged.

## Consequences

- **Positive**: Operators can increase subscriber limits per track via `SFU_RELAY_CAPACITY=1000` without rebuilding
- **Positive**: Default remains `500` — no behavioural change for existing deployments
- **Positive**: Reconnecting users with a new peer_id can publish tracks without waiting for stale peer cleanup
- **Negative**: Invalid values (non-numeric) silently fall back to `500`; operator should validate their config

## Usage

```bash
# Default (500 subscribers per track)
docker run -e ... nook:latest

# Custom limit
docker run -e SFU_RELAY_CAPACITY=1000 ... nook:latest
```
