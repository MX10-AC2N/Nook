<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { isAuthenticated, isAdmin, authLoading } from '$lib/authStore';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let redirectError = $state<string | null>(null);
	let redirecting = $state(true);

	async function handleRedirect() {
		if (!browser) return;
		
		try {
			redirecting = true;
			redirectError = null;
			await new Promise(resolve => setTimeout(resolve, 300));
			
			if ($authLoading) {
				await new Promise(resolve => setTimeout(resolve, 200));
			}

			if ($isAdmin) {
				goto('/admin');
			} else if ($isAuthenticated) {
				goto('/chat');
			} else {
				goto('/login');
			}
		} catch (err) {
			redirectError = err instanceof Error ? err.message : 'Erreur de redirection';
			redirecting = false;
		}
	}

	onMount(() => {
		handleRedirect();
	});
</script>

<svelte:head>
	<title>Nook - Messagerie familiale ultra privée</title>
	<meta name="description" content="Messagerie familiale ultra privée, auto-hébergée et chiffrée de bout en bout" />
</svelte:head>

<div class="home-container">
	{#if redirecting}
		<div class="redirecting">
			<div class="loader"></div>
			<p>Redirection en cours...</p>
		</div>
	{:else if redirectError}
		<div class="redirect-error">
			<h1>❌ Erreur de redirection</h1>
			<p class="error-detail">{redirectError}</p>
			<div class="error-actions">
				<button onclick={handleRedirect} class="retry-btn">🔄 Réessayer</button>
				<a href="/login" class="login-link">👤 Aller à la connexion</a>
			</div>
		</div>
	{:else}
		<div class="redirecting">
			<div class="loader"></div>
			<p>Redirection en cours...</p>
		</div>
	{/if}
</div>

<style>
	.home-container { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 60vh; text-align: center; }
	.redirecting { display: flex; flex-direction: column; align-items: center; gap: 1.5rem; }
	.redirecting p { color: #666; font-size: 1.1rem; }
	.loader { width: 50px; height: 50px; border: 4px solid #e0e0e0; border-top-color: #2d5a27; border-radius: 50%; animation: spin 1s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
	.redirect-error { max-width: 400px; padding: 2rem; }
	.redirect-error h1 { font-size: 1.5rem; color: #d32f2f; margin-bottom: 1rem; }
	.error-detail { color: #666; margin-bottom: 1.5rem; }
	.error-actions { display: flex; flex-direction: column; gap: 0.75rem; }
	.retry-btn { padding: 0.75rem 1.5rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 1rem; transition: background 0.2s; }
	.retry-btn:hover { background: #3d7a37; }
	.login-link { padding: 0.75rem 1.5rem; background: #f5f5f5; color: #333; text-decoration: none; border-radius: 8px; transition: background 0.2s; }
	.login-link:hover { background: #eeeeee; }
</style>
