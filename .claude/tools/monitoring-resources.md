# 📊 Monitoring Resources — Nook

> Outils de monitoring applicables à Nook sur Zimaboard ARM64.
> Mis à jour : session 44

---

## ✅ Recommandé — intégration dans Nook

### sysinfo — Métriques système depuis Rust (LOT 3)
**Crate :** https://crates.io/crates/sysinfo  
**Décision :** Ajouter dans Cargo.toml + exposer `GET /api/admin/metrics`

```toml
sysinfo = "0.32"
```

```rust
// backend/src/admin.rs — nouvelle route
use sysinfo::System;

pub async fn get_system_metrics(
    State(state): State<Arc<SharedState>>,
    Extension(CurrentUser(user)): Extension<CurrentUser>,
) -> impl IntoResponse {
    if user.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "Accès refusé" }))).into_response();
    }
    let mut sys = System::new_all();
    sys.refresh_all();
    Json(json!({
        "cpu_usage":      sys.global_cpu_usage(),
        "memory_used":    sys.used_memory(),
        "memory_total":   sys.total_memory(),
        "uptime_secs":    System::uptime(),
        "load_avg":       System::load_average().one,
    })).into_response()
}
```

```typescript
// frontend/src/routes/admin/+page.svelte — section métriques
const metrics = await fetch('/api/admin/metrics', { credentials: 'include' }).then(r => r.json());
```

---

## 🟡 Outil externe — Beszel (monitoring Zimaboard)

**Repo :** https://github.com/henrygd/beszel  
**Décision :** Déployer séparément sur le Zimaboard, pas intégré dans Nook  
**Usage :** Dashboard web pour surveiller CPU/RAM/disque/Docker du Zimaboard

```yaml
# docker-compose sur le Zimaboard (séparé de Nook)
beszel:
  image: henrygd/beszel
  ports: ["8090:8090"]
  volumes: ["./beszel-data:/app/data"]
```

---

## ❌ Non retenus pour Nook

| Outil | Raison |
|---|---|
| Prometheus + Grafana | Trop lourd pour un Zimaboard 8Go, overkill pour usage familial |
| monitor-rs (TUI) | TUI terminal — pas d'interface web, pas intégrable |
| btop/bpytop | C++/Python — déjà disponible sur le Zimaboard si besoin |
| OpenTelemetry | Infrastructure enterprise — hors scope |
| Sentry | Service cloud payant — hors scope pour un projet self-hosted |

---

## 📝 Ce qui existe déjà dans Nook

- **`tracing`** — logging structuré, actif en production (`RUST_LOG=info`)
- **`GET /api/health`** — health check basique, retourne "OK"
- **`CompressionLayer`** — compression brotli/gzip sur les réponses

Le seul manque actionnable : **métriques CPU/RAM** via `sysinfo` dans la page admin.
