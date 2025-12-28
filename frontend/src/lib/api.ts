// frontend/src/lib/api.ts

const API_BASE = '/api';

export async function changePassword(currentPassword: string, newPassword: string): Promise<{ success: boolean; message: string }> {
  try {
    const response = await fetch(`${API_BASE}/change-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
    });
    
    return await response.json();
  } catch (error) {
    console.error('Erreur changement mot de passe:', error);
    return { 
      success: false, 
      message: error instanceof Error ? error.message : 'Erreur de connexion' 
    };
  }
}

export async function login(memberId: string): Promise<{ success: boolean; user?: object; token?: string }> {
  try {
    const response = await fetch(`${API_BASE}/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({ member_id: memberId })
    });
    
    return await response.json();
  } catch (error) {
    console.error('Erreur login:', error);
    return { 
      success: false, 
      message: error instanceof Error ? error.message : 'Erreur de connexion' 
    };
  }
}

export async function checkAuth(): Promise<{ status: string; memberId?: string }> {
  try {
    const response = await fetch(`${API_BASE}/validate-session`, {
      method: 'GET',
      credentials: 'include'
    });
    
    if (response.ok) {
      const data = await response.json();
      return { status: 'approved', memberId: data.member_id };
    }
    
    return { status: 'guest' };
  } catch (error) {
    console.error('Erreur checkAuth:', error);
    return { status: 'error' };
  }
}

export async function getUserInfo(): Promise<{ id: string; name: string; username: string; role: string } | null> {
  try {
    const response = await fetch(`${API_BASE}/user-info`, {
      method: 'GET',
      credentials: 'include'
    });
    
    if (response.ok) {
      return await response.json();
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
