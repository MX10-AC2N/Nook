# 🐛 BUGS.md — Nook

> Mis à jour : **2026-03-08** (session 35)

---

## 🔴 BUGS ACTIFS

### [SEC-01] XSS via `{@html}` dans le chat — **CORRIGÉ session 35**
- Fichier : `frontend/src/lib/sanitize.ts` (nouveau) + `chat/+page.svelte`
- Fix : `{@html sanitizeHtml(msg.content)}` avec DOMPurify

### [SEC-01b] Pas de CSP — **CORRIGÉ session 35**
- Fichier : `frontend/src/app.html`
- Fix : `<meta http-equiv="Content-Security-Policy" ...>`

### [SEC-02] Rate limit global (NotKeyed) — pas par IP
- Fichier : `backend/src/main.rs` ligne 135
- Risque : faible en contexte familial — prévu session 38

### [SEC-04] Pas de validation magic bytes uploads
- Fichier : `backend/src/upload.rs`
- Prévu : session 36

### [SEC-05] Pas de limite taille messages WS
- Fichier : `backend/src/webrtc.rs`
- Prévu : session 36

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
| SEC-01 | 35 | `{@html}` sans sanitisation → XSS | `sanitize.ts` DOMPurify + `sanitizeHtml()` dans chat |
| SEC-01b | 35 | Pas de CSP | `app.html` meta Content-Security-Policy |
| R33a | 33 | `GET /chess/{id}/moves` → SAN vs UCI | `e2e.spec.ts` : `expect(body).toContain('e2e4')` |
| R33b | 33 | `.cell-last` absent après reload | `chessStore.svelte.ts` : restauration `lastMove` |
| R33c | 33 | `loginAsAdmin` 429 retry insuffisant | boucle for (2 retries × 6s) |
| R32a | 32 | `GET /chess/{id}/moves` retourne objet vs array | `chess.rs` : `Json(moves_json)` |
| R32b | 32 | `POST /polls/{id}/vote` sans `success:true` | `polls.rs` |
| R32c | 32 | Test chess IA : `ai_difficulty` vs `opponent` | `e2e.spec.ts` |
| R32d | 32 | Chat race condition `waitForResponse` | `expect('.conversation-item').toBeVisible` |
| R32e | 32 | `loginAsAdmin` 429 pollution quota global | `describe.serial('Rate Limiting')` |
| R32f | 32 | `loginAs` flaky sous charge CI | Retry automatique |
| DT-05 | — | WebRTC WAN instable (TURN absent) | Non résolu — prévu |
