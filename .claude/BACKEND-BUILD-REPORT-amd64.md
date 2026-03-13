# 🏗️ Backend Build Report — amd64 — Nook

> Généré automatiquement par `Backend.yml` · target `x86_64-unknown-linux-gnu`
> **2026-03-13 18:09 UTC**

---

## Statut global : ❌ FAIL

| Champ | Valeur |
|-------|--------|
| **Architecture** | `amd64` (`x86_64-unknown-linux-gnu`) |
| **Branche** | `develop` |
| **Commit** | [`ad45a7c`](https://github.com/MX10-AC2N/Nook/commit/ad45a7c5a8caff101dacb1cea39c33dc942ffeba) |
| **Rust** | `rustc 1.94.0 (4a4ef493e 2026-03-02)` |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23064277096) |

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

*Rapport généré par `.github/workflows/Backend.yml` · job `x86_64-unknown-linux-gnu`*
