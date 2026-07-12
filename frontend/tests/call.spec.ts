// frontend/tests/call.spec.ts
// Tests E2E pour les appels audio/vidéo WebRTC dans Nook
// Vérifie le flux complet: signalisation, ICE, état d'appel

import { test, expect } from '@playwright/test';

const BASE = `${process.env.NOOK_BASE_URL || 'http://localhost:6300'}/api`;

// ════════════════════════════════════════════════════
// Tests API WebRTC (signalisation via REST)
// ════════════════════════════════════════════════════

test.describe('WebRTC — Signalisation API', () => {

  test('POST /api/webrtc/offer → 401 sans auth', async ({ request }) => {
    const res = await request.post(`${BASE}/webrtc/offer`, {
      data: { offer: 'test', conversation_id: 'default_global' },
    });
    expect(res.status()).toBe(401);
  });

  test('POST /api/webrtc/answer → 401 sans auth', async ({ request }) => {
    const res = await request.post(`${BASE}/webrtc/answer`, {
      data: { answer: 'test', conversation_id: 'default_global' },
    });
    expect(res.status()).toBe(401);
  });

  test('POST /api/webrtc/offer → 200 avec auth', async ({ request }) => {
    // Login d'abord
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (!login.ok()) return;

    const res = await request.post(`${BASE}/webrtc/offer`, {
      data: { offer: 'v=0\r\ntest', conversation_id: 'default_global' },
    });
    expect([200, 201]).toContain(res.status());
  });

  test('POST /api/webrtc/answer → 200 avec auth', async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (!login.ok()) return;

    const res = await request.post(`${BASE}/webrtc/answer`, {
      data: { answer: 'v=0\r\ntest', conversation_id: 'default_global' },
    });
    expect([200, 201]).toContain(res.status());
  });
});

// ════════════════════════════════════════════════════
// Tests appel via API (avec auth complète)
// ════════════════════════════════════════════════════

test.describe('WebRTC — Flux appel complet', () => {

  let adminCookie: string;
  let adminId: string;

  test.beforeAll(async ({ request }) => {
    const login = await request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (login.ok()) {
      const headers = login.headers();
      const cookies = headers['set-cookie'];
      if (cookies) {
        adminCookie = Array.isArray(cookies) ? cookies[0] : cookies;
      }
      const body = await login.json();
      adminId = body.user?.id ?? '';
    }
  });

  test('Créer offre WebRTC → 200', async ({ request }) => {
    if (!adminCookie) return;

    const res = await request.post(`${BASE}/webrtc/offer`, {
      headers: { Cookie: adminCookie },
      data: {
        offer: 'v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-',
        conversation_id: 'default_global',
      },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.status).toBe('offer_sent');
  });

  test('Créer réponse WebRTC → 200', async ({ request }) => {
    if (!adminCookie) return;

    const res = await request.post(`${BASE}/webrtc/answer`, {
      headers: { Cookie: adminCookie },
      data: {
        answer: 'v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-',
        conversation_id: 'default_global',
      },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.status).toBe('answer_sent');
  });
});
