// src/lib/api.ts

/**
 * Base URL de l'API (relative à la racine du serveur).
 */
const API_BASE = '/api';

/**
 * Parse la réponse HTTP en JSON (si possible) et renvoie un objet
 * contenant `ok`, `status`, `data` (ou `null` si vide) et `text`.
 *
 * @param {Response} response
 * @returns {Promise<{ ok: boolean; status: number; data: any; text: string }>}
 */
async function parseResponse(response: Response) {
  const text = await response.text();

  // Si le corps est vide, on renvoie `null` comme donnée.
  if (!text.trim()) {
    return { ok: response.ok, status: response.status, data: null, text };
  }

  try {
    const data = JSON.parse(text);
    return { ok: response.ok, status: response.status, data, text };
  } catch (e) {
    // Retourner le texte brut si le JSON est invalide.
    return { ok: false, status: response.status, data: null, text };
  }
}

/**
 * Change le mot de passe d’un utilisateur (ou du compte courant).
 *
 * @param {string} newPassword
 * @param {string} [userId] - Si fourni, change le mot de passe d’un autre utilisateur (admin).
 * @returns {Promise<{ success: boolean; message: string }>}
 */
export async function changePassword(
  newPassword: string,
  userId?: string
): Promise<{ success: boolean; message: string }> {
  try {
    const payload: Record<string, unknown> = { new_password: newPassword };
    if (userId) payload.user_id = userId;

    const response = await fetch(`${API_BASE}/auth/change-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(payload),
    });

    const { ok, status, data, text } = await parseResponse(response);

    if (!ok) {
      // Si le serveur renvoie un message d’erreur, on le privilégie.
      const msg = data?.message ?? `Erreur ${status}: ${text}`;
      return { success: false, message: msg };
    }

    // Succès – on renvoie le message du serveur ou un texte générique.
    return { success: true, message: data?.message ?? 'Mot de passe changé' };
  } catch (err) {
    console.error('Erreur changement mot de passe:', err);
    return {
      success: false,
      message: err instanceof Error ? err.message : 'Erreur de connexion',
    };
  }
}

/**
 * Récupère les informations de l’utilisateur connecté.
 *
 * @returns {Promise<{
 *   id: string;
 *   username: string;
 *   name: string;
 *   role: string;
 *   approved: boolean;
 *   needs_password_change: boolean;
 * } | null>}
 */
export async function getUserInfo(): Promise<{
  id: string;
  username: string;
  name: string;
  role: string;
  approved: boolean;
  needs_password_change: boolean;
} | null> {
  try {
    const response = await fetch(`${API_BASE}/auth/me`, {
      method: 'GET',
      credentials: 'include',
    });

    const { ok, data, text } = await parseResponse(response);

    if (!ok) {
      console.warn(`getUserInfo: réponse non‑OK (${response.status})`);
      return null;
    }

    // Le backend renvoie `{ user: {...} }` ou `{ authenticated: false }`
    return data?.user ?? null;
  } catch (err) {
    console.error('Erreur getUserInfo:', err);
    return null;
  }
}

/**
 * Déconnecte l’utilisateur actuel.
 */
export async function logout(): Promise<void> {
  try {
    await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      credentials: 'include',
    });
  } catch (err) {
    console.error('Erreur logout:', err);
  }
}

/**
 * Récupère le token d’invitation stocké en sessionStorage (si présent) et le supprime.
 *
 * @returns {string|null}
 */
export function getPendingInviteToken(): string | null {
  if (typeof window === 'undefined') return null;

  const token = sessionStorage.getItem('pending_invite_token');
  if (token) sessionStorage.removeItem('pending_invite_token');
  return token;
}

/**
 * Stocke le token d’invitation dans sessionStorage.
 *
 * @param {string} token
 */
export function setPendingInviteToken(token: string): void {
  if (typeof window !== 'undefined') {
    sessionStorage.setItem('pending_invite_token', token);
  }
}
/**
 * Valide un token d'invitation et retourne ses données (nom de la famille, etc.).
 * @param {string} token - Le token à valider.
 * @returns {Promise<{ valid: boolean; familyName?: string; name?: string; expiresAt?: string }>}
 */
export async function validateInviteToken(
  token: string
): Promise<{ valid: boolean; familyName?: string; name?: string; expiresAt?: string }> {
  try {
    const response = await fetch(`${API_BASE}/invite/validate?token=${encodeURIComponent(token)}`, {
      credentials: 'include',
    });
    const { ok, data } = await parseResponse(response);
    return { valid: ok, ...data };
  } catch (err) {
    console.error('Erreur validateInviteToken:', err);
    return { valid: false };
  }
}

/**
 * Accepte une invitation et crée le compte utilisateur.
 * @param {string} token
 * @param {string} username
 * @param {string} name
 * @param {string} password
 * @returns {Promise<{ success: boolean; message: string }>}
 */
export async function acceptInvite(
  token: string,
  username: string,
  name: string,
  password: string
): Promise<{ success: boolean; message: string }> {
  try {
    const response = await fetch(`${API_BASE}/invite/accept`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ token, username, name, password }),
    });
    const { ok, status, data, text } = await parseResponse(response);
    if (!ok) {
      const msg = data?.message ?? `Erreur ${status}: ${text}`;
      return { success: false, message: msg };
    }
    return { success: true, message: data?.message ?? 'Compte créé avec succès' };
  } catch (err) {
    console.error('Erreur acceptInvite:', err);
    return {
      success: false,
      message: err instanceof Error ? err.message : 'Erreur de connexion',
    };
  }
}