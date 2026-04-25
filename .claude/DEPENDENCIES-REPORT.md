# 📦 Rapport Dépendances — Nook 2026-04-25

## Score : 74/100 (+4 depuis 2026-04-21)

---

## 🔴 CRITIQUE (0)

*Aucun problème critique identifié.*

---

## 🟡 HAUTE (1 → 0 après PR #32)

### ✅ H6 (CORRIGÉ dans PR #32) — Dépendances Rust inutilisées
- **Avant** : `tower-service`, `serde_urlencoded`, `lazy_static`, `home` — aucun import trouvé
- **Correction** : Supprimés de `backend/Cargo.toml`
- **Fichier** : `backend/Cargo.toml`

### Dépendances conservées (utilisées)
- ✅ `urlencoding = "2.1"` — utilisé dans `gifs_updater.rs`
- ✅ `sysinfo = "0.32"` — utilisé dans `admin.rs`

---

## 🟢 MOYENNE (2 → 0 après PR #32)

### ✅ M9 (CORRIGÉ dans PR #32) — `chacha20poly1305` 0.10.1 → 0.10.8
- **Avant** : version 0.10.1 avec patch de sécurité disponible
- **Correction** : Mise à jour vers 0.10.8 dans `backend/Cargo.toml`
- **Fichier** : `backend/Cargo.toml`

### ✅ M10 (CORRIGÉ dans PR #32) — `uuid` frontend 13 → 14
- **Avant** : version ^13.0.0, `@types/uuid` inutile
- **Correction** : Mise à jour vers ^14.0.0, suppression de `@types/uuid`
- **Fichier** : `frontend/package.json`

---

## ✅ BIEN IMPLÉMENTÉ (positifs)

### Licences
- ✅ **Aucun problème de licences** (MIT/Apache-2.0/BSD/ISC)

### Dépendances Frontend
- ✅ **DOMPurify** ajouté (sanitisation SVG dans Icon.svelte)
- ✅ **simple-peer** supprimé (PR #28) — utilise `RTCPeerConnection` natif

### Dépendances Backend
- ✅ **4 dépendances inutilisées supprimées** (PR #32) :
  - ❌ `tower-service` (retiré)
  - ❌ `serde_urlencoded` (retiré)
  - ❌ `lazy_static` (retiré)
  - ❌ `home` (retiré)
- ✅ **Dépendances conservées** :
  - `urlencoding` (utilisé dans `gifs_updater.rs`)
  - `sysinfo` (utilisé dans `admin.rs`)
- ✅ **chacha20poly1305** mis à jour vers 0.10.8 (M9)

---

## 📋 RÉSUMÉ DES CORRECTIONS (PR #32)

| Action | Dépendance | État |
|--------|-------------|--------|
| Supprimer | `tower-service 0.3` | ✅ Retiré (PR #32) |
| Supprimer | `serde_urlencoded 0.7` | ✅ Retiré (PR #32) |
| Supprimer | `lazy_static 1.4` | ✅ Retiré (PR #32) |
| Supprimer | `home 0.5` | ✅ Retiré (PR #32) |
| Mettre à jour | `chacha20poly1305 0.10.1 → 0.10.8` | ✅ Fait (PR #32, M9) |
| Conserver | `urlencoding 2.1` | ✅ Utilisé dans `gifs_updater.rs` |
| Conserver | `sysinfo 0.32` | ✅ Utilisé dans `admin.rs` |
| Ajouter | `dompurify` (frontend) | ✅ Pour SVG sanitization (PR #31) |

---

## 🧪 TESTS RECOMMANDÉS

```bash
# 1. Vérifier que les dépendances supprimées ne sont plus dans l'arbre
cd backend && cargo tree | grep -E "tower-service|serde_urlencoded|lazy_static|home"
# → Aucun résultat attendu

# 2. Vérifier que urlencoding et sysinfo sont toujours présents
cargo tree | grep -E "urlencoding|sysinfo"
# → Doit apparaître

# 3. Vérifier que le backend compile
cargo check

# 4. Vérifier les dépendances frontend
cd ../frontend && npm run build
```

---

## 📊 ÉVOLUTION DES SCORES

| Date | Dépendances | Progression |
|------|-------------|------------|
| 2026-04-09 | 70/100 | Base |
| 2026-04-21 (avant PR #31) | 70/100 | = |
| 2026-04-21 (après PR #31) | 72/100 | +2 (4 deps supprimées) |
| 2026-04-25 (après PR #32) | **74/100** | **+4** (M9 + H6) |

**Progression** : +4 points grâce à la mise à jour de `chacha20poly1305` et suppression définitive des dépendances inutilisées.

---

## 🔗 RÉFÉRENCES

- [RustSec Advisory Database](https://rustsec.org/)
- [Cargo Audit](https://github.com/rustsec/cargo-audit/)
- [Snyk Vulnerability Database](https://security.snyk.io/)
