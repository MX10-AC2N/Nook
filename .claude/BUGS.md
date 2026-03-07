# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-07** (session 31)

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

| ID | Session | Titre | Fix |
|----|---------|-------|-----|
| DT-02 | 31 | Chess temps réel absent — adversaire voit coups au refresh | WS reconnect exponentiel + gestion chess_player_joined + chess_ai_move dans chessStore |
| DT-05 | — | WebRTC WAN instable (TURN absent) | Non résolu — prévu |
| R31a | 31 | `send_message` ne broadcastait pas via WS | Ajout broadcast `new_message` dans `db.rs::send_message()` |
| R31b | 31 | `sendMessage()` appelé avec mauvaise signature (3 params) | Signature corrigée : `(content, convId)` |
| R31c | 31 | Upload échoue silencieusement > 50 Mo | Vérification côté client + message d'erreur avec timeout |
| R25 | 26 | Polls E2E race condition `waitForResponse` après `goto()` | `Promise.all([waitForResponse, goto()])` |
| R24 | 25 | Layout bloque sur `!cryptoInitialized` | Crypto failure = mode dégradé non-bloquant |
| R23 | 23 | `fill('#username')` avant layout onMount | `waitFor('#username', visible, 20s)` |
| R22 | 22 | `clearSession` goto('/') → authStore.init avec cookie | `page.request.post(logout)` avant tout goto |
| R21 | 21 | `fullyParallel:true` partage browser context | `fullyParallel: false` |
