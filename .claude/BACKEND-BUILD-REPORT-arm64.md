# 🏗️ Backend Build Report — arm64 — Nook

> Généré automatiquement par `Backend.yml` · target `aarch64-unknown-linux-gnu`
> **2026-03-13 18:00 UTC**

---

## Statut global : ❌ FAIL

| Champ | Valeur |
|-------|--------|
| **Architecture** | `arm64` (`aarch64-unknown-linux-gnu`) |
| **Branche** | `develop` |
| **Commit** | [`00acb2b`](https://github.com/MX10-AC2N/Nook/commit/00acb2bf2bde8e14bbc400ccc01c8f755136ac71) |
| **Rust** | `rustc 1.94.0 (4a4ef493e 2026-03-02)` |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23063943557) |

---

## Étapes

| Étape | Statut | Détail |
|-------|--------|--------|
| **cargo check** | ❌ | exit 101 |
| **cargo clippy** | ❌ | exit 101 (-D warnings) |
| **cargo build --release** | ❌ | binaire N/A stripped |

---

## Erreurs cargo check

```
(aucune)
```

---

## Warnings clippy (-D warnings = fail si présents)

```
(aucun)
```

### Contexte complet (fichier + ligne)

```
(aucun)
```

---

## Erreurs cargo build --release

```
(aucune)
```

---

## Résumé compilation

```
error[E0432]: unresolved import `governor::state::KeyedRateLimiter`
```

---

*Rapport généré par `.github/workflows/Backend.yml` · job `aarch64-unknown-linux-gnu`*
