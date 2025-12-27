<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { isAuthenticated, authUser } from '$lib/authStore';
	import { messages, decryptedMessages, loadMessages, sendMessage, formatTimestamp, toggleGifs, showGifs, gifResults, gifLoading, searchGifs } from '$lib/chatStore';

	let newMessage = $state('');
	let conversationId = $state('default_global');
	let chatContainer: HTMLElement;
	let gifSearchQuery = $state('');

	onMount(async () => {
		if ($isAuthenticated && conversationId) {
			await loadMessages(conversationId);
		}
	});

	async function handleSendMessage() {
		if (!newMessage.trim()) return;
		await sendMessage(newMessage, conversationId, [], new Uint8Array());
		newMessage = '';
	}

	async function handleSearchGifs() {
		if (gifSearchQuery.trim()) {
			await searchGifs(gifSearchQuery);
		}
	}

	function selectGif(gifUrl: string) {
		console.log('GIF sélectionné:', gifUrl);
		toggleGifs();
	}

	function scrollToBottom() {
		if (chatContainer) {
			chatContainer.scrollTop = chatContainer.scrollHeight;
		}
	}
</script>

<svelte:head>
	<title>Chat - Nook</title>
</svelte:head>

<div class="chat-container">
	<div class="chat-sidebar">
		<div class="sidebar-header">
			<h2>Conversations</h2>
		</div>
		<div class="conversation-list">
			<button class="conversation-item active">
				<span class="conv-avatar">👨‍👩‍👧‍👦</span>
				<div class="conv-info">
					<span class="conv-name">Groupe Global</span>
					<span class="conv-preview">Bienvenue sur Nook !</span>
				</div>
			</button>
		</div>
	</div>

	<div class="chat-main">
		<div class="chat-header">
			<h2>👨‍👩‍👧‍👦 Groupe Global</h2>
		</div>

		<div class="chat-messages" bind:this={chatContainer}>
			{#each $decryptedMessages as message (message.id)}
				<div class="message" class:own={message.isCurrentUser}>
					<div class="message-content">
						<span class="message-sender">{message.senderName}</span>
						<p class="message-text">{message.decryptedContent || message.content}</p>
						<span class="message-time">{formatTimestamp(message.timestamp)}</span>
					</div>
				</div>
			{/each}
		</div>

		{#if $showGifs}
			<div class="gif-panel">
				<div class="gif-search">
					<input type="text" bind:value={gifSearchQuery} placeholder="Rechercher un GIF..." onkeydown={(e) => e.key === 'Enter' && handleSearchGifs()} />
					<button onclick={handleSearchGifs}>🔍</button>
				</div>
				<div class="gif-results">
					{#if $gifLoading}
						<p>Chargement...</p>
					{:else if $gifResults.length > 0}
						{#each $gifResults as gif}
							<button class="gif-item" onclick={() => selectGif(gif.media[0].gif.url)}>
								<img src={gif.media[0].tinygif.url} alt={gif.title} />
							</button>
						{/each}
					{:else}
						<p class="no-results">Recherchez des GIFs pour les envoyer</p>
					{/if}
				</div>
			</div>
		{/if}

		<form class="chat-input" onsubmit={(e) => { e.preventDefault(); handleSendMessage(); }}>
			<button type="button" class="gif-btn" onclick={toggleGifs}>🎬</button>
			<input type="text" bind:value={newMessage} placeholder="Votre message..." />
			<button type="submit" class="send-btn" disabled={!newMessage.trim()}>Envoyer</button>
		</form>
	</div>
</div>

<style>
	.chat-container { display: flex; height: calc(100vh - 140px); gap: 1rem; padding: 1rem; max-width: 1200px; margin: 0 auto; }
	.chat-sidebar { width: 280px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); overflow: hidden; flex-shrink: 0; }
	.sidebar-header { padding: 1rem; background: #2d5a27; color: white; }
	.sidebar-header h2 { font-size: 1rem; margin: 0; }
	.conversation-list { padding: 0.5rem; }
	.conversation-item { width: 100%; display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; border: none; background: none; cursor: pointer; border-radius: 8px; transition: background 0.2s; text-align: left; }
	.conversation-item:hover, .conversation-item.active { background: #f0f7f0; }
	.conv-avatar { font-size: 1.5rem; }
	.conv-info { display: flex; flex-direction: column; }
	.conv-name { font-weight: 500; color: #333; font-size: 0.9rem; }
	.conv-preview { font-size: 0.8rem; color: #888; }
	
	.chat-main { flex: 1; display: flex; flex-direction: column; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); overflow: hidden; }
	.chat-header { padding: 1rem; background: #f8f9fa; border-bottom: 1px solid #eee; }
	.chat-header h2 { font-size: 1rem; margin: 0; color: #333; }
	.chat-messages { flex: 1; padding: 1rem; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem; }
	.message { display: flex; }
	.message.own { justify-content: flex-end; }
	.message-content { max-width: 70%; padding: 0.75rem 1rem; background: #f0f0f0; border-radius: 16px; }
	.message.own .message-content { background: #e8f5e9; }
	.message-sender { font-size: 0.75rem; color: #2d5a27; font-weight: 500; display: block; margin-bottom: 0.25rem; }
	.message-text { margin: 0; color: #333; word-wrap: break-word; }
	.message-time { font-size: 0.7rem; color: #999; display: block; margin-top: 0.25rem; text-align: right; }
	
	.gif-panel { padding: 0.75rem; background: #f8f9fa; border-top: 1px solid #eee; }
	.gif-search { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
	.gif-search input { flex: 1; padding: 0.5rem; border: 1px solid #ddd; border-radius: 8px; }
	.gif-search button { padding: 0.5rem 0.75rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; }
	.gif-results { display: flex; flex-wrap: wrap; gap: 0.5rem; max-height: 150px; overflow-y: auto; }
	.gif-item { border: none; background: none; padding: 0; cursor: pointer; }
	.gif-item img { width: 80px; height: 80px; object-fit: cover; border-radius: 8px; }
	.no-results { color: #888; font-size: 0.9rem; width: 100%; text-align: center; }
	
	.chat-input { display: flex; gap: 0.75rem; padding: 1rem; background: #f8f9fa; border-top: 1px solid #eee; }
	.chat-input input { flex: 1; padding: 0.75rem 1rem; border: 1px solid #ddd; border-radius: 24px; outline: none; }
	.chat-input input:focus { border-color: #2d5a27; }
	.gif-btn, .send-btn { padding: 0.75rem 1rem; border: none; border-radius: 24px; cursor: pointer; font-size: 0.9rem; }
	.gif-btn { background: #f0f0f0; color: #666; }
	.send-btn { background: #2d5a27; color: white; font-weight: 500; }
	.send-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
