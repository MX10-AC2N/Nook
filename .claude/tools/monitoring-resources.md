# Monitoring & Observabilité - Ressources Rust

Ce fichier recense les outils de monitoring et observabilité écrits en Rust pour surveiller vos applications et serveurs.

## 📊 Dashboards Web

### Interfaces Web Complètes

**[Beszel](https://github.com/henrygd/beszel)** - Monitoring de serveur léger

| Aspect | Détails |
|--------|---------|
| **UI** | Interface web moderne |
| **Agents** | Agents Go léger |
| **Metrics** | CPU, RAM, Disk, Network |
| **Docker** | Surveillance conteneurs |
| **Alerting** | Alertes configurables |
| **Lightweight** | Alternative à Prometheus/Grafana |
| **Multi-server** | Supervision multi-serveurs |
| **Licence** | MIT |

**Stack technique :**
- Backend: Go
- Frontend: React/TypeScript
- Agent: Go lightweight

**Endpoints :**
```
GET /api/systems          # Liste des systèmes
GET /api/systems/:id/stats # Stats d'un système
GET /api/alerts           # Alertes actives
```

---

**[rustmon](https://github.com/imdadareeph/rustmon)** - Observabilité système en Rust pur

| Aspect | Détails |
|--------|---------|
| **Stack** | Rust 62.7% + React/TypeScript |
| **Backend** | Axum + Tokio |
| **Features** | CPU, RAM, Disk, Network, Processes |
| **Streaming** | WebSocket temps réel (1s intervalle) |
| **Docker** | Conteneurs, images, volumes, réseaux |
| **Dashboards** | Widgets modulaires configurables |
| **API** | REST endpoints + WebSocket `/ws` |
| **Licence** | MIT |

**Endpoints WebSocket :**
```
ws://127.0.0.1:3000/ws

Topics disponibles:
- metrics.cpu / metrics.cpu.cores / metrics.cpu.history
- metrics.memory / metrics.swap
- metrics.disk / metrics.network
- metrics.processes
- docker.containers / docker.images / docker.volumes
```

**Endpoints REST :**
```
GET /api/settings          # Paramètres
GET /api/state             # État système
GET /api/metrics           # Snapshot métriques
GET /api/docker/containers # Conteneurs Docker
GET /api/docker/images     # Images Docker
GET /api/docker/volumes    # Volumes Docker
GET /api/dashboards        # Tableaux de bord
GET /api/dashboards/:id    # Dashboard spécifique
```

**Intégration suggérée :**
- Exposer `/api/metrics` dans Axum
- Dashboard admin avec état serveur
- Intégration notifications push

---

**[batmon](https://github.com/)** - Monitoring batteries (IoT/Embebbed)

| Aspect | Détails |
|--------|---------|
| **Focus** | Batteries et énergie |
| **IoT** | Raspberry Pi, ESP32 |
| **Metrics** | Voltage, courant, capacité |

---

## 🖥️ TUI Terminal

### Moniteurs TUI Interactifs

**[monitor-rs](https://github.com/Tinega-Devops/monitor-rs)** - Moniteur TUI terminal en Rust

| Aspect | Détails |
|--------|---------|
| **Interface** | TUI interactive via ratatui |
| **Metrics** | CPU, RAM, Disk I/O, Network throughput |
| **Alertes** | Système configurable avec seuils |
| **Logs** | `alerts.log` pour historique |
| **Stack** | 95.5% Rust pur |
| **Cross-platform** | Linux, macOS (Windows à venir) |
| **Licence** | MIT |

**Architecture modulaire :**
```
monitor-rs/
├── src/
│   ├── metrics/           # Collecteurs système
│   │   ├── cpu.rs         # Métriques CPU
│   │   ├── memory.rs      # Métriques RAM
│   │   ├── disk.rs        # Métriques disque
│   │   ├── network.rs     # Métriques réseau
│   │   └── snapshot.rs    # Capture instantanée
│   ├── alerting/          # Système d'alertes
│   │   ├── handler.rs     # Gestionnaire
│   │   └── rules.rs       # Règles configurables
│   └── ui/                # Interface TUI
│       ├── dashboard.rs    # Dashboard principal
│       ├── cpu_widget.rs  # Widget CPU
│       ├── memory_widget.rs # Widget RAM
│       ├── disk_widget.rs # Widget disque
│       ├── net_widget.rs  # Widget réseau
│       └── theme.rs       # Thèmes
├── Dockerfile
└── docker-compose.yml
```

**Système d'alertes :**
```rust
// alerting/rules.rs
fn default_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "High CPU".into(),
            metric: Metric::CpuUsage,
            threshold: 85.0,
            comparison: Comparison::GreaterThan,
        },
        AlertRule {
            name: "Low Memory".into(),
            metric: Metric::MemoryAvailable,
            threshold: 10.0,
            comparison: Comparison::LessThan,
        },
    ]
}
```

**Logs d'alertes :**
```
[ALERT] High CPU Usage triggered at 2025-05-25 16:30:22. Threshold: 85.0
[ALERT] Low Memory triggered at 2025-05-25 17:45:10. Threshold: 10.0%
```

**Installation :**
```bash
git clone https://github.com/Tinega-Devops/monitor-rs.git
cd monitor-rs
cargo build --release
./target/release/monitor-rs
# Contrôles: q pour quitter
```

---

**[btop](https://github.com/aristocratos/btop)** - Monitor system (C++/Python)

| Aspect | Détails |
|--------|---------|
| **Interface** | TUI moderne via ratatui |
| **Metrics** | CPU, RAM, Disk, Network, Processes |
| **Themes** | Thèmes personnalisables |
| **Languages** | Support multi-langues |

**Note :** Écrit en C++ mais très populaire dans l'écosystème Rust pour le monitoring.

---

**[bpytop](https://github.com/aristocratos/bpytop)** - Python port de btop

| Aspect | Détails |
|--------|---------|
| **Language** | Python |
| **Compat** | Linux, macOS |

---

## 📈 Instrumentation Métriques

### Bibliothèques Rust

**[metrics](https://github.com/metrics-rs/metrics)** - Framework d'instrumentation

| Aspect | Détails |
|--------|---------|
| **Types** | Counter, Gauge, Histogram |
| **Output** | Prometheus, stdout, etc. |
| **Performance** | Overhead minimal |

**Utilisation :**
```rust
use metrics::{counter, gauge, histogram};

counter!("requests_total", "method" => "GET");
gauge!("active_connections", 42.0);
histogram!("request_duration", 0.123, "route" => "/api");
```

---

**[tracing](https://github.com/tokio-rs/tracing)** - Tracing distribué

| Aspect | Détails |
|--------|---------|
| **Spans** | Traces hiérarchiques |
| **Events** | Logging structuré |
| **Subscriber** | Multiple outputs |
| **async** | Support async natif |

**Utilisation :**
```rust
use tracing::{info, span, Level};

let span = span!(Level::INFO, "request_handler");
let _guard = span.enter();

info!("Processing request");
```

---

**[prometheus](https://github.com/prometheus/client_rust)** - Client Prometheus

| Aspect | Détails |
|--------|---------|
| **Registry** | Metric registry |
| **Encoder** | Text, Protobuf |
| **Handler** | `/metrics` endpoint |

**Exemple Axum :**
```rust
use prometheus::{Encoder, TextEncoder, Registry};

let registry = Registry::new();
let metrics_handler = move || {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Response::new(buffer.into())
};
```

---

**[metrics-util](https://github.com/metrics-rs/metrics-util)** - Utilities metrics

| Aspect | Détails |
|--------|---------|
| **Quantiles** | HdrHistogram |
| **Sampling** | Reservoir sampling |
| **Layering** | Tower middleware |

---

**[once_cell](https://github.com/matklad/once_cell)** - State lazy initialization

| Aspect | Détails |
|--------|---------|
| **sync** | Thread-safe lazy |
| **unsync** | Single-thread lazy |

---

## 🔍 Logging & Tracing

### Bibliothèques de Logging

**[tracing-subscriber](https://github.com/tokio-rs/tracing)** - Subscriber pour tracing

| Aspect | Détails |
|--------|---------|
| **fmt** | Formatage lisible |
| **JSON** | Format JSON structuré |
| **env** | Configuration via RUST_LOG |
| **Filter** | Filtrage par niveau |

**Configuration :**
```rust
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_target(true)
    .init();
```

---

**[log](https://github.com/rust-log/log)** - Logging facade

| Aspect | Détails |
|--------|---------|
| **Facade** | Interface de logging |
| **Macros** | info!, warn!, error! |
| **Env** | Variables d'environnement |

---

**[sentry](https://github.com/getsentry/sentry-rust)** - Error tracking

| Aspect | Détails |
|--------|---------|
| **Errors** | Capture d'erreurs |
| **Breadcrumbs** | Contexte d'exécution |
| **Release** | Tracking par version |

---

**[tracing-opentelemetry](https://github.com/tokio-rs/tracing)** - OpenTelemetry integration

| Aspect | Détails |
|--------|---------|
| **OTLP** | Export OTLP protocol |
| **Jaeger** | Compatible Jaeger |
| **Zipkin** | Support Zipkin |

---

## 🐳 Docker & Containers

### Monitoring Docker

**[docker-api-rust](https://github.com/7mind/izumi-rage)** - API Docker Rust

| Aspect | Détails |
|--------|---------|
| **API** | Docker Engine API |
| **Async** | Support Tokio |
| **Containers** | Stats, logs, management |

---

**[bollard](https://github.com/fussybeaver/bollard)** - Docker API async

| Aspect | Détails |
|--------|---------|
| **Async** | Entièrement async |
| **Protocol** | Docker HTTP API |
| **Tls** | Support TLS |

---

**[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - System information

| Aspect | Détails |
|--------|---------|
| **CPU** | Per-core et global |
| **Memory** | RAM et swap |
| **Processes** | Liste processus |
| **Disks** | I/O disque |
| **Networks** | Traffic réseau |
| **Containers** | Docker containers |

**Utilisation :**
```rust
use sysinfo::{System, Containers};

let mut sys = System::new_all();
sys.refresh_all();

// CPU
println!("CPU usage: {}%", sys.global_cpu_usage());

// Memory
println!("RAM: {} / {}",
    sys.used_memory(),
    sys.total_memory()
);

// Containers
if let Some(containers) = sys.containers() {
    for (pid, container) in containers {
        println!("{:?}", container);
    }
}
```

---

## 🔧 Health Checks

### Endpoints de Santé

**[axum-health](https://github.com/antonpetrov145/axum_health)** - Health check pour Axum

| Aspect | Détails |
|--------|---------|
| **Endpoint** | `/health` |
| **Checks** | Personnalisables |
| **Liveness** | Liveness probe |
| **Readiness** | Readiness probe |

**Utilisation Axum :**
```rust
use axum_health::{health, HealthCheck};

async fn my_check() -> Result<(), ()> {
    // Vérifier connexion DB
    Ok(())
}

let app = Router::new()
    .route("/health", health().add_check("my_service", my_check));
```

---

**[tower-http](https://github.com/tower-rs/tower-http)** - Middleware HTTP

| Aspect | Détails |
|--------|---------|
| **CORS** | Configuration CORS |
| **Compression** | gzip, brotli |
| **Tracing** | Distributed tracing |
| **Follow-redirects** | Redirect handling |

---

## 📊 Visualisation

### Dashboards & UI

**[ratatui](https://github.com/ratatui/ratatui)** - TUI library

| Aspect | Détails |
|--------|---------|
| **Backend** | crossterm, termwiz, ncurses |
| **Widgets** | Charts, tables, sparklines |
| **Styling** | Styles, colors, text |
| **Async** | Support async events |

**Widgets disponibles :**
- `Gauge` - Barres de progression
- `Sparkline` - Graphiques miniatures
- `Chart` - Graphiques avec Bar, Line, Points
- `Table` - Tableaux formatés
- `List` - Listes scrollables

---

**[plotly](https://github.com/sigpwned/plotly)** - Graphiques pour Rust

| Aspect | Détails |
|--------|---------|
| **Charts** | Line, bar, scatter, pie |
| **Output** | HTML, PNG, SVG |
| **Web** | Intégration web |

---

**[egui](https://github.com/emilk/egui)** - Immediate mode GUI

| Aspect | Détails |
|--------|---------|
| **Web** | WebAssembly |
| **Desktop** | Native |
| **Immediate** | Immediate mode |

---

## 🆚 Comparatif des Outils

| Outil | Type | Interface | Alertes | Docker | Best for |
|-------|------|-----------|---------|--------|----------|
| **Beszel** | Dashboard | Web | ✅ | ✅ | Multi-serveurs |
| **rustmon** | Dashboard | Web (React) | Via API | ✅ | Dashboard admin |
| **monitor-rs** | TUI | Terminal | ✅ | ✅ | Debug SSH |
| **btop** | TUI | Terminal | ❌ | ❌ | Monitoring rapide |
| **metrics + tracing** | Library | N/A | Via code | Via code | Custom metrics |

---

## 🚀 Intégration Recommandée

### Stack Minimal (Nook)

```
┌─────────────────────────────────────────┐
│  Beszel        → Dashboard global        │
│  sysinfo       → Health /metrics        │
│  tracing       → Structured logging      │
│  monitor-rs    → Debug terminal         │
└─────────────────────────────────────────┘
```

### Stack Complète

```
┌─────────────────────────────────────────┐
│  Prometheus     → Collection metrics    │
│  Grafana        → Visualisation         │
│  Loki           → Logs                  │
│  Beszel         → Dashboard             │
│  monitor-rs     → Debug terminal        │
│  tracing        → Application traces     │
└─────────────────────────────────────────┘
```

---

## 📝 Checklist Monitoring Nook

### Minimal (Recommandé)

- [ ] Health check endpoint `/health`
- [ ] Logging structuré (tracing)
- [ ] Intégration d'un monitoring natif en Rust

### Intermediate

- [ ] Exposer `/api/metrics` (sysinfo)
- [ ] Intégration rustmon
- [ ] Alertes push (notifications existantes)

---

## 📚 Ressources Complémentaires

### Awesome Lists

**[awesome-rust](https://github.com/rust-unofficial/awesome-rust)** - Section Observability

**[awesome-sysadmin](https://github.com/awesome-sysadmin/awesome-sysadmin)** - Monitoring section

### Articles

**[Rust Observability Guide](https://www.shuttle.rs/blog/2024/01/09/rust-observability)** - Guide complet

**[Metrics in Rust](https://www.bitquery.io/blog/rust-metrics)** - Tutorial metrics

---

## Notes

- **Last updated**: 2026-03-29
- **Focus**: Outils Rust pour monitoring
- **Écosystème**: En croissance rapide
- **Mise à jour**: Trimestrielle recommandée
```

---