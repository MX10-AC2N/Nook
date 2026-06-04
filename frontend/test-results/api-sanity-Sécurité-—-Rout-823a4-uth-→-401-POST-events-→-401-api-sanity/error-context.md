# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: api-sanity.spec.ts >> Sécurité — Routes non-auth → 401 >> POST /events → 401
- Location: tests/api-sanity.spec.ts:81:5

# Error details

```
ReferenceError: BASE is not defined
```

# Test source

```ts
  1   | // frontend/tests/api-sanity.spec.ts
  2   | // Tests de sécurité et sanité API — AUCUN login requis.
  3   | // Vérifie que chaque route protégée rejette les requêtes non authentifiées (401)
  4   | // et que les routes admin rejettent les utilisateurs normaux (403).
  5   | // Ces tests sont rapides (~30s) et ne consomment aucun quota de rate limit.
  6   | 
  7   | import { test, expect } from '@playwright/test';
  8   | 
  9   | test.describe('Sanité — Serveur', () => {
  10  | 
  11  |   test('GET /api/health → "OK"', async ({ request }) => {
  12  |     const res = await request.get(`${BASE}/health`);
  13  |     expect(res.status()).toBe(200);
  14  |     expect((await res.text()).trim()).toBe('OK');
  15  |   });
  16  | 
  17  |   test('GET /push/vapid-public-key → 200 (route publique, pas d\'auth requise)', async ({ request }) => {
  18  |     // La clé VAPID publique doit être accessible sans cookie :
  19  |     // le browser en a besoin pour créer un PushSubscription avant même le login.
  20  |     const res = await request.get(`${BASE}/push/vapid-public-key`);
  21  |     expect(res.status()).toBe(200);
  22  |     const body = await res.json();
  23  |     expect(typeof body.public_key).toBe('string');
  24  |   });
  25  | 
  26  | });
  27  | 
  28  | test.describe('Sécurité — Routes non-auth → 401', () => {
  29  | 
  30  |   const routes: Array<{ method: 'GET' | 'POST' | 'DELETE' | 'PATCH'; path: string; body?: object }> = [
  31  |     // Auth
  32  |     { method: 'GET',    path: '/auth/me' },
  33  |     { method: 'POST',   path: '/auth/logout' },
  34  |     { method: 'POST',   path: '/auth/change-password', body: { new_password: 'x' } },
  35  |     { method: 'POST',   path: '/auth/public-key', body: { public_key: 'x' } },
  36  |     { method: 'GET',    path: '/auth/public-keys?conversation_id=default_global' },
  37  |     // Conversations
  38  |     { method: 'GET',    path: '/conversations' },
  39  |     { method: 'POST',   path: '/conversations', body: { name: 'x' } },
  40  |     { method: 'GET',    path: '/conversations/default_global' },
  41  |     { method: 'GET',    path: '/conversations/default_global/messages' },
  42  |     { method: 'POST',   path: '/conversations/default_global/messages', body: { content: 'x' } },
  43  |     { method: 'GET',    path: '/conversations/default_global/participants' },
  44  |     { method: 'POST',   path: '/conversations/default_global/participants', body: { user_id: 'x' } },
  45  |     { method: 'POST',   path: '/conversations/default_global/leave' },
  46  |     { method: 'PATCH',  path: '/conversations/default_global/rename', body: { name: 'x' } },
  47  |     // Upload/Download
  48  |     { method: 'GET',    path: '/download/fake-id-000' },
  49  |     // Events
  50  |     { method: 'GET',    path: '/events' },
  51  |     { method: 'POST',   path: '/events', body: { title: 'x', date: '2026-01-01' } },
  52  |     { method: 'DELETE', path: '/events/fake-id' },
  53  |     // Polls
  54  |     { method: 'GET',    path: '/polls' },
  55  |     { method: 'POST',   path: '/polls', body: { question: 'x', options: ['a', 'b'] } },
  56  |     { method: 'GET',    path: '/polls/fake-id' },
  57  |     { method: 'POST',   path: '/polls/fake-id/vote', body: { option_id: 'x' } },
  58  |     { method: 'POST',   path: '/polls/fake-id/close' },
  59  |     { method: 'DELETE', path: '/polls/fake-id' },
  60  |     // Chess
  61  |     { method: 'GET',    path: '/chess/list' },
  62  |     { method: 'POST',   path: '/chess/create', body: { color: 'white' } },
  63  |     { method: 'GET',    path: '/chess/invitations' },
  64  |     { method: 'GET',    path: '/chess/fake-id' },
  65  |     { method: 'POST',   path: '/chess/fake-id/move', body: { from: 'e2', to: 'e4' } },
  66  |     { method: 'GET',    path: '/chess/fake-id/moves?from=e2' },
  67  |     { method: 'POST',   path: '/chess/fake-id/ai-move' },
  68  |     { method: 'POST',   path: '/chess/fake-id/resign' },
  69  |     // Réactions
  70  |     { method: 'POST',   path: '/conversations/default_global/messages/x/reactions', body: { emoji: '👍' } },
  71  |     { method: 'DELETE', path: '/conversations/default_global/messages/x/reactions' },
  72  |     { method: 'GET',    path: '/conversations/default_global/messages/x/reactions' },
  73  |     // Profil
  74  |     { method: 'POST',   path: '/user/update', body: { name: 'x' } },
  75  |     { method: 'GET',    path: '/users/available' },
  76  |     // Push
  77  |     { method: 'GET',    path: '/push/preferences' },
  78  |   ];
  79  | 
  80  |   for (const route of routes) {
  81  |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  82  |       let res;
  83  |       if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
  84  |       else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
  85  |       else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
> 86  |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
      |                                        ^ ReferenceError: BASE is not defined
  87  |       expect(res.status()).toBe(401);
  88  |     });
  89  |   }
  90  | 
  91  |   test('POST /api/upload/chat sans auth → 401', async ({ request }) => {
  92  |     const res = await request.post('/api/upload/chat', {
  93  |       multipart: {
  94  |         file: { name: 'x.txt', mimeType: 'text/plain', buffer: Buffer.from('x') },
  95  |         conversation_id: 'default_global',
  96  |       },
  97  |     });
  98  |     expect(res.status()).toBe(401);
  99  |   });
  100 | 
  101 | });
  102 | 
  103 | test.describe('Sécurité — Routes admin → 401 sans auth', () => {
  104 | 
  105 |   const adminRoutes = [
  106 |     { method: 'GET' as const,  path: '/users/pending' },
  107 |     { method: 'GET' as const,  path: '/users' },
  108 |     { method: 'POST' as const, path: '/users/approve', body: { user_id: 'x' } },
  109 |     { method: 'GET' as const,  path: '/invites' },
  110 |     { method: 'POST' as const, path: '/invites' },
  111 |     { method: 'POST' as const, path: '/invites/delete', body: { invite_id: 'x' } },
  112 |     { method: 'GET' as const,  path: '/analytics' },
  113 |   ];
  114 | 
  115 |   for (const route of adminRoutes) {
  116 |     test(`${route.method} ${route.path} → 401`, async ({ request }) => {
  117 |       const res = route.method === 'GET'
  118 |         ? await request.get(`${BASE}${route.path}`)
  119 |         : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
  120 |       expect(res.status()).toBe(401);
  121 |     });
  122 |   }
  123 | 
  124 | });
  125 | 
  126 | // ─────────────────────────────────────────────────────────────────
  127 | // Sécurit renforcée — tests des fixes S45
  128 | // ─────────────────────────────────────────────────────────────────
  129 | test.describe('Sécurité — Mot de passe faible → rejeté', () => {
  130 |   test('Mot de passe 1 char → 400', async ({ request }) => {
  131 |     const res = await request.post(`${BASE}/auth/register`, {
  132 |       data: { username: 'weakpwd1', password: 'a', email: 'w1@nook.local', name: 'W1' },
  133 |     });
  134 |     expect(res.status()).toBe(400);
  135 |   });
  136 | 
  137 |   test('Mot de passe 5 chars → 400', async ({ request }) => {
  138 |     const res = await request.post(`${BASE}/auth/register`, {
  139 |       data: { username: 'weakpwd2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
  140 |     });
  141 |     expect(res.status()).toBe(400);
  142 |   });
  143 | 
  144 |   test('Mot de passe 7 chars → 400', async ({ request }) => {
  145 |     const res = await request.post(`${BASE}/auth/register`, {
  146 |       data: { username: 'weakpwd3', password: 'abcdefg', email: 'w3@nook.local', name: 'W3' },
  147 |     });
  148 |     expect(res.status()).toBe(400);
  149 |   });
  150 | 
  151 |   test('Mot de passe 8 chars → accepte', async ({ request }) => {
  152 |     const res = await request.post(`${BASE}/auth/register`, {
  153 |       data: { username: 'okpwd1', password: 'Test1234', email: 'ok1@nook.local', name: 'OK1' },
  154 |     });
  155 |     // 200 = créé, 409 = déjà existe — les deux sont OK
  156 |     expect([200, 409]).toContain(res.status());
  157 |   });
  158 | });
  159 | 
  160 | test.describe('Sécurité — Change password autre user → 403', () => {
  161 |   test('User normal change pwd autre user → 403', async ({ request }) => {
  162 |     // Login e2e_ci
  163 |     const login = await request.post(`${BASE}/auth/login`, {
  164 |       data: { username: 'e2e_ci', password: 'E2eTest123!' },
  165 |     });
  166 |     expect([200, 401]).toContain(login.status()); // 401 if not approved yet
  167 |     if (login.status() === 200) {
  168 |       const res = await request.post(`${BASE}/auth/change-password`, {
  169 |         data: { new_password: 'Hacked123!', user_id: 'admin-initial-id-0000-0000-000000000001' },
  170 |       });
  171 |       expect(res.status()).toBe(403);
  172 |     }
  173 |   });
  174 | });
  175 | 
  176 | test.describe('Sécurité — Upload validation', () => {
  177 |   test('Upload fichier vide → 400', async ({ request }) => {
  178 |     const login = await request.post(`${BASE}/auth/login`, {
  179 |       data: { username: 'admin', password: 'changeme2026' },
  180 |     });
  181 |     if (login.ok()) {
  182 |       const res = await request.post(`${BASE}/upload/chat`, {
  183 |         multipart: {
  184 |           file: { name: 'empty.txt', mimeType: 'text/plain', buffer: Buffer.from('') },
  185 |           conversation_id: 'default_global',
  186 |           from_user_id: 'admin-initial-id-0000-0000-000000000001',
```