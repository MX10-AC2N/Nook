// frontend/src/lib/api.ts

const API_BASE = '/api';

export async function changePassword(newPassword: string, userId?: string): Promise<{ success: boolean; message: string }> {
  try {
    // Pour changement normal (utilisateur connecté) ou first-setup (admin initial)
    const payload: any = { new_password: newPassword };
    if (userId) payload.user_id = userId;

    const endpoint = userId ? '/first-setup' : '/change-password';

    const response = await fetch(`${API_BASE}${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(payload)
    });

    // Essayer de parser le JSON, même si la réponse est vide
    const text = await response.text();
    
    if (!text.trim()) {
      return { success: false, message: `Erreur ${response.status}: Réponse vide du serveur` };
    }

    try {
      const data = JSON.parse(text);
      return { success: response.ok, message: data.message || 'Succès' };
    } catch {
      return { success: false, message: `Erreur ${response.status}: ${text}` };
    }
  } catch (error) {
    console.error('Erreur changement mot de passe:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Erreur de connexion'
    };
  }
}

export async function getUserInfo(): Promise<{
  id: string;
  username: string;
  name: string;
  role: string;
  approved: boolean;
  needs_password_change: boolean
} | null> {
  try {
    const response = await fetch(`${API_BASE}/auth/me`, {
      method: 'GET',
      credentials: 'include'
    });

    if (response.ok) {
      const text = await response.text();
      if (!text.trim()) return null;
      const data = JSON.parse(text);
      return data.user || null;
    }
    return null;
  } catch (error) {
    console.error('Erreur getUserInfo:', error);
    return null;
  }
}

export async function createJoinRequest(token: string, name: string, publicKey: string): Promise<{ success: boolean; message: string }> {
  try {
    const response = await fetch(`${API_BASE}/join?token=${encodeURIComponent(token)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ name, public_key: publicKey })
    });

    const text = await response.text();
    
    if (!text.trim()) {
      return { success: false, message: `Erreur ${response.status}: Réponse vide` };
    }

    try {
      const data = JSON.parse(text);
      return { success: response.ok, message: data.message || response.statusText };
    } catch {
      return { success: false, message: `Erreur ${response.status}: ${text}` };
    }
  } catch (error) {
    console.error('Erreur createJoinRequest:', error);
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Erreur de connexion'
    };
  }
}

export async function logout(): Promise<void> {
  try {
    await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      credentials: 'include'
    });
  } catch (error) {
    console.error('Erreur logout:', error);
  }
}

export function getPendingInviteToken() {
  if (typeof window === 'undefined') return null;
  const token = sessionStorage.getItem('pending_invite_token');
  if (token) sessionStorage.removeItem('pending_invite_token');
  return token;
}

export function setPendingInviteToken(token: string) {
  if (typeof window !== 'undefined') sessionStorage.setItem('pending_invite_token', token);
}
