// /frontend/src/lib/auth.js
import { goto } from '$app/navigation';

export async function checkAuth() {
  try {
    // 1. Vérifier si on a un cookie de session utilisateur
    const sessionResponse = await fetch('/api/validate-session', {
      credentials: 'include'
    });

    if (sessionResponse.ok) {
      const data = await sessionResponse.json();
      return { status: 'approved', memberId: data.member_id };
    }

    // 2. Vérifier si on a un cookie de session admin
    const adminResponse = await fetch('/api/admin/validate', {
      credentials: 'include'
    });
    if (adminResponse.ok) {
      return { status: 'admin', memberId: null };
    }

    // 3. Vérifier si on a un token d'invitation dans l'URL
    const urlParams = new URLSearchParams(window.location.search);
    const inviteToken = urlParams.get('token');

    if (inviteToken) {
      sessionStorage.setItem('pending_invite_token', inviteToken);
      return { status: 'guest', memberId: null };
    }

    // 4. Par défaut
    return { status: 'guest', memberId: null };

  } catch (error) {
    console.error('Auth check failed:', error);
    throw error;
  }
}

export async function login(memberId) {
  const response = await fetch('/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ member_id: memberId }),
    credentials: 'include'
  });

  if (!response.ok) {
    throw new Error('Login failed');
  }
  return response.json();
}

export async function logout() {
  await fetch('/api/logout', {
    method: 'POST',
    credentials: 'include'
  });
  goto('/');
}