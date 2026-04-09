# 🏗️ Backend Build Report — amd64 — Nook

> **unknown** | commit 1cb544d | [run](https://github.com/MX10-AC2N/Nook/actions/runs/24203970594)

## Récapitulatif statuts

| Check | Résultat |
|-------|----------|
| **cargo build** | ❌ FAIL |
| **cargo check** | ❌ exit=101 |
| **cargo clippy** | ❌ exit=101 |

| Métrique | Valeur |
|----------|--------|
| **Bin Size** | N/A |
| **Compile Time** | N/A |
| **Warnings (check)** | N/A |
| **Errors (check)** | N/A |
| **New Warnings** | N/A |
| **Deprecated refs** | 0 |
| **Dead code** | 0 |
| **Unused vars** | 0 |
| **Unreachable** | 0 |

---

## ⚠️ Warnings cargo check (top)

```
(aucun warning)
```

## ❌ Erreurs cargo check

```
error[E0425]: cannot find function `validate_session` in module `crate::auth`
error[E0609]: no field `pool` on type `Arc<SharedState>`
error[E0609]: no field `config` on type `Arc<SharedState>`
error[E0034]: multiple applicable items in scope
```

## ❌ Erreurs cargo build

```
error[E0425]: cannot find function `validate_session` in module `crate::auth`
error[E0609]: no field `pool` on type `Arc<SharedState>`
error[E0609]: no field `config` on type `Arc<SharedState>`
error[E0034]: multiple applicable items in scope
```

## 🔧 Clippy warnings

```
(non disponible ou aucun)
```

### Clippy lint types (top 10)

```
(non disponible)
```

---

## 🧪 Tests

```
(non disponible)

(non disponible)
```

---

## 📦 Compilation (30 derniers crates)

```



```

---

*Rapport généré par `.github/scripts/generate-backend-report.sh`*
