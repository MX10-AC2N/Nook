// frontend/src/lib/api.ts

const API_BASE = '/api';

export async function changePassword(newPassword: string, userId?: string): Promise<{ success: boolean; message: string }> {
  try {
    // Pour changement normal (utilisateur connecté) ou first-setup (admin initial)
    // Le backend gère les deux cas (avec ou sans current_password, selon la route)
    const payload: any = { new_password: newPassword };
    if (userId) payload.user_id = userId;  // Pour first-setup

    const endpoint = userId ? '/first-setup' : '/change-password';

    const response = await fetch(`\( {API_BASE} \){endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(payload)
    });

    const data = await response.json();
    return { success: response.ok, message: data.message || 'Succès' };
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
    const response = await fetch(`${API_BASE}/user-info`, {
      method: 'GET',
      credentials: 'include'
    });

    if (response.ok) {
      const data = await response.json();
      return data.user || null;  // Le backend retourne { user: {...} }
    }

    return null;
  } catch (error) {
    console.error('Erreur getUserInfo:', error);
    return null;
  }
}

export async function createJoinRequest(token: string, name: string, publicKey: string): Promise<{ success: boolean; message: string }> {
  try {
    const response = await fetch(`\( {API_BASE}/join?token= \){encodeURIComponent(token)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ name, public_key: publicKey })
    });

    const data = await response.json();
    return { success: response.ok, message: data.message || response.statusText };
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
    await fetch(`${API_BASE}/logout`, {
      method: 'POST',
      credentials: 'include'
    });
  } catch (error) {
    console.error('Erreur logout:', error);
  }
}