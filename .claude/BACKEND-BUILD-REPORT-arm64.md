# 🏗️ Backend Build Report — arm64 — Nook

> Généré automatiquement par `Backend.yml` · target `aarch64-unknown-linux-gnu`
> **2026-02-28 18:57 UTC**

---

## Statut global : ❌ FAIL

| Champ | Valeur |
|-------|--------|
| **Architecture** | `arm64` (`aarch64-unknown-linux-gnu`) |
| **Branche** | `develop` |
| **Commit** | [`4c2971c`](https://github.com/MX10-AC2N/Nook/commit/4c2971c27ff56c00f106b23f75e8e450b1e1f31b) |
| **Rust** | `rustc 1.93.1 (01f6ddf75 2026-02-11)` |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/22526917804) |

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
error[E0599]: no method named `choose` found for struct `Vec<Move>` in the current scope
```

---

*Rapport généré par `.github/workflows/Backend.yml` · job `aarch64-unknown-linux-gnu`*
