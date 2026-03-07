# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-07** (session 26)

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
| R25 | 26 | Polls E2E race condition `waitForResponse` après `goto()` | `Promise.all([waitForResponse, goto()])` — listener enregistré AVANT navigation |
| R24 | 25 | Layout bloque sur `!cryptoInitialized` → `#username` jamais visible | Crypto failure = mode dégradé non-bloquant, guard template sur `loading` seul |
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
| R_B1 | 26 | `state_invalid_export` conversationStore | Déjà corrigé dans le code (objet $state encapsulé) |
| R_B3 | 26 | `connectionError.set()` cassé | Déjà corrigé : `setConnectionError()` dans chatStore |

---

## 🌐 Architecture LAN ↔ WAN (référence rapide)

```
LAN : HTTP 192.168.x.x:6300 → SameSite=Lax
WAN : HTTPS via Nginx → X-Forwarded-Proto: https → SameSite=None; Secure
CORS : ALLOWED_ORIGINS env, jamais Any avec credentials
```
