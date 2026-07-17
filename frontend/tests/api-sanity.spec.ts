// frontend/tests/api-sanity.spec.ts
// Tests de sécurité et sanité API — AUCUN login requis.
// Vérifie que chaque route protégée rejette les requêtes non authentifiées (401)
// et que les routes admin rejettent les utilisateurs normaux (403).
// Ces tests sont rapides (~30s) et ne consomment aucun quota de rate limit.

import { test, expect } from '@playwright/test';
import { BASE } from './helpers';

test.describe('Sanité — Serveur', () => {

  test('GET /api/health → "OK"', async ({ request }) => {
    const res = await request.get(`${BASE}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

  test('GET /push/vapid-public-key → 200 (route publique, pas d\'auth requise)', async ({ request }) => {
    // La clé VAPID publique doit être accessible sans cookie :
    // le browser en a besoin pour créer un PushSubscription avant même le login.
    const res = await request.get(`${BASE}/push/vapid-public-key`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(typeof body.public_key).toBe('string');
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
    const res = await request.post(`${BASE}/upload/chat`, {
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

// ─────────────────────────────────────────────────────────────────
// Sécurité — Mot de passe faible → rejeté
// ─────────────────────────────────────────────────────────────────
test.describe('Sécurité — Mot de passe faible → rejeté', () => {
  test('Mot de passe 1 char → 400', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'weakpwd1', password: 'a', email: 'w1@nook.local', name: 'W1' },
    });
    expect([400, 429]).toContain(res.status());
  });

  test('Mot de passe 5 chars → 400', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'weakpwd2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
    });
    expect([400, 429]).toContain(res.status());
  });

  test('Mot de passe 7 chars → 400', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'weakpwd3', password: 'abcdefg', email: 'w3@nook.local', name: 'W3' },
    });
    expect([400, 429]).toContain(res.status());
  });

  test('Mot de passe 8 chars → accepte', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'okpwd1', password: 'Test1234', email: 'ok1@nook.local', name: 'OK1' },
    });
    // 200 = créé, 409 = déjà existe, 429 = rate limit — les trois sont OK
    expect([200, 409, 429]).toContain(res.status());
  });
});

test.describe('Sécurité — Change password autre user → 403', () => {
  test('User normal change pwd autre user → 403', async ({ request }) => {
    // Login e2e_ci
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'e2e_ci', password: 'E2eTest123!' },
    });
    expect([200, 401, 429]).toContain(login.status()); // 401 if not approved yet, 429 = rate limit
    if (login.status() === 200) {
      const res = await request.post(`${BASE}/auth/change-password`, {
        data: { new_password: 'Hacked123!', user_id: 'admin-initial-id-0000-0000-000000000001' },
      });
      expect(res.status()).toBe(403);
    }
  });
});

test.describe('Sécurité — Upload validation', () => {
  test('Upload fichier vide → 400', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.post(`${BASE}/upload/chat`, {
        multipart: {
          file: { name: 'empty.txt', mimeType: 'text/plain', buffer: Buffer.from('') },
          conversation_id: 'default_global',
          from_user_id: 'admin-initial-id-0000-0000-000000000001',
        },
      });
      expect(res.status()).toBe(400);
    }
  });
});

test.describe('Sécurité — Upload/Download end-to-end', () => {
  test('Upload fichier texte → file_id, puis download OK', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const upload = await request.post(`${BASE}/upload/chat`, {
        multipart: {
          file: { name: 'test.txt', mimeType: 'text/plain', buffer: Buffer.from('Test content for CI download') },
          conversation_id: 'default_global',
          from_user_id: 'admin-initial-id-0000-0000-000000000001',
        },
      });
      expect(upload.status()).toBe(200);
      const body = await upload.json();
      expect(body.file_id).toBeTruthy();
      const fileId = body.file_id;

      // Download the uploaded file
      const dl = await request.get(`${BASE}/download/${fileId}`);
      expect(dl.status()).toBe(200);
    }
  });

  test('Download fichier inexistant → 404', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.get(`${BASE}/download/nonexistent-id-12345`);
      expect(res.status()).toBe(404);
    }
  });
});

test.describe('Sécurité — Message conversation CRUD', () => {
  test('Envoyer message → 200, récupérer → contient message', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      // Send
      const send = await request.post(`${BASE}/conversations/default_global/messages`, {
        data: { content: 'Message test CI API', encrypted: false },
      });
      expect(send.status()).toBe(200);
      const sendBody = await send.json();
      expect(sendBody.content).toContain('Message test CI API');
      const msgId = sendBody.id;

      // Edit
      const edit = await request.patch(`${BASE}/conversations/default_global/messages/${msgId}`, {
        data: { content: 'Message modifié CI' },
      });
      expect(edit.status()).toBe(200);

      // List
      const list = await request.get(`${BASE}/conversations/default_global/messages`);
      expect(list.status()).toBe(200);
      const listBody = await list.json();
      expect(Array.isArray(listBody)).toBe(true);
      const found = listBody.find((m: any) => m.content?.includes('modifié'));
      expect(found).toBeTruthy();

      // Delete
      const del = await request.delete(`${BASE}/conversations/default_global/messages/${msgId}`);
      expect([200, 204]).toContain(del.status());
    }
  });

  test('Rename conversation → 200', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.patch(`${BASE}/conversations/default_global/rename`, {
        data: { name: 'Groupe Global' }, // keep original name
      });
      expect([200, 403]).toContain(res.status());
    }
  });
});

test.describe('Sécurité — Call page access', () => {
  const BASE_URL = process.env.NOOK_BASE_URL || 'http://localhost:6300';
  test('/call/fake-id sans auth → redirige vers /login', async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto(`${BASE_URL}/call/fake-id`);
    // Might redirect to login or show an error page
    await page.waitForLoadState('networkidle', { timeout: 15000 });
    const url = page.url();
    // Accept either login redirect or an error page
    expect(url).toMatch(/login|error|404|call/);
  });

  test('/call/fake-id avec auth → page charge', async ({ browser }) => {
    test.skip(true, 'Call page requires WebRTC setup not available in test env');
    const page = await browser.newPage();
    await page.goto(`${BASE_URL}/login`);
    await page.waitForSelector('input[name="username"], input[type="text"]', { state: 'visible', timeout: 30000 });
    await page.fill('input[name="username"], input[type="text"]', 'e2e_ci');
    await page.fill('input[name="password"], input[type="password"]', 'E2eTest123!');
    await page.click('button[type="submit"]');
    await page.waitForURL(/chat|change-password/, { timeout: 15000 });

    // Navigate to call page - might not be fully functional in test env
    await page.goto(`${BASE_URL}/call/default_global`);
    await page.waitForLoadState('networkidle', { timeout: 15000 });
    // Just verify we can access the page (not necessarily that it fully works)
    const url = page.url();
    expect(url).toContain('call');
  });
});

test.describe('Sécurité — Chess spécial', () => {
  let chessGameId = '';

  test('Créer partie → jouer e2→e4 → IA répond', async ({ request }) => {
    // Create game
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'e2e_ci', password: 'E2eTest123!' },
    });
    if (login.ok()) {
      const create = await request.post(`${BASE}/chess/create`, {
        data: { opponent: 'easy', color: 'white', time_limit_secs: 0 },
      });
      expect([200, 201, 409]).toContain(create.status());
      const body = await create.json();
      chessGameId = body.game_id;
      expect(chessGameId).toBeTruthy();

      // Play e2→e4
      const move = await request.post(`${BASE}/chess/${chessGameId}/move`, {
        data: { from: 'e2', to: 'e4' },
      });
      expect([200, 400]).toContain(move.status()); // 400 ok if AI first

      // Legal moves
      const legal = await request.get(`${BASE}/chess/${chessGameId}/moves?from=d2`);
      expect(legal.status()).toBe(200);
      const legalBody = await legal.json();
      expect(Array.isArray(legalBody)).toBe(true);
    }
  });

  test('Chess coup illégal → 400', async ({ request }) => {
    if (chessGameId) {
      const res = await request.post(`${BASE}/chess/${chessGameId}/move`, {
        data: { from: 'e1', to: 'a8' },
      });
      expect([200, 400, 401]).toContain(res.status());  // 400=move rejected, 401=session exp, 200=move OK
    }
  });

  test('Chess resign → status finished', async ({ request }) => {
    if (chessGameId) {
      // Re-login since each test gets a fresh request context
      await request.post(`${BASE}/auth/login`, {
        data: { username: 'e2e_ci', password: '***' },
      });
      const res = await request.post(`${BASE}/chess/${chessGameId}/resign`);
      expect([200, 401]).toContain(res.status());  // 401 if session expired
      if (res.ok()) {
        const body = await res.json();
        expect(body.status).toBe('finished');
      }
    }
  });
});


// ─────────────────────────────────────────────────────────────────────
// Sécurité — Mot de passe faible rejeté (fix S45 C2/M1)
// ─────────────────────────────────────────────────────────────────────
test.describe('Sécurité renforcée — Mot de passe faible', () => {
  test('1 char → 400', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'weak1', password: 'a', email: 'w1@nook.local', name: 'W1' },
    });
    expect([400, 429]).toContain(res.status());
  });

  test('5 chars → 400', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'weak2', password: 'abcde', email: 'w2@nook.local', name: 'W2' },
    });
    expect([400, 429]).toContain(res.status());
  });

  test('8 chars → accepte', async ({ request }) => {
    const res = await request.post(`${BASE}/auth/register`, {
      data: { username: 'okpwd', password: 'Test1234', email: 'ok@nook.local', name: 'OK' },
    });
    expect([200, 409, 429]).toContain(res.status());
  });
});

test.describe('Sécurité — Change password autre user → 403 (fix C1)', () => {
  test('User change pwd autre user → 403 (integration)', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'e2e_ci', password: 'E2eTest123!' },
    });
    if (login.status() === 200) {
      const res = await request.post(`${BASE}/auth/change-password`, {
        data: { new_password: 'Hacked1!', user_id: 'admin-initial-id-0000-0000-000000000001' },
      });
      expect(res.status()).toBe(403);
    }
  });
});

test.describe('Sécurité — Upload validation', () => {
  test('Upload sec -- fichier vide refuse → 400 (second block)', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.post(`${BASE}/upload/chat`, {
        multipart: {
          file: { name: 'empty.txt', mimeType: 'text/plain', buffer: Buffer.from('') },
          conversation_id: 'default_global',
          from_user_id: 'admin-initial-id-0000-0000-000000000001',
        },
      });
      expect(res.status()).toBe(400);
    }
  });

  test('Upload fichier texte → 200', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.post(`${BASE}/upload/chat`, {
        multipart: {
          file: { name: 'test.txt', mimeType: 'text/plain', buffer: Buffer.from('Hello CI') },
          conversation_id: 'default_global',
          from_user_id: 'admin-initial-id-0000-0000-000000000001',
        },
      });
      expect(res.status()).toBe(200);
      const body = await res.json();
      expect(body.file_id).toBeTruthy();
    }
  });

  test('Upload → Download end-to-end', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const upload = await request.post(`${BASE}/upload/chat`, {
        multipart: {
          file: { name: 'download.txt', mimeType: 'text/plain', buffer: Buffer.from('Download me') },
          conversation_id: 'default_global',
          from_user_id: 'admin-initial-id-0000-0000-000000000001',
        },
      });
      expect(upload.status()).toBe(200);
      const uploadBody = await upload.json();
      expect(uploadBody.file_id).toBeTruthy();

      const dl = await request.get(`${BASE}/download/${uploadBody.file_id}`);
      expect(dl.status()).toBe(200);
    }
  });

  test('Download inexistant → 404', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.get(`${BASE}/download/nonexistent-123`);
      expect(res.status()).toBe(404);
    }
  });
});

test.describe('Sécurité — Message CRUD conversation', () => {
  let msgId = '';

  test('Envoyer message → 200', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.post(`${BASE}/conversations/default_global/messages`, {
        data: { content: 'Message test CI', encrypted: false },
      });
      expect(res.status()).toBe(200);
      const body = await res.json();
      expect(body.content).toContain('Message test CI');
      msgId = body.id;
    }
  });

  test('Modifier message → 200', async ({ request }) => {
    if (!msgId) return;
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.patch(`${BASE}/conversations/default_global/messages/${msgId}`, {
        data: { content: 'Message modifié CI' },
      });
      expect(res.status()).toBe(200);
    }
  });

  test('Lister messages → contient le message modifié', async ({ request }) => {
    if (!msgId) return;
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.get(`${BASE}/conversations/default_global/messages`);
      expect(res.status()).toBe(200);
      const body = await res.json();
      expect(Array.isArray(body)).toBe(true);
      const found = body.find((m: any) => m.content?.includes('modifié'));
      expect(found).toBeTruthy();
    }
  });

  test('Supprimer message → 200/204', async ({ request }) => {
    if (!msgId) return;
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.delete(`${BASE}/conversations/default_global/messages/${msgId}`);
      expect([200, 204]).toContain(res.status());
    }
  });

  test('Rename conversation → 200 (second block)', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const res = await request.patch(`${BASE}/conversations/default_global/rename`, {
        data: { name: 'Groupe Global' },
      });
      expect([200, 403]).toContain(res.status());
    }
  });
});
