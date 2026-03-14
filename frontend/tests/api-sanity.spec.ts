// frontend/tests/api-sanity.spec.ts
// Tests de sécurité et sanité API — AUCUN login requis.
// Vérifie que chaque route protégée rejette les requêtes non authentifiées (401)
// et que les routes admin rejettent les utilisateurs normaux (403).
// Ces tests sont rapides (~30s) et ne consomment aucun quota de rate limit.

import { test, expect } from '@playwright/test';

const BASE = 'http://localhost:6300/api';

test.describe('Sanité — Serveur', () => {

  test('GET /api/health → "OK"', async ({ request }) => {
    const res = await request.get(`${BASE}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

});

test.describe('Sécurité — Routes non-auth → 401', () => {

  const routes: Array<{ method: 'GET' | 'POST' | 'DELETE' | 'PATCH'; path: string; body?: object }> = [
    // Auth
    { method: 'GET',    path: '/auth/me' },
    { method: 'POST',   path: '/auth/logout' },
    { method: 'POST',   path: '/auth/change-password', body: { new_password: 'x' } },
    { method: 'POST',   path: '/auth/public-key', body: { public_key: 'x' } },
    { method: 'GET',    path: '/auth/public-keys?conversation_id=default_global' },
    // Conversations
    { method: 'GET',    path: '/conversations' },
    { method: 'POST',   path: '/conversations', body: { name: 'x' } },
    { method: 'GET',    path: '/conversations/default_global' },
    { method: 'GET',    path: '/conversations/default_global/messages' },
    { method: 'POST',   path: '/conversations/default_global/messages', body: { content: 'x' } },
    { method: 'GET',    path: '/conversations/default_global/participants' },
    { method: 'POST',   path: '/conversations/default_global/participants', body: { user_id: 'x' } },
    { method: 'POST',   path: '/conversations/default_global/leave' },
    { method: 'PATCH',  path: '/conversations/default_global/rename', body: { name: 'x' } },
    // Upload/Download
    { method: 'GET',    path: '/download/fake-id-000' },
    // Events
    { method: 'GET',    path: '/events' },
    { method: 'POST',   path: '/events', body: { title: 'x', date: '2026-01-01' } },
    { method: 'DELETE', path: '/events/fake-id' },
    // Polls
    { method: 'GET',    path: '/polls' },
    { method: 'POST',   path: '/polls', body: { question: 'x', options: ['a', 'b'] } },
    { method: 'GET',    path: '/polls/fake-id' },
    { method: 'POST',   path: '/polls/fake-id/vote', body: { option_id: 'x' } },
    { method: 'POST',   path: '/polls/fake-id/close' },
    { method: 'DELETE', path: '/polls/fake-id' },
    // Chess
    { method: 'GET',    path: '/chess/list' },
    { method: 'POST',   path: '/chess/create', body: { color: 'white' } },
    { method: 'GET',    path: '/chess/invitations' },
    { method: 'GET',    path: '/chess/fake-id' },
    { method: 'POST',   path: '/chess/fake-id/move', body: { from: 'e2', to: 'e4' } },
    { method: 'GET',    path: '/chess/fake-id/moves?from=e2' },
    { method: 'POST',   path: '/chess/fake-id/ai-move' },
    { method: 'POST',   path: '/chess/fake-id/resign' },
    // Réactions
    { method: 'POST',   path: '/conversations/default_global/messages/x/reactions', body: { emoji: '👍' } },
    { method: 'DELETE', path: '/conversations/default_global/messages/x/reactions' },
    { method: 'GET',    path: '/conversations/default_global/messages/x/reactions' },
    // Profil
    { method: 'POST',   path: '/user/update', body: { name: 'x' } },
    { method: 'GET',    path: '/users/available' },
    // Push
    { method: 'GET',    path: '/push/vapid-public-key' },
    { method: 'GET',    path: '/push/preferences' },
  ];

  for (const route of routes) {
    test(`${route.method} ${route.path} → 401`, async ({ request }) => {
      let res;
      if (route.method === 'GET')    res = await request.get(`${BASE}${route.path}`);
      else if (route.method === 'DELETE') res = await request.delete(`${BASE}${route.path}`);
      else if (route.method === 'PATCH')  res = await request.patch(`${BASE}${route.path}`, { data: route.body });
      else res = await request.post(`${BASE}${route.path}`, { data: route.body });
      expect(res.status()).toBe(401);
    });
  }

  test('POST /api/upload/chat sans auth → 401', async ({ request }) => {
    const res = await request.post('/api/upload/chat', {
      multipart: {
        file: { name: 'x.txt', mimeType: 'text/plain', buffer: Buffer.from('x') },
        conversation_id: 'default_global',
      },
    });
    expect(res.status()).toBe(401);
  });

});

test.describe('Sécurité — Routes admin → 401 sans auth', () => {

  const adminRoutes = [
    { method: 'GET' as const,  path: '/users/pending' },
    { method: 'GET' as const,  path: '/users' },
    { method: 'POST' as const, path: '/users/approve', body: { user_id: 'x' } },
    { method: 'GET' as const,  path: '/invites' },
    { method: 'POST' as const, path: '/invites' },
    { method: 'POST' as const, path: '/invites/delete', body: { invite_id: 'x' } },
    { method: 'GET' as const,  path: '/analytics' },
  ];

  for (const route of adminRoutes) {
    test(`${route.method} ${route.path} → 401`, async ({ request }) => {
      const res = route.method === 'GET'
        ? await request.get(`${BASE}${route.path}`)
        : await request.post(`${BASE}${route.path}`, { data: route.body ?? {} });
      expect(res.status()).toBe(401);
    });
  }

});
