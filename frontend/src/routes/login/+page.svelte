<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { isAuthenticated } from '$lib/authStore';
	import { onMount } from 'svelte';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleLogin() {
		if (!username || !password) {
			error = 'Veuillez remplir tous les champs';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await fetch('/api/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ username, password })
			});

			if (response.ok) {
				goto('/chat');
			} else {
				const data = await response.json();
				error = data.message || 'Identifiants incorrects';
			}
		} catch (err) {
			error = 'Erreur de connexion au serveur';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		if ($isAuthenticated) goto('/chat');
	});
</script>

<svelte:head>
	<title>Connexion - Nook</title>
</svelte:head>

<div class="login-container">
	<div class="login-card">
		<h1>🌱 Nook</h1>
		<p class="subtitle">Connexion à votre espace familial</p>

		<form onsubmit={(e) => { e.preventDefault(); handleLogin(); }}>
			<div class="form-group">
				<label for="username">Identifiant</label>
				<input type="text" id="username" bind:value={username} placeholder="Votre identifiant" disabled={loading} />
			</div>

			<div class="form-group">
				<label for="password">Mot de passe</label>
				<input type="password" id="password" bind:value={password} placeholder="Votre mot de passe" disabled={loading} />
			</div>

			{#if error}
				<div class="error-message">{error}</div>
			{/if}

			<button type="submit" class="login-btn" disabled={loading}>
				{loading ? 'Connexion...' : 'Se connecter'}
			</button>
		</form>

		<div class="links">
			<a href="/register">Créer un compte</a>
			<span class="separator">•</span>
			<a href="/help">Aide</a>
		</div>
	</div>
</div>

<style>
	.login-container { display: flex; align-items: center; justify-content: center; min-height: calc(100vh - 100px); padding: 1rem; }
	.login-card { background: white; padding: 2rem; border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); width: 100%; max-width: 400px; text-align: center; }
	h1 { font-size: 2rem; margin-bottom: 0.5rem; }
	.subtitle { color: #666; margin-bottom: 2rem; }
	.form-group { margin-bottom: 1.25rem; text-align: left; }
	label { display: block; margin-bottom: 0.5rem; font-weight: 500; color: #333; }
	input { width: 100%; padding: 0.75rem; border: 2px solid #e0e0e0; border-radius: 8px; font-size: 1rem; transition: border-color 0.2s; }
	input:focus { outline: none; border-color: #2d5a27; }
	.error-message { background: #ffebee; color: #c62828; padding: 0.75rem; border-radius: 8px; margin-bottom: 1rem; font-size: 0.9rem; }
	.login-btn { width: 100%; padding: 0.875rem; background: #2d5a27; color: white; border: none; border-radius: 8px; font-size: 1rem; font-weight: 500; cursor: pointer; transition: background 0.2s; }
	.login-btn:hover:not(:disabled) { background: #3d7a37; }
	.login-btn:disabled { opacity: 0.6; cursor: not-allowed; }
	.links { margin-top: 1.5rem; display: flex; justify-content: center; gap: 0.5rem; font-size: 0.9rem; }
	.links a { color: #2d5a27; text-decoration: none; }
	.links a:hover { text-decoration: underline; }
	.separator { color: #ccc; }
</style>
