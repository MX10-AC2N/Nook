# 📦 Rapport Dépendances — Nook 2026-04-21

## Score : 72/100 (+2 depuis 2026-04-21)

---

## 🔴 CRITIQUE (0)

*Aucun problème critique identifié.*

---

## 🟡 HAUTE (1 → 0 après PR #31)

### ✅ H6 (CORRIGÉ dans PR #31) — Dépendances Rust inutilisées
- **Avant** : `tower-service`, `serde_urlencoded`, `lazy_static`, `home` — aucun import trouvé
- **Correction** : Supprimés de `backend/Cargo.toml`
- **Fichier** : `backend/Cargo.toml`

### Dépendances conservées (utilisées)
- ✅ `urlencoding = "2.1"` — utilisé dans `gifs_updater.rs`
- ✅ `sysinfo = "0.32"` — utilisé dans `admin.rs`

---

## 🟢 MOYENNE (2)

### M9 — `chacha20poly1305` 0.10.1 → 0.10.8
- **Problème** : Patch crypto disponible
- **Recommandation** : `cargo update -p chacha20poly1305`
- **Fichier** : `backend/Cargo.toml`

### M10 — `uuid` frontend 13 → 14 (major)
- **Problème** : Version majeure disponible
- **Recommandation** : Vérifier la compatibilité et migrer
- **Fichier** : `frontend/package.json`

---

## ✅ BIEN IMPLÉMENTÉ (positifs)

### Licences
- ✅ **Aucun problème de licences** (MIT/Apache-2.0/BSD/ISC)

### Dépendances Frontend (après PR #31)
- ✅ **DOMPurify** ajouté (sanitisation SVG dans Icon.svelte)
- ✅ **simple-peer** supprimé (PR #28) — utilise `RTCPeerConnection` natif

### Dépendances Backend (après PR #31)
- ✅ **4 dépendances inutilisées supprimées** :
  - ❌ `tower-service` (retiré)
  - ❌ `serde_urlencoded` (retiré)
  - ❌ `lazy_static` (retiré)
  - ❌ `home` (retiré)
- ✅ **Dépendances conservées** :
  - `urlencoding` (utilisé)
  - `sysinfo` (utilisé)

---

## 📋 RÉSUMÉ DES CORRECTIONS (PR #31)

| Action | Dépendance | État |
|--------|-------------|--------|
| Supprimer | `tower-service 0.3` | ✅ Retiré |
| Supprimer | `serde_urlencoded 0.7` | ✅ Retiré |
| Supprimer | `lazy_static 1.4` | ✅ Retiré |
| Supprimer | `home 0.5` | ✅ Retiré |
| Conserver | `urlencoding 2.1` | ✅ Utilisé dans `gifs_updater.rs` |
| Conserver | `sysinfo 0.32` | ✅ Utilisé dans `admin.rs` |
| Ajouter | `dompurify` (frontend) | ✅ Pour SVG sanitization |

---

## 🧪 TESTS RECOMMANDÉS

```bash
# 1. Vérifier que les dépendances supprimées ne sont plus dans l'arbre
cd backend && cargo tree | grep -E "tower-service|serde_urlencoded|lazy_static|home"
# → Aucun résultat attendu

# 2. Vérifier que urlencoding est toujours présent
cargo tree | grep "urlencoding"
# → Doit apparaître

# 3. Vérifier que le frontend build passe avec DOMPurify
cd frontend && npm run build

# 4. Mettre à jour chacha20poly1305
cd backend && cargo update -p chacha20poly1305
```

---

## 📊 ÉVOLUTION DES SCORES

| Date | Dépendances | Progression |
|------|-------------|------------|
| 2026-04-09 | 70/100 | Base |
| 2026-04-21 (avant PR #31) | 70/100 | = |
| 2026-04-21 (après PR #31) | **72/100** | **+2** (4 deps supprimées) |

**Progression** : +2 points grâce à la suppression de 4 dépendances inutilisées.

---

## 🔗 RÉFÉRENCES

- [RustSec Advisory Database](https://rustsec.org/)
- [Cargo Audit](https://github.com/rustsec/cargo-audit/)
- [Snyk Vulnerability Database](https://security.snyk.io/)
