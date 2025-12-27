import { goto } from '$app/navigation';
import { browser } from '$app/environment';

export async function checkAuth() {
	try {
		const sessionResponse = await fetch('/api/validate-session', { credentials: 'include' });
		if (sessionResponse.ok) {
			const data = await sessionResponse.json();
			return { status: 'approved', memberId: data.member_id };
		}
		
		const adminResponse = await fetch('/api/admin/validate', { credentials: 'include' });
		if (adminResponse.ok) {
			return { status: 'admin', memberId: null };
		}
		
		if (browser) {
			const urlParams = new URLSearchParams(window.location.search);
			const inviteToken = urlParams.get('token');
			if (inviteToken) {
				sessionStorage.setItem('pending_invite_token', inviteToken);
				return { status: 'guest', memberId: null };
			}
		}
		
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
	if (!response.ok) throw new Error('Login failed');
	return response.json();
}

export async function logout() {
	try {
		await fetch('/api/logout', { method: 'POST', credentials: 'include' });
	} catch (err) {
		console.error('Logout error:', err);
	}
	if (browser) goto('/');
}

export function getPendingInviteToken() {
	if (!browser) return null;
	const token = sessionStorage.getItem('pending_invite_token');
	if (token) sessionStorage.removeItem('pending_invite_token');
	return token;
}

export function setPendingInviteToken(token) {
	if (browser) sessionStorage.setItem('pending_invite_token', token);
}
