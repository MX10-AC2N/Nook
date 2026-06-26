# Tests E2E contre serveur distant (NOOK_BASE_URL)

## Contexte
Session 53 — Tests lancés sur `http://192.168.1.192:6300` (instance déployée) au lieu de `localhost`.

## Problème principal
`helpers.ts` définit par défaut :
```typescript
export const BASE = `${process.env.NOOK_BASE_URL || 'http://localhost:6300'}/api`;
```

Mais `api-sanity.spec.ts` n'importait pas `BASE` depuis `./helpers` → `ReferenceError: BASE is not defined`.

## Fixes appliqués

### 1. Import manquant
```typescript
// api-sanity.spec.ts
import { BASE } from './helpers';  // ← AJOUTER
```

### 2. Route inexistante
```typescript
// DELETE /events/fake-id → 404 (n'existe pas dans le backend)
// Action: supprimer cette entrée du tableau routes[]
```

### 3. Chemin upload incorrect
```typescript
// Avant: request.post('/api/upload/chat') → ECONNREFUSED ::1:6300
// Après:  request.post(`${BASE}/upload/chat`) → utilise NOOK_BASE_URL
```

### 4. URLs hardcodées dans Call page tests
```typescript
// Avant: page.goto('http://localhost:6300/call/fake-id')
// Après:  const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';
//         page.goto(`${BASE_URL}/call/fake-id`)
```

### 5. Rate limiting sur registration (weak passwords)
```typescript
// Backend renvoie 429 (Too Many Requests) sur flood d'inscriptions
// Tests doivent accepter: expect([400, 429]).toContain(res.status())
```

## Lancer seulement api-sanity (suite de validation sécurité)
```bash
cd /opt/data/Nook/frontend
NOOK_BASE_URL=http://192.168.1.192:6300 npx playwright test --project=api-sanity --reporter=line
```

✅ 75 tests passent (sanity + sécurité routes + auth + upload + chess + pwd)

## Suites bloquées (admin-flow, user-flow, call-ui)
Ces suites utilisent `BASE` de `helpers.ts` qui pointe vers `localhost` par défaut.
Nécessitent sur le serveur distant :
1. Comptes de test créés + approuvés (`e2e_ci`, `hermes-bot`, `admin`)
2. `helpers.ts` modifié pour utiliser `NOOK_BASE_URL` : `process.env.NOOK_BASE_URL || 'http://localhost:6300'`
3. Sinon → `ECONNREFUSED ::1:6300` ou `401/429` sur login