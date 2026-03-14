// frontend/tests/api-sanity.spec.ts
// Tests 401/403 sans session — rapides, aucun login, jamais affectés par le rate limit.
// Vérifie que toutes les routes protégées rejettent les requêtes non authentifiées.

import { test, expect } from '@playwright/test';

const BASE = 'http://localhost:6300/api';

test.describe('API Sanity — Non authentifié', () => {

  test('GET /api/health → OK', async ({ request }) => {
    const res = await request.get(`${BASE}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

  // Routes qui doivent retourner 401 sans cookie
  const routes401: Array<{ method: 'GET' | 'POST' | 'DELETE'; path: string; body?: object }> = [
    { method: 'GET',  path: '/auth/me' },
    { method: 'GET',  path: '/conversations' },
    { method: 'GET',  path: '/conversations/default_global/messages' },
    { method: 'GET',  path: '/conversations/default_global/participants' },
    { method: 'GET',  path: '/events' },
    { method: 'GET',  path: '/chess/list' },
    { method: 'GET',  path: '/polls' },
    { method: 'GET',  path: '/invites' },
    { method: 'GET',  path: '/users/available' },
    { method: 'GET',  path: '/analytics' },
    { method: 'GET',  path: '/auth/public-keys?conversation_id=default_global' },
    { method: 'GET',  path: '/download/fake-id-000' },
    { method: 'POST', path: '/conversations', body: { name: 'x' } },
    { method: 'POST', path: '/conversations/default_global/messages', body: { content: 'x' } },
    { method: 'POST', path: '/polls', body: { question: 'x', options: ['a', 'b'] } },
    { method: 'POST', path: '/chess/create', body: { color: 'white' } },
    { method: 'POST', path: '/auth/public-key', body: { public_key: 'abc' } },
    { method: 'POST', path: '/push/subscribe', body: { endpoint: 'x', keys: { p256dh: 'x', auth: 'x' } } },
    { method: 'POST', path: '/conversations/default_global/messages/x/reactions', body: { emoji: '👍' } },
    { method: 'DELETE', path: '/conversations/default_global/messages/x/reactions' },
  ];

  for (const route of routes401) {
    test(`${route.method} ${route.path} → 401`, async ({ request }) => {
      const res = route.method === 'GET'
        ? await request.get(`${BASE}${route.path}`)
        : route.method === 'DELETE'
          ? await request.delete(`${BASE}${route.path}`)
          : await request.post(`${BASE}${route.path}`, { data: route.body });
      expect(res.status()).toBe(401);
    });
  }

  // Routes admin uniquement → 401 sans cookie (pas 403 car pas de session du tout)
  test('GET /api/users/pending sans auth → 401', async ({ request }) => {
    const res = await request.get(`${BASE}/users/pending`);
    expect(res.status()).toBe(401);
  });

  test('GET /api/users sans auth → 401', async ({ request }) => {
    const res = await request.get(`${BASE}/users`);
    expect(res.status()).toBe(401);
  });

  // Upload sans auth
  test('POST /api/upload/chat sans auth → 401', async ({ request }) => {
    const res = await request.post(`${BASE.replace('/api', '')}/api/upload/chat`, {
      multipart: {
        file: { name: 'x.txt', mimeType: 'text/plain', buffer: Buffer.from('x') },
        conversation_id: 'default_global',
      },
    });
    expect(res.status()).toBe(401);
  });

});
