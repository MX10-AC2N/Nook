# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-13** (session 36)

---

## 🔴 BUGS ACTIFS

*Aucun bug actif bloquant.*

---

## 📋 Règles Svelte 5 (éviter régressions)

```typescript
// ✅ export $state → objet encapsulant, mutation via propriété
export const store = $state<State>({...});
store.prop = newValue;  // ✅
// ❌ jamais : export let x = $state(); x = newVal;
// ❌ jamais : writable() Svelte 4
// ❌ $derived/$effect hors composant .svelte
```

---

## ✅ BUGS RÉSOLUS — Index compact

> Détails complets dans `SESSIONS.md`. Format : `[Session] Titre — Fix en une ligne`

| ID | Session | Titre | Fix |
|----|---------|-------|-----|
| R37 | 37 | `waitForSodium()` bloquait `loading=false` → `#username` jamais visible en CI | Sodium lancé en fire-and-forget, `loading=false` après `authStore.init()` uniquement |
| R36a | 36 | Page blanche Zimaboard — base_inject_middleware inutile | Supprimé de main.rs + app.html nettoyé |
| R36b | 36 | Rate limit 429 en CI E2E — NotKeyed global épuisé par les tests | KeyedRateLimiter par IP, quota 30/min |
| R33 | 33 | `clearSession` ne vidait pas localStorage → `isAuthenticated=true` | `page.evaluate(() => localStorage.clear())` |
| R25 | 26 | Polls E2E race condition `waitForResponse` après `goto()` | `Promise.all([waitForResponse, goto()])` |
| R24 | 25 | Layout bloque sur `!cryptoInitialized` → `#username` jamais visible | Crypto failure = mode dégradé non-bloquant |
| R23 | 23 | `fill('#username')` avant layout onMount | `waitFor('#username', visible, 20s)` |
| R22 | 22 | `clearSession` goto('/') → authStore.init avec cookie | `page.request.post(logout)` avant tout goto |
| R21 | 21 | `fullyParallel:true` partage browser context | `fullyParallel: false` |
| R20 | 20 | Race condition matrix amd64/arm64 | Deux fichiers rapport séparés |
| R19 | 19 | git push TEST_REPORT non-fast-forward | Fetch avant push dans workflow |
| R18 | 18 | Admin UI : #username disabled localStorage | `loginAsAdmin` API-first |
| R17 | 17 | Chess page strict mode violation h1 | Un seul h1 par page |
| R16 | 16 | Logout button introuvable E2E | Sélecteur data-testid ajouté |
| R15 | 15 | e2e_ci absent conversation_participants | Ajout dans E2E_SETUP init |
| R14 | 13 | Prune supprime default_global | Exclure conversations système |
| R13 | 12 | Cookie SameSite=Lax bloque WAN | Détecter X-Forwarded-Proto → None;Secure |
| R12 | 11 | CORS bloque LAN + WAN simultanément | Lister origines explicites |
| R11 | 10 | crypto.randomUUID HTTP LAN | Fallback UUID v4 manuel |
| R05 | 5 | SQLite SQLITE_CANTOPEN code 14 | `create_if_missing(true)` |
| R04 | 4 | Linker crash Docker (.cargo/config.toml) | Ne pas COPY .cargo/ dans Docker |
| R03 | 3 | proc-macro async-trait crash | Retirer tower_governor |
| R02 | 2 | rand_core diamond dep | `rand_core = "0.6"` explicite |
| R01 | 2 | axum 0.8 breaking changes | Routes {param}, Message::Text .into() |
| R_B1 | 26 | `state_invalid_export` conversationStore | Déjà corrigé (objet $state encapsulé) |
| R_B3 | 26 | `connectionError.set()` cassé | Déjà corrigé : `setConnectionError()` |

---

## 🛡️ Sécurité — État des vulnérabilités (audit S35)

| ID | Vulnérabilité | Statut | Session fix |
|----|---------------|--------|-------------|
| SEC-01 | XSS `{@html}` chat | ✅ Résolu | S35 (DOMPurify) |
| SEC-02 | Rate limit global (non IP) | ✅ Résolu | **S36** (KeyedRateLimiter par IP) |
| SEC-04 | Magic bytes uploads non validés | ✅ Résolu | **S36** (validate_magic_bytes) |
| SEC-05 | Pas de limite taille messages WS | ✅ Résolu | **S36** (64KB limit) |
| SEC-03 | Token session UUID (entropy ok, 256 bits optionnel) | 🟡 Faible risque | S37 optionnel |
| SEC-06 | emergency.rs non connecté | 🟡 Informationnel | Avant activation |

---

## 🌐 Architecture LAN ↔ WAN (référence rapide)

```
LAN : HTTP 192.168.x.x:6300 → SameSite=Lax
WAN : HTTPS via Nginx → X-Forwarded-Proto: https → SameSite=None; Secure
CORS : ALLOWED_ORIGINS env, jamais Any avec credentials
Rate limit : 30 req/min par IP (KeyedRateLimiter, governor)
```

---
*Mis à jour session 37 — ajout R37*
