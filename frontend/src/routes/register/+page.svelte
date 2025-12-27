<script lang="ts">
	import { goto } from '$app/navigation';
	import { getPendingInviteToken } from '$lib/auth';

	let name = $state('');
	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let success = $state(false);
	let loading = $state(false);
	let inviteToken = $state('');

	onMount(() => {
		inviteToken = getPendingInviteToken() || '';
		if (!inviteToken) {
			error = 'Un token d\'invitation est requis pour créer un compte';
		}
	});

	async function handleRegister() {
		if (!name || !username || !password || !confirmPassword) {
			error = 'Veuillez remplir tous les champs';
			return;
		}

		if (password !== confirmPassword) {
			error = 'Les mots de passe ne correspondent pas';
			return;
		}

		if (password.length < 8) {
			error = 'Le mot de passe doit contenir au moins 8 caractères';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await fetch('/api/register', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name, username, password, invite_token: inviteToken })
			});

			const data = await response.json();

			if (response.ok) {
				success = true;
			} else {
				error = data.message || 'Erreur lors de l\'inscription';
			}
		} catch (err) {
			error = 'Erreur de connexion au serveur';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Inscription - Nook</title>
</svelte:head>

<div class="register-container">
	{#if success}
		<div class="success-card">
			<h1>✅ Inscription发送ée</h1>
			<p>Votre demande d'inscription a été envoyée à l'administrateur.</p>
			<p>Vous recevrez une notification une fois votre compte approuvé.</p>
			<a href="/login" class="back-btn">Retour à la connexion</a>
		</div>
	{:else}
		<div class="register-card">
			<h1>🌱 Nook</h1>
			<p class="subtitle">Créer votre compte familial</p>

			{#if error}
				<div class="error-message">{error}</div>
			{/if}

			<form onsubmit={(e) => { e.preventDefault(); handleRegister(); }}>
				<div class="form-group">
					<label for="name">Prénom</label>
					<input type="text" id="name" bind:value={name} placeholder="Votre prénom" disabled={loading || !inviteToken} />
				</div>

				<div class="form-group">
					<label for="username">Identifiant</label>
					<input type="text" id="username" bind:value={username} placeholder="Identifiant unique" disabled={loading || !inviteToken} />
				</div>

				<div class="form-group">
					<label for="password">Mot de passe</label>
					<input type="password" id="password" bind:value={password} placeholder="Au moins 8 caractères" disabled={loading || !inviteToken} />
				</div>

				<div class="form-group">
					<label for="confirmPassword">Confirmer le mot de passe</label>
					<input type="password" id="confirmPassword" bind:value={confirmPassword} placeholder="Répétez le mot de passe" disabled={loading || !inviteToken} />
				</div>

				<button type="submit" class="register-btn" disabled={loading || !inviteToken}>
					{loading ? 'Inscription...' : 'S\'inscrire'}
				</button>
			</form>

			<div class="links">
				<a href="/login">Déjà un compte ? Se connecter</a>
			</div>
		</div>
	{/if}
</div>

<style>
	.register-container { display: flex; align-items: center; justify-content: center; min-height: calc(100vh - 100px); padding: 1rem; }
	.register-card, .success-card { background: white; padding: 2rem; border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); width: 100%; max-width: 400px; text-align: center; }
	.success-card h1 { color: #2d5a27; margin-bottom: 1rem; }
	h1 { font-size: 2rem; margin-bottom: 0.5rem; }
	.subtitle { color: #666; margin-bottom: 2rem; }
	.form-group { margin-bottom: 1.25rem; text-align: left; }
	label { display: block; margin-bottom: 0.5rem; font-weight: 500; color: #333; }
	input { width: 100%; padding: 0.75rem; border: 2px solid #e0e0e0; border-radius: 8px; font-size: 1rem; }
	input:focus { outline: none; border-color: #2d5a27; }
	.error-message { background: #ffebee; color: #c62828; padding: 0.75rem; border-radius: 8px; margin-bottom: 1rem; font-size: 0.9rem; }
	.register-btn { width: 100%; padding: 0.875rem; background: #2d5a27; color: white; border: none; border-radius: 8px; font-size: 1rem; font-weight: 500; cursor: pointer; }
	.register-btn:hover:not(:disabled) { background: #3d7a37; }
	.register-btn:disabled { opacity: 0.6; cursor: not-allowed; }
	.back-btn { display: inline-block; margin-top: 1.5rem; padding: 0.75rem 1.5rem; background: #2d5a27; color: white; text-decoration: none; border-radius: 8px; }
	.links { margin-top: 1.5rem; }
	.links a { color: #2d5a27; text-decoration: none; font-size: 0.9rem; }
</style>
