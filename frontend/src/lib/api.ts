/// <reference lib="es2020" />
/// <reference lib="dom" />

// src/lib/api.ts

/**
 * Base URL de l'API (relative à la racine du serveur).
 */
const API_BASE = '/api';

/**
 * Interface pour la réponse parsée
 */
interface ParsedResponse {
  ok: boolean;
  status: number;
  data: unknown;
  text: string;
}

/**
 * Interface pour les informations utilisateur
 */
export interface UserInfo {
  id: string;
  username: string;
  name: string;
  role: string;
  approved: boolean;
  needs_password_change: boolean;
}

/**
 * Interface pour la réponse API générique
 */
interface ApiResponse {
  message?: string;
  [key: string]: unknown;
}

/**
 * Interface pour la réponse de validation d'invitation
 */
export interface InviteValidationResponse {
  valid: boolean;
  familyName?: string;
  name?: string;
  expiresAt?: string;
}

/**
 * Interface pour la réponse générique de succès/erreur
 */
export interface OperationResponse {
  success: boolean;
  message: string;
}

/**
 * Parse la réponse HTTP en JSON (si possible) et renvoie un objet
 * contenant `ok`, `status`, `data` (ou `null` si vide) et `text`.
 */
async function parseResponse(response: Response): Promise<ParsedResponse> {
  const text: string = await response.text();

  // Si le corps est vide, on renvoie `null` comme donnée.
  if (!text.trim()) {
    return { ok: response.ok, status: response.status, data: null, text };
  }

  try {
    const data: unknown = JSON.parse(text);
    return { ok: response.ok, status: response.status, data, text };
  } catch (e) {
    // Retourner le texte brut si le JSON est invalide.
    return { ok: false, status: response.status, data: null, text };
  }
}

/**
 * Type guard pour vérifier si un objet est une ApiResponse
 */
function isApiResponse(data: unknown): data is ApiResponse {
  return typeof data === 'object' && data !== null;
}

/**
 * Extrait le message d'une ApiResponse
 */
function extractMessage(data: unknown, fallback: string): string {
  if (isApiResponse(data) && typeof data.message === 'string') {
    return data.message;
  }
  return fallback;
}

/**
 * Change le mot de passe d'un utilisateur (ou du compte courant).
 */
export async function changePassword(
  newPassword: string,
  userId?: string
): Promise<OperationResponse> {
  try {
    const payload: Record<string, unknown> = { new_password: newPassword };
    if (userId) {
      payload.user_id = userId;
    }

    const response: Response = await fetch(`${API_BASE}/auth/change-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(payload),
    });

    const { ok, status, data, text } = await parseResponse(response);

    if (!ok) {
      const msg: string = extractMessage(data, `Erreur ${status}: ${text}`);
      return { success: false, message: msg };
    }

    return { 
      success: true, 
      message: extractMessage(data, 'Mot de passe changé') 
    };
  } catch (err: unknown) {
    console.error('Erreur changement mot de passe:', err);
    return {
      success: false,
      message: err instanceof Error ? err.message : 'Erreur de connexion',
    };
  }
}

/**
 * Type guard pour vérifier si un objet est un UserInfo
 */
function isUserInfo(data: unknown): data is UserInfo {
  return (
    typeof data === 'object' &&
    data !== null &&
    'id' in data &&
    'username' in data &&
    'name' in data &&
    'role' in data &&
    'approved' in data &&
    'needs_password_change' in data &&
    typeof (data as UserInfo).id === 'string' &&
    typeof (data as UserInfo).username === 'string' &&
    typeof (data as UserInfo).name === 'string' &&
    typeof (data as UserInfo).role === 'string' &&
    typeof (data as UserInfo).approved === 'boolean' &&
    typeof (data as UserInfo).needs_password_change === 'boolean'
  );
}

/**
 * Récupère les informations de l'utilisateur connecté.
 */
export async function getUserInfo(): Promise<UserInfo | null> {
  try {
    const response: Response = await fetch(`${API_BASE}/auth/me`, {
      method: 'GET',
      credentials: 'include',
    });

    const { ok, data } = await parseResponse(response);

    if (!ok) {
      console.warn(`getUserInfo: réponse non‑OK (${response.status})`);
      return null;
    }

    // Le backend renvoie `{ user: {...} }` ou `{ authenticated: false }`
    if (isApiResponse(data) && 'user' in data && isUserInfo(data.user)) {
      return data.user;
    }

    return null;
  } catch (err: unknown) {
    console.error('Erreur getUserInfo:', err);
    return null;
  }
}

/**
 * Déconnecte l'utilisateur actuel.
 */
export async function logout(): Promise<void> {
  try {
    await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      credentials: 'include',
    });
  } catch (err: unknown) {
    console.error('Erreur logout:', err);
  }
}

/**
 * Récupère le token d'invitation stocké en sessionStorage (si présent) et le supprime.
 */
export function getPendingInviteToken(): string | null {
  if (typeof window === 'undefined') {
    return null;
  }

  const token: string | null = sessionStorage.getItem('pending_invite_token');
  if (token) {
    sessionStorage.removeItem('pending_invite_token');
  }
  return token;
}

/**
 * Stocke le token d'invitation dans sessionStorage.
 */
export function setPendingInviteToken(token: string): void {
  if (typeof window !== 'undefined') {
    sessionStorage.setItem('pending_invite_token', token);
  }
}

/**
 * Valide un token d'invitation
 */
export async function validateInviteToken(
  token: string
): Promise<InviteValidationResponse> {
  try {
    const response: Response = await fetch(
      `${API_BASE}/invite/validate?token=${encodeURIComponent(token)}`,
      {
        credentials: 'include',
      }
    );
    const { ok, data } = await parseResponse(response);

    if (!ok || !isApiResponse(data)) {
      return { valid: false };
    }

    return {
      valid: ok,
      familyName: typeof data.familyName === 'string' ? data.familyName : undefined,
      name: typeof data.name === 'string' ? data.name : undefined,
      expiresAt: typeof data.expiresAt === 'string' ? data.expiresAt : undefined,
    };
  } catch (err: unknown) {
    console.error('Erreur validateInviteToken:', err);
    return { valid: false };
  }
}

/**
 * Accepte une invitation et crée un compte
 */
export async function acceptInvite(
  token: string,
  username: string,
  name: string,
  password: string
): Promise<OperationResponse> {
  try {
    const response: Response = await fetch(`${API_BASE}/invite/accept`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ token, username, name, password }),
    });

    const { ok, status, data, text } = await parseResponse(response);

    if (!ok) {
      const msg: string = extractMessage(data, `Erreur ${status}: ${text}`);
      return { success: false, message: msg };
    }

    return {
      success: true,
      message: extractMessage(data, 'Compte créé avec succès')
    };
  } catch (err: unknown) {
    console.error('Erreur acceptInvite:', err);
    return {
      success: false,
      message: err instanceof Error ? err.message : 'Erreur de connexion',
    };
  }
}

/**
 * Wrapper fetch générique — préfixe API_BASE, parse JSON, lance une
 * Erreur si status >= 400.
 */
export async function apiFetch<T = unknown>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE}${path}`;
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string> ?? {}),
  };
  if (options.body && typeof options.body === 'string') {
    headers['Content-Type'] ??= 'application/json';
  }
  const res = await fetch(url, { ...options, headers });
  const text = await res.text();
  if (!res.ok) {
    let detail = text;
    try { const j = JSON.parse(text); detail = j.message ?? j.error ?? text; }
    catch { /* body non JSON */ }
    throw new Error(`API ${res.status} ${detail}`);
  }
  if (!text.trim()) return undefined as T;
  return JSON.parse(text) as T;
}