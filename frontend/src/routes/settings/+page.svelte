<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated, authUser, updateUser } from '$lib/authStore';

	let userName = $state('');
	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let message = $state('');
	let error = $state('');
	let saving = $state(false);
	let activeTab = $state<'profile' | 'security' | 'appearance'>('profile');

	const themes = [
		{ id: 'jardin-secret', name: '🌿 Jardin Secret', description: 'Doux, naturel, aquarelle' },
		{ id: 'space-hub', name: '🚀 Space Hub', description: 'Futuriste, néon, épuré' },
		{ id: 'maison-chaleureuse', name: '🏠 Maison Chaleureuse', description: 'Feutre, crayon, bois' }
	];
	let currentTheme = $state('jardin-secret');
	let darkMode = $state(false);

	onMount(async () => {
		if (!$isAuthenticated) {
			goto('/login');
			return;
		}
		if (authUser) {
			userName = authUser.name || '';
		}
		loadTheme();
	});

	function loadTheme() {
		if (typeof window !== 'undefined') {
			const saved = localStorage.getItem('nook-theme') || 'jardin-secret';
			currentTheme = saved;
			darkMode = localStorage.getItem('nook-dark-mode') === 'true';
			applyTheme();
		}
	}

	function applyTheme() {
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', currentTheme);
			document.documentElement.setAttribute('data-dark', darkMode.toString());
			localStorage.setItem('nook-theme', currentTheme);
			localStorage.setItem('nook-dark-mode', darkMode.toString());
		}
	}

	async function updateProfile() {
		if (!userName.trim()) {
			error = 'Le nom ne peut pas être vide';
			return;
		}

		saving = true;
		error = '';
		message = '';

		try {
			const response = await fetch('/api/user/update', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ name: userName })
			});

			if (response.ok) {
				updateUser({ name: userName });
				message = 'Profil mis à jour avec succès';
			} else {
				const data = await response.json();
				error = data.message || 'Erreur lors de la mise à jour';
			}
		} catch (err) {
			error = 'Erreur de connexion';
		} finally {
			saving = false;
		}
	}

	async function changePassword() {
		if (!currentPassword || !newPassword || !confirmPassword) {
			error = 'Veuillez remplir tous les champs';
			return;
		}

		if (newPassword.length < 8) {
			error = 'Le nouveau mot de passe doit contenir au moins 8 caractères';
			return;
		}

		if (newPassword !== confirmPassword) {
			error = 'Les mots de passe ne correspondent pas';
			return;
		}

		saving = true;
		error = '';
		message = '';

		try {
			const response = await fetch('/api/change-password', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
			});

			if (response.ok) {
				message = 'Mot de passe modifié avec succès';
				currentPassword = '';
				newPassword = '';
				confirmPassword = '';
			} else {
				const data = await response.json();
				error = data.message || 'Erreur lors du changement de mot de passe';
			}
		} catch (err) {
			error = 'Erreur de connexion';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Paramètres - Nook</title>
</svelte:head>

<div class="settings-container">
	<h1>⚙️ Paramètres</h1>

	<div class="settings-tabs">
		<button class="tab" class:active={activeTab === 'profile'} onclick={() => activeTab = 'profile'}>Profil</button>
		<button class="tab" class:active={activeTab === 'security'} onclick={() => activeTab = 'security'}>Sécurité</button>
		<button class="tab" class:active={activeTab === 'appearance'} onclick={() => activeTab = 'appearance'}>Apparence</button>
	</div>

	<div class="settings-content">
		{#if activeTab === 'profile'}
			<div class="settings-section">
				<h2>Informations du profil</h2>
				<form onsubmit={(e) => { e.preventDefault(); updateProfile(); }}>
					<div class="form-group">
						<label for="name">Prénom</label>
						<input type="text" id="name" bind:value={userName} placeholder="Votre prénom" />
					</div>
					<div class="form-group">
						<label>Identifiant</label>
						<input type="text" value={authUser?.username || ''} disabled />
						<span class="hint">L'identifiant ne peut pas être modifié</span>
					</div>
					<button type="submit" class="save-btn" disabled={saving}>
						{saving ? 'Enregistrement...' : 'Enregistrer'}
					</button>
				</form>
			</div>
		{:else if activeTab === 'security'}
			<div class="settings-section">
				<h2>Changer le mot de passe</h2>
				<form onsubmit={(e) => { e.preventDefault(); changePassword(); }}>
					<div class="form-group">
						<label for="currentPassword">Mot de passe actuel</label>
						<input type="password" id="currentPassword" bind:value={currentPassword} />
					</div>
					<div class="form-group">
						<label for="newPassword">Nouveau mot de passe</label>
						<input type="password" id="newPassword" bind:value={newPassword} />
						<span class="hint">Au moins 8 caractères</span>
					</div>
					<div class="form-group">
						<label for="confirmPassword">Confirmer le nouveau mot de passe</label>
						<input type="password" id="confirmPassword" bind:value={confirmPassword} />
					</div>
					<button type="submit" class="save-btn" disabled={saving}>
						{saving ? 'Modification...' : 'Changer le mot de passe'}
					</button>
				</form>
			</div>
		{:else if activeTab === 'appearance'}
			<div class="settings-section">
				<h2>Thème</h2>
				<div class="theme-grid">
					{#each themes as theme}
						<button 
							class="theme-card" 
							class:selected={currentTheme === theme.id}
							onclick={() => { currentTheme = theme.id; applyTheme(); }}
						>
							<span class="theme-icon">{theme.id === 'jardin-secret' ? '🌿' : theme.id === 'space-hub' ? '🚀' : '🏠'}</span>
							<span class="theme-name">{theme.name}</span>
							<span class="theme-desc">{theme.description}</span>
						</button>
					{/each}
				</div>

				<div class="theme-option">
					<label class="toggle-label">
						<input type="checkbox" checked={darkMode} onchange={(e) => { darkMode = e.currentTarget.checked; applyTheme(); }} />
						<span>Mode sombre</span>
					</label>
				</div>
			</div>
		{/if}

		{#if message}
			<div class="success-message">{message}</div>
		{/if}
		{#if error}
			<div class="error-message">{error}</div>
		{/if}
	</div>
</div>

<style>
	.settings-container { max-width: 600px; margin: 0 auto; padding: 1rem; }
	h1 { font-size: 1.5rem; color: #2d5a27; margin-bottom: 1.5rem; }
	
	.settings-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; border-bottom: 2px solid #eee; padding-bottom: 0.5rem; }
	.tab { padding: 0.75rem 1.25rem; background: none; border: none; cursor: pointer; font-size: 0.95rem; color: #666; border-radius: 8px 8px 0 0; transition: all 0.2s; }
	.tab:hover { background: #f0f7f0; }
	.tab.active { background: #2d5a27; color: white; }
	
	.settings-content { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); padding: 1.5rem; }
	.settings-section h2 { font-size: 1.1rem; color: #333; margin-bottom: 1rem; }
	
	.form-group { margin-bottom: 1rem; }
	label { display: block; font-size: 0.85rem; color: #666; margin-bottom: 0.25rem; }
	input { width: 100%; padding: 0.75rem; border: 1px solid #ddd; border-radius: 8px; font-size: 0.95rem; }
	input:focus { outline: none; border-color: #2d5a27; }
	input:disabled { background: #f5f5f5; color: #888; }
	.hint { font-size: 0.75rem; color: #888; margin-top: 0.25rem; display: block; }
	
	.save-btn { width: 100%; padding: 0.875rem; background: #2d5a27; color: white; border: none; border-radius: 8px; font-size: 0.95rem; cursor: pointer; transition: background 0.2s; }
	.save-btn:hover:not(:disabled) { background: #3d7a37; }
	.save-btn:disabled { opacity: 0.6; cursor: not-allowed; }
	
	.theme-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; }
	.theme-card { padding: 1rem; background: #f8f9fa; border: 2px solid transparent; border-radius: 12px; cursor: pointer; text-align: center; transition: all 0.2s; }
	.theme-card:hover { background: #f0f7f0; }
	.theme-card.selected { border-color: #2d5a27; background: #e8f5e9; }
	.theme-icon { font-size: 2rem; display: block; margin-bottom: 0.5rem; }
	.theme-name { font-weight: 500; color: #333; display: block; }
	.theme-desc { font-size: 0.75rem; color: #888; }
	
	.theme-option { padding-top: 1rem; border-top: 1px solid #eee; }
	.toggle-label { display: flex; align-items: center; gap: 0.75rem; cursor: pointer; }
	.toggle-label input { width: auto; }
	
	.success-message { margin-top: 1rem; padding: 0.75rem; background: #e8f5e9; color: #2d5a27; border-radius: 8px; }
	.error-message { margin-top: 1rem; padding: 0.75rem; background: #ffebee; color: #c62828; border-radius: 8px; }
</style>
