<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import Avatar from './Avatar.svelte';

  interface SearchResult {
    id: string;
    conversation_id: string;
    sender_id: string;
    sender_name: string;
    sender_avatar_style?: string;
    sender_avatar_seed?: string;
    content: string;
    message_type: string;
    timestamp: number;
    created_at: number;
  }

  let searchQuery = $state('');
  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let showResults = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  // Formater le timestamp
  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
  }

  // Formater la date
  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) {
      return 'Aujourd\'hui';
    } else if (date.toDateString() === yesterday.toDateString()) {
      return 'Hier';
    } else {
      return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' });
    }
  }

  // Recherche avec debounce
  async function performSearch() {
    if (!searchQuery.trim() || searchQuery.trim().length < 2) {
      searchResults = [];
      showResults = false;
      return;
    }

    isSearching = true;
    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(searchQuery.trim())}`, {
        credentials: 'include'
      });
      
      if (res.ok) {
        searchResults = await res.json();
        showResults = searchResults.length > 0;
      }
    } catch (e) {
      console.error('Erreur recherche:', e);
    } finally {
      isSearching = false;
    }
  }

  // Debounce la recherche
  function handleInput() {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    
    searchTimeout = setTimeout(() => {
      performSearch();
    }, 300);
  }

  // Naviguer vers le message
  function goToMessage(result: SearchResult) {
    goto(`/chat?conv=${result.conversation_id}&msg=${result.id}`);
    showResults = false;
    searchQuery = '';
  }

  // Fermer les résultats quand on clique ailleurs
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.search-container')) {
      showResults = false;
    }
  }

  onMount(() => {
    if (browser) {
      document.addEventListener('click', handleClickOutside);
      return () => {
        document.removeEventListener('click', handleClickOutside);
        if (searchTimeout) {
          clearTimeout(searchTimeout);
        }
      };
    }
  });
</script>

<div class="search-container">
  <div class="search-input-wrapper">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    
    <input
      type="text"
      class="search-input"
      placeholder="Rechercher des messages..."
      bind:value={searchQuery}
      oninput={handleInput}
      onfocus={() => { if (searchResults.length > 0) showResults = true; }}
    />
    
    {#if isSearching}
      <div class="search-spinner"></div>
    {:else if searchQuery}
      <button 
        class="search-clear" 
        onclick={() => { searchQuery = ''; searchResults = []; showResults = false; }}
        aria-label="Effacer la recherche"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    {/if}
  </div>

  {#if showResults}
    <div class="search-results">
      <div class="results-header">
        <span class="results-count">{searchResults.length} résultat{searchResults.length > 1 ? 's' : ''}</span>
      </div>
      
      <div class="results-list">
        {#each searchResults as result (result.id)}
          <button 
            class="result-item" 
            onclick={() => goToMessage(result)}
          >
            <div class="result-avatar">
              <Avatar 
                style={result.sender_avatar_style} 
                seed={result.sender_avatar_seed} 
                name={result.sender_name}
                size={32}
              />
            </div>
            
            <div class="result-content">
              <div class="result-header">
                <span class="result-sender">{result.sender_name}</span>
                <span class="result-date">{formatDate(result.created_at)} à {formatTime(result.created_at)}</span>
              </div>
              <div class="result-text">
                {#if result.message_type === 'image'}
                  🖼️ Image
                {:else if result.message_type === 'file'}
                  📄 Fichier
                {:else if result.message_type === 'audio'}
                  🎵 Audio
                {:else if result.message_type === 'video'}
                  🎬 Vidéo
                {:else}
                  {result.content.length > 100 ? result.content.slice(0, 100) + '...' : result.content}
                {/if}
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {:else if searchQuery && searchQuery.trim().length >= 2 && !isSearching && searchResults.length === 0}
    <div class="no-results">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <span>Aucun résultat pour "{searchQuery}"</span>
    </div>
  {/if}
</div>

<style>
  .search-container {
    position: relative;
    margin-bottom: 0.75rem;
  }
  
  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }
  
  .search-icon {
    position: absolute;
    left: 0.75rem;
    width: 1rem;
    height: 1rem;
    color: var(--text-muted, #94a3b8);
    pointer-events: none;
  }
  
  .search-input {
    width: 100%;
    padding: 0.6rem 2.5rem 0.6rem 2.5rem;
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 0.5rem;
    font-size: 0.9rem;
    background: var(--bg-primary, #fff);
    color: var(--text-primary, #1e293b);
    transition: all 0.15s ease;
  }
  
  .search-input:focus {
    outline: none;
    border-color: var(--primary, #6366f1);
    box-shadow: 0 0 0 2px var(--primary-light, #eef2ff);
  }
  .search-input:focus-visible {
    outline: 2px solid #4f9cf9;
    outline-offset: 2px;
  }
  
  .search-input::placeholder {
    color: var(--text-muted, #94a3b8);
  }
  
  .search-spinner {
    position: absolute;
    right: 0.75rem;
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--border, #e2e8f0);
    border-top-color: var(--primary, #6366f1);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .search-clear {
    position: absolute;
    right: 0.5rem;
    width: 1.5rem;
    height: 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
    border-radius: 50%;
    transition: all 0.15s ease;
  }
  
  .search-clear:hover {
    background: var(--bg-hover, #f1f5f9);
    color: var(--text-primary, #1e293b);
  }
  
  .search-clear svg {
    width: 0.75rem;
    height: 0.75rem;
  }
  
  .search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 0.25rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 0.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    z-index: 100;
    max-height: 400px;
    overflow-y: auto;
  }
  
  .results-header {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
  }
  
  .results-count {
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
    font-weight: 500;
  }
  
  .results-list {
    display: flex;
    flex-direction: column;
  }
  
  .result-item {
    display: flex;
    gap: 0.75rem;
    padding: 0.75rem;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border, #e2e8f0);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.15s ease;
  }
  
  .result-item:last-child {
    border-bottom: none;
  }
  
  .result-item:hover {
    background: var(--bg-hover, #f1f5f9);
  }
  
  .result-avatar {
    flex-shrink: 0;
  }
  
  .result-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }
  
  .result-sender {
    font-weight: 500;
    color: var(--text-primary, #1e293b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .result-date {
    font-size: 0.7rem;
    color: var(--text-muted, #94a3b8);
    white-space: nowrap;
  }
  
  .result-text {
    font-size: 0.85rem;
    color: var(--text-secondary, #64748b);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .no-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 0.25rem;
    padding: 1rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 0.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    z-index: 100;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-muted, #94a3b8);
    font-size: 0.9rem;
  }
  
  .no-results svg {
    width: 1.5rem;
    height: 1.5rem;
    opacity: 0.5;
  }
</style>
