# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: api-sanity.spec.ts >> Sécurité — Change password autre user → 403 >> User normal change pwd autre user → 403
- Location: tests/api-sanity.spec.ts:161:3

# Error details

```
Error: expect(received).toContain(expected) // indexOf

Expected value: 429
Received array: [200, 401]
```

# Test source

```ts
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
  86  |       else res = await request.post(`${BASE}${route.path}`, { data: route.body });
  87  |       expect(res.status()).toBe(401);
  88  |     });
  89  |   }
  90  | 
  91  |   test('POST /api/upload/chat sans auth → 401', async ({ request }) => {
  92  |     const res = await request.post(`${BASE}/upload/chat`, {
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
> 166 |     expect([200, 401]).toContain(login.status()); // 401 if not approved yet
      |                        ^ Error: expect(received).toContain(expected) // indexOf
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
  187 |         },
  188 |       });
  189 |       expect(res.status()).toBe(400);
  190 |     }
  191 |   });
  192 | });
  193 | 
  194 | test.describe('Sécurité — Upload/Download end-to-end', () => {
  195 |   test('Upload fichier texte → file_id, puis download OK', async ({ request }) => {
  196 |     const login = await request.post(`${BASE}/auth/login`, {
  197 |       data: { username: 'admin', password: 'changeme2026' },
  198 |     });
  199 |     if (login.ok()) {
  200 |       const upload = await request.post(`${BASE}/upload/chat`, {
  201 |         multipart: {
  202 |           file: { name: 'test.txt', mimeType: 'text/plain', buffer: Buffer.from('Test content for CI download') },
  203 |           conversation_id: 'default_global',
  204 |           from_user_id: 'admin-initial-id-0000-0000-000000000001',
  205 |         },
  206 |       });
  207 |       expect(upload.status()).toBe(200);
  208 |       const body = await upload.json();
  209 |       expect(body.file_id).toBeTruthy();
  210 |       const fileId = body.file_id;
  211 | 
  212 |       // Download the uploaded file
  213 |       const dl = await request.get(`${BASE}/download/${fileId}`);
  214 |       expect(dl.status()).toBe(200);
  215 |     }
  216 |   });
  217 | 
  218 |   test('Download fichier inexistant → 404', async ({ request }) => {
  219 |     const login = await request.post(`${BASE}/auth/login`, {
  220 |       data: { username: 'admin', password: 'changeme2026' },
  221 |     });
  222 |     if (login.ok()) {
  223 |       const res = await request.get(`${BASE}/download/nonexistent-id-12345`);
  224 |       expect(res.status()).toBe(404);
  225 |     }
  226 |   });
  227 | });
  228 | 
  229 | test.describe('Sécurité — Message conversation CRUD', () => {
  230 |   test('Envoyer message → 200, récupérer → contient message', async ({ request }) => {
  231 |     const login = await request.post(`${BASE}/auth/login`, {
  232 |       data: { username: 'admin', password: 'changeme2026' },
  233 |     });
  234 |     if (login.ok()) {
  235 |       // Send
  236 |       const send = await request.post(`${BASE}/conversations/default_global/messages`, {
  237 |         data: { content: 'Message test CI API', encrypted: false },
  238 |       });
  239 |       expect(send.status()).toBe(200);
  240 |       const sendBody = await send.json();
  241 |       expect(sendBody.content).toContain('Message test CI API');
  242 |       const msgId = sendBody.id;
  243 | 
  244 |       // Edit
  245 |       const edit = await request.patch(`${BASE}/conversations/default_global/messages/${msgId}`, {
  246 |         data: { content: 'Message modifié CI' },
  247 |       });
  248 |       expect(edit.status()).toBe(200);
  249 | 
  250 |       // List
  251 |       const list = await request.get(`${BASE}/conversations/default_global/messages`);
  252 |       expect(list.status()).toBe(200);
  253 |       const listBody = await list.json();
  254 |       expect(Array.isArray(listBody)).toBe(true);
  255 |       const found = listBody.find((m: any) => m.content?.includes('modifié'));
  256 |       expect(found).toBeTruthy();
  257 | 
  258 |       // Delete
  259 |       const del = await request.delete(`${BASE}/conversations/default_global/messages/${msgId}`);
  260 |       expect([200, 204]).toContain(del.status());
  261 |     }
  262 |   });
  263 | 
  264 |   test('Rename conversation → 200', async ({ request }) => {
  265 |     const login = await request.post(`${BASE}/auth/login`, {
  266 |       data: { username: 'admin', password: 'changeme2026' },
```