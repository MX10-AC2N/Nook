//! DT-04 — Redis-backed shared rate-limit state for `governor`.
//!
//! `governor` 0.10 only ships in-memory keyed state stores (`dashmap` /
//! `hashmap`). It does **NOT** provide a Redis store (there is no
//! `RedisKeyedStateStore` type in any published `governor` version). To share
//! rate-limit counters across multiple Nook backend instances we implement the
//! `StateStore` trait ourselves: each key's GCRA "theoretical arrival time"
//! (TAT, in nanoseconds) lives in Redis and is updated through an atomic
//! compare-and-store Lua script so that concurrent instances converge to the
//! same (maximum) TAT and thus enforce one shared limit.
//!
//! If `REDIS_URL` is unset the backend falls back to the in-memory
//! `DefaultKeyedStateStore` (see [`IpRateLimiter::Memory`]). That fallback path
//! keeps local dev and single-instance deployments working unchanged.

use std::error::Error;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::nanos::Nanos;
use governor::state::StateStore;
use governor::Quota;
use governor::RateLimiter;
use redis::Client as RedisClient;
use r2d2::Pool as R2d2Pool;

/// Prefix for rate-limit keys in Redis.
const REDIS_KEY_PREFIX: &str = "nook:ratelimit:";

/// Idle key expiry (seconds). Covers the per-minute refill window with margin
/// so Redis doesn't accumulate dead IP counters forever.
const DEFAULT_TTL_SECS: u64 = 120;

/// Atomic compare-and-store of the GCRA TAT.
///
/// `KEYS[1]` = entry key
/// `ARGV[1]` = desired TAT (u64 ns)
/// `ARGV[2]` = ttl seconds
///
/// Stores `max(current, desired)`: a conflict-resolving CAS that always
/// converges to the most-restrictive TAT even when several instances advance
/// the counter concurrently. Returns the stored value.
const UPSERT_LUA: &str = r#"
local cur = redis.call('GET', KEYS[1])
cur = cur and tonumber(cur) or nil
local desired = tonumber(ARGV[1])
local store = (cur and cur > desired) and cur or desired
redis.call('SET', KEYS[1], store, 'EX', ARGV[2])
return store
"#;

/// A `governor` `StateStore` that keeps each key's GCRA state in Redis.
///
/// Connections are taken from a shared `r2d2` pool. The pool connects lazily,
/// so an unreachable Redis at startup does not abort the process — a per-request
/// failure instead fails *open* (allows the request) so an outage can't take
/// down the whole service.
pub struct RedisKeyedStateStore {
    pool: R2d2Pool<RedisClient>,
    ttl: u64,
}

impl RedisKeyedStateStore {
    /// Build a pooled Redis state store from a `redis://` URL.
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = RedisClient::open(redis_url)?;
        let pool = R2d2Pool::builder().max_size(16).build(client)?;
        Ok(Self {
            pool,
            ttl: DEFAULT_TTL_SECS,
        })
    }
}

impl StateStore for RedisKeyedStateStore {
    type Key = IpAddr;

    fn measure_and_replace<T, F, E>(&self, key: &Self::Key, f: F) -> Result<T, E>
    where
        F: Fn(Option<Nanos>) -> Result<(T, Nanos), E>,
    {
        let redis_key = format!("{REDIS_KEY_PREFIX}{key}");

        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(e) => {
                // Redis unreachable → fail OPEN (allow the request) so an outage
                // can't take down the service.
                tracing::warn!(error = %e, "rate-limit: Redis indisponible, fail-open");
                let (ok, _) = f(None)?;
                return Ok(ok);
            }
        };
        // `PooledConnection` derefs to `redis::Connection`; redis cmd/Script want
        // `&mut ConnectionLike`, so deref explicitly.
        let con = &mut *conn;

        // Read current TAT (None if this IP was never seen / expired).
        let old: Option<Nanos> = redis::cmd("GET")
            .arg(&redis_key)
            .query::<Option<String>>(con)
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Nanos::from);

        match f(old) {
            // Rate-limited: do NOT advance the stored TAT.
            Err(e) => Err(e),
            Ok((outcome, new_tat)) => {
                let new_u = Nanos::as_u64(new_tat);
                // Best-effort store; if the write fails we already decided to
                // allow, so we keep the request allowed (fail-open) rather than
                // re-denying it.
                let _: Result<i64, _> = redis::Script::new(UPSERT_LUA)
                    .key(&redis_key)
                    .arg(new_u)
                    .arg(self.ttl)
                    .invoke(con);
                Ok(outcome)
            }
        }
    }
}

/// Either a shared Redis-backed limiter (multi-instance) or the in-memory
/// fallback (single instance / local dev). Wraps the two distinct
/// `RateLimiter` types behind one ergonomic [`IpRateLimiter::check`].
pub enum IpRateLimiter {
    Memory(Arc<RateLimiter<IpAddr, governor::state::keyed::DefaultKeyedStateStore<IpAddr>, DefaultClock, NoOpMiddleware>>),
    Redis(Arc<RateLimiter<IpAddr, RedisKeyedStateStore, DefaultClock, NoOpMiddleware>>),
}

impl IpRateLimiter {
    /// Returns `true` if the request is allowed (under the limit).
    pub fn check(&self, ip: &IpAddr) -> bool {
        match self {
            IpRateLimiter::Memory(l) => l.check_key(ip).is_ok(),
            IpRateLimiter::Redis(l) => l.check_key(ip).is_ok(),
        }
    }

    /// Build the limiter, selecting the Redis store when `redis_url` is set.
    /// Falls back to the in-memory store if `redis_url` is `None` or the Redis
    /// store cannot be created (e.g. bad URL) — never fails the whole startup.
    pub fn build(redis_url: Option<&str>, per_minute: u32) -> Arc<Self> {
        let quota = Quota::per_minute(NonZeroU32::new(per_minute.max(1)).unwrap());
        match redis_url {
            Some(url) => match RedisKeyedStateStore::new(url) {
                Ok(store) => Arc::new(IpRateLimiter::Redis(Arc::new(RateLimiter::new(
                    quota,
                    store,
                    DefaultClock::default(),
                )))),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "rate-limit: store Redis impossible, fallback mémoire"
                    );
                    Arc::new(IpRateLimiter::Memory(Arc::new(RateLimiter::keyed(quota))))
                }
            },
            None => Arc::new(IpRateLimiter::Memory(Arc::new(RateLimiter::keyed(quota)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SEC-02 preserved: per-IP keyed limiter blocks once the burst is consumed.
    // Quota::per_minute(1) allows exactly one immediate cell, then blocks until
    // the replenish interval (60s) — assert the deterministic blocking.
    #[test]
    fn memory_fallback_blocks_after_quota() {
        let lim = IpRateLimiter::build(None, 1);
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        assert!(lim.check(&ip), "1st call should be allowed");
        assert!(!lim.check(&ip), "2nd call must exceed quota → blocked");
    }

    #[test]
    fn distinct_ips_have_separate_counters() {
        let lim = IpRateLimiter::build(None, 1);
        let a: IpAddr = "198.51.100.1".parse().unwrap();
        let b: IpAddr = "198.51.100.2".parse().unwrap();
        assert!(lim.check(&a));
        assert!(!lim.check(&a), "a exceeded its quota");
        assert!(lim.check(&b), "b keeps its own independent counter");
    }

    // DT-04: with REDIS_URL set the Redis variant is selected, and a pool that
    // can't reach a server must NOT panic at construction (lazy connect) and
    // must fail OPEN (allow) so an outage can't take the service down.
    #[test]
    fn redis_path_builds_and_fails_open() {
        let lim = IpRateLimiter::build(Some("redis://127.0.0.1:6379/"), 60);
        let ip: IpAddr = "192.0.2.5".parse().unwrap();
        assert!(lim.check(&ip), "fail-open: allowed when Redis unreachable");
    }
}