<script lang="ts">
	import { page } from '$app/stores';
	import { authStore, isAuthenticated, isAdmin, authLoading } from '$lib/authStore';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';

	let { children } = $props();
	
	let showMenu = $state(false);
	let appError = $state<string | null>(null);
	let loading = $state(true);

	const navItems = [
		{ path: '/chat', label: '💬 Chat', requiresAuth: true },
		{ path: '/calendar', label: '📅 Calendrier', requiresAuth: true },
		{ path: '/admin', label: '👑 Administration', requiresAuth: true, requiresAdmin: true },
		{ path: '/settings', label: '⚙️ Paramètres', requiresAuth: true },
		{ path: '/help', label: '❓ Aide', requiresAuth: false }
	];

	function toggleMenu() {
		showMenu = !showMenu;
	}

	async function handleLogout() {
		await fetch('/api/logout', { method: 'POST', credentials: 'include' });
		showMenu = false;
		goto('/login');
	}

	$effect(() => {
		if (!loading && !isAuthenticated && $page.url.pathname !== '/login' && !$page.url.pathname.startsWith('/join')) {
			goto('/login');
		}
	});

	onMount(async () => {
		setTimeout(() => {
			loading = false;
		}, 500);
	});
</script>

<svelte:head>
	<title>Nook - Messagerie familiale</title>
</svelte:head>

{#if loading}
	<div class="loading-screen">
		<div class="loader"></div>
		<p>Chargement de Nook...</p>
	</div>
{:else if appError}
	<div class="error-screen">
		<h1>❌ Erreur système</h1>
		<p>{appError}</p>
		<button onclick={() => window.location.reload()} class="retry-button">
			🔄 Recharger l'application
		</button>
	</div>
{:else}
	<div class="app-container">
		<header class="app-header">
			<button class="menu-toggle" onclick={toggleMenu} aria-label="Menu">☰</button>
			<h1 class="app-title">🌱 Nook</h1>
			{#if $isAuthenticated}
				<div class="user-info">
					<span class="user-name">{$authStore.user?.name}</span>
					<button class="logout-btn" onclick={handleLogout} aria-label="Déconnexion">🔌</button>
				</div>
			{/if}
		</header>

		{#if showMenu}
			<aside class="side-menu" class:open={showMenu}>
				<div class="menu-header">
					<h2>Menu Nook</h2>
					<button class="close-menu" onclick={toggleMenu} aria-label="Fermer">✕</button>
				</div>
				<nav class="menu-nav">
					{#each navItems as item}
						{#if item.requiresAuth && !$isAuthenticated}
						{:else if item.requiresAdmin && !$isAdmin}
						{:else}
							<a 
								href={item.path} 
								class="menu-item"
								class:active={$page.url.pathname.startsWith(item.path)}
								onclick={toggleMenu}
							>
								{item.label}
							</a>
						{/if}
					{/each}
				</nav>
				<div class="menu-footer">
					<p class="version">Version 3.0 • SvelteKit</p>
					<button class="logout-full" onclick={handleLogout}>🔌 Déconnexion</button>
				</div>
			</aside>
			{#if showMenu}
				<button class="menu-backdrop" onclick={toggleMenu} aria-label="Fermer le menu"></button>
			{/if}
		{/if}

		<main class="app-content">
			{@render children()}
		</main>

		<footer class="app-footer">
			<p>© {new Date().getFullYear()} Nook • Messagerie privée pour la famille</p>
		</footer>
	</div>
{/if}

<style>
	:global(*) { box-sizing: border-box; margin: 0; padding: 0; }
	:global(body) { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f0; color: #333; min-height: 100vh; }
	
	.loading-screen, .error-screen { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; gap: 1rem; }
	.loader { width: 40px; height: 40px; border: 4px solid #ddd; border-top-color: #2d5a27; border-radius: 50%; animation: spin 1s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
	.retry-button { padding: 0.75rem 1.5rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 1rem; }
	
	.app-container { display: flex; flex-direction: column; min-height: 100vh; }
	.app-header { display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1rem; background: linear-gradient(135deg, #2d5a27, #4a7c43); color: white; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
	.menu-toggle { background: none; border: none; font-size: 1.5rem; cursor: pointer; padding: 0.5rem; color: white; }
	.app-title { font-size: 1.25rem; font-weight: 600; }
	.user-info { display: flex; align-items: center; gap: 0.5rem; }
	.user-name { font-size: 0.9rem; opacity: 0.9; }
	.logout-btn { background: none; border: none; font-size: 1.2rem; cursor: pointer; opacity: 0.8; transition: opacity 0.2s; }
	.logout-btn:hover { opacity: 1; }
	
	.side-menu { position: fixed; top: 0; right: -280px; width: 280px; height: 100vh; background: white; box-shadow: -4px 0 20px rgba(0,0,0,0.15); z-index: 1000; transition: right 0.3s ease; display: flex; flex-direction: column; }
	.side-menu.open { right: 0; }
	.menu-header { display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #eee; }
	.menu-header h2 { font-size: 1.1rem; color: #2d5a27; }
	.close-menu { background: none; border: none; font-size: 1.2rem; cursor: pointer; color: #666; }
	.menu-nav { flex: 1; padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
	.menu-item { padding: 0.75rem 1rem; text-decoration: none; color: #333; border-radius: 8px; transition: all 0.2s; }
	.menu-item:hover { background: #f0f7f0; }
	.menu-item.active { background: #e8f5e9; color: #2d5a27; font-weight: 500; }
	.menu-footer { padding: 1rem; border-top: 1px solid #eee; }
	.version { font-size: 0.8rem; color: #999; margin-bottom: 0.75rem; }
	.logout-full { width: 100%; padding: 0.75rem; background: #f5f5f5; border: none; border-radius: 8px; cursor: pointer; color: #666; }
	.menu-backdrop { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.3); z-index: 999; border: none; cursor: pointer; }
	
	.app-content { flex: 1; padding: 1rem; max-width: 800px; margin: 0 auto; width: 100%; }
	.app-footer { padding: 1rem; text-align: center; color: #888; font-size: 0.85rem; }
</style>
