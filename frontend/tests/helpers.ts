// frontend/tests/helpers.ts
// Helpers partagés entre tous les fichiers de tests E2E Nook.
//
// PRINCIPES :
//   - loginAs() : 1 seul login browser par describe (via beforeAll + storageState)
//   - loginAsAdmin() : API-first, bypass /login pour éviter le rate limit
//   - clearSession() : révoque token + vide cookies + localStorage
//   - ADMIN_NEW_PASSWORD : mot de passe défini lors du 1er login admin en CI

import { expect, type Page, type BrowserContext } from '@playwright/test';

export const ADMIN_NEW_PASSWORD = 'AdminCI2026!';
export const E2E_USER = 'e2e_ci';
export const E2E_PASS = 'E2eTest123!';
export const BASE = `${process.env.NOOK_BASE_URL || 'http://localhost:6300'}/api`;

// ─────────────────────────────────────────────────────────────────
// clearSession — révoque token serveur + vide browser state
// ─────────────────────────────────────────────────────────────────
export async function clearSession(page: Page): Promise<void> {
  try {
    await page.request.post(`${BASE}/auth/logout`);
  } catch { /* pas de session active → OK */ }
  await page.context().clearCookies();
  try {
    await page.evaluate(() => localStorage.clear());
  } catch { /* contexte vierge → OK */ }
}

// ─────────────────────────────────────────────────────────────────
// loginAs — login browser complet (UI réelle)
// À utiliser dans beforeAll d'un describe, UNE SEULE FOIS par suite.
// ─────────────────────────────────────────────────────────────────
export async function loginAs(page: Page, username: string, password: string): Promise<void> {
  await clearSession(page);
  
  // API-first login: set cookie via API, then navigate
  const loginRes = await page.request.post(`${BASE}/auth/login`, {
    data: { username, password },
  });
  
  if (!loginRes.ok()) {
    const body = await loginRes.json().catch(() => ({}));
    throw new Error(`loginAs(${username}) API failed: HTTP ${loginRes.status()} - ${body.message || 'unknown'}`);
  }
  
  // Cookie is now set, navigate to app
  await page.goto('/chat');
  await page.waitForLoadState('networkidle', { timeout: 15_000 });
}

// ─────────────────────────────────────────────────────────────────
// loginViaAPI — login API-first (pas de browser, pas de rate limit)
// Pose le cookie auth_token directement dans le context.
// À utiliser pour tous les tests API qui n'ont pas besoin de l'UI.
// ─────────────────────────────────────────────────────────────────
export async function loginViaAPI(page: Page, username: string, password: string): Promise<void> {
  await clearSession(page);
  const res = await page.request.post(`${BASE}/auth/login`, {
    data: { username, password },
  });
  if (!res.ok()) throw new Error(`loginViaAPI(${username}) échoué : HTTP ${res.status()}`);
}

// ─────────────────────────────────────────────────────────────────
// loginAsAdmin — flux complet admin (change-password si nécessaire)
// API-first pour éviter le rate limit sur /auth/login.
// ─────────────────────────────────────────────────────────────────
export async function loginAsAdmin(page: Page): Promise<void> {
  await clearSession(page);

  // Essai 1 : mdp déjà changé (run 2+)
  let res = await page.request.post(`${BASE}/auth/login`, {
    data: { username: 'admin', password: ADMIN_NEW_PASSWORD },
  });

  // Essai 2 : mdp initial (premier run, DB fraîche)
  if (!res.ok()) {
    res = await page.request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: 'changeme2026' },
    });
    if (!res.ok()) throw new Error(`Login admin échoué : HTTP ${res.status()}`);
  }

  const body = await res.json();

  // Changement de mot de passe obligatoire au premier login
  if (body.user?.needs_password_change) {
    const chgRes = await page.request.post(`${BASE}/auth/change-password`, {
      data: { new_password: ADMIN_NEW_PASSWORD, user_id: body.user.id },
    });
    if (!chgRes.ok()) throw new Error(`Change-password admin échoué : HTTP ${chgRes.status()}`);

    // Re-login avec le nouveau mot de passe
    res = await page.request.post(`${BASE}/auth/login`, {
      data: { username: 'admin', password: ADMIN_NEW_PASSWORD },
    });
    if (!res.ok()) throw new Error(`Re-login admin après change-pwd échoué : HTTP ${res.status()}`);
    console.log('🔐 Mot de passe admin changé');
  }

  // Naviguer sur /admin et attendre que la page soit prête
  await page.goto('/admin');
  await page.locator('.admin-header').waitFor({ state: 'visible', timeout: 15_000 });
  console.log('✅ Admin connecté sur /admin');
}

// ─────────────────────────────────────────────────────────────────
// waitForAppReady — attend la fin du chargement du layout
// ─────────────────────────────────────────────────────────────────
export async function waitForAppReady(page: Page): Promise<void> {
  await expect(
    page.locator('[data-testid="loading-screen"]')
  ).not.toBeVisible({ timeout: 15_000 });
}
