<!-- frontend/src/components/MediaPlayer.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, get } from 'svelte/store';
  import { downloadAndDecryptMedia } from '$lib/mediaStore';
  import { connectionError } from '$lib/chatStore';
  import { formatDuration } from '$lib/mediaStore';
  
  export let message: any;
  export let isCurrentUser: boolean = false;
  
  let audioElement: HTMLAudioElement | HTMLVideoElement | null = null;
  let isPlaying = false;
  let currentTime = 0;
  let duration = 0;
  let progress = 0;
  let isLoaded = false;
  let isLoading = false;
  let error = null;
  let previewUrl: string | null = null;
  let decryptedBlob: Blob | null = null;
  
  const playbackRateOptions = [0.5, 1, 1.5, 2];
  let currentPlaybackRate = 1;
  
  onMount(async () => {
    if (message.media_url && !isLoaded) {
      await loadMedia();
    }
  });
  
  onDestroy(() => {
    cleanupMedia();
  });
  
  async function loadMedia() {
    if (isLoading || isLoaded) return;
    
    isLoading = true;
    error = null;
    
    try {
      // Télécharger et déchiffrer le média
      decryptedBlob = await downloadAndDecryptMedia(
        message.media_url,
        message.encrypted_keys,
        message.nonce,
        message.sender_id
      );
      
      // Créer une URL objet pour la prévisualisation
      previewUrl = URL.createObjectURL(decryptedBlob);
      
      // Initialiser l'élément audio/vidéo
      if (message.media_type === 'audio') {
        audioElement = new Audio(previewUrl);
      } else {
        // Pour la vidéo, nous allons utiliser un élément <video> dans le template
        duration = message.duration || 0;
      }
      
      if (audioElement) {
        audioElement.onloadedmetadata = () => {
          duration = audioElement?.duration || message.duration || 0;
          isLoaded = true;
          isLoading = false;
        };
        
        audioElement.ontimeupdate = () => {
          currentTime = audioElement?.currentTime || 0;
          progress = duration ? (currentTime / duration) * 100 : 0;
        };
        
        audioElement.onended = () => {
          isPlaying = false;
          currentTime = 0;
          progress = 0;
        };
        
        audioElement.onerror = () => {
          error = 'Erreur lors de la lecture du média';
          isLoading = false;
          isLoaded = false;
        };
      } else {
        isLoaded = true;
        isLoading = false;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur lors du chargement du média';
      console.error('Erreur chargement média:', err);
      isLoading = false;
    }
  }
  
  function cleanupMedia() {
    if (audioElement) {
      audioElement.pause();
      audioElement.onloadedmetadata = null;
      audioElement.ontimeupdate = null;
      audioElement.onended = null;
      audioElement.onerror = null;
      audioElement = null;
    }
    
    if (previewUrl) {
      URL.revokeObjectURL(previewUrl);
      previewUrl = null;
    }
    
    if (decryptedBlob) {
      decryptedBlob = null;
    }
  }
  
  function togglePlay() {
    if (!isLoaded && !isLoading) {
      loadMedia();
      return;
    }
    
    if (!audioElement || isLoading) return;
    
    if (isPlaying) {
      audioElement.pause();
      isPlaying = false;
    } else {
      audioElement.play().catch(err => {
        console.error('Erreur lecture audio:', err);
        connectionError.set('Impossible de lire le média');
      });
      isPlaying = true;
    }
  }
  
  function handleTimeUpdate(e: Event) {
    const input = e.target as HTMLInputElement;
    const time = parseFloat(input.value);
    currentTime = time;
    progress = (time / duration) * 100;
    
    if (audioElement) {
      audioElement.currentTime = time;
    }
  }
  
  function changePlaybackRate() {
    const currentIndex = playbackRateOptions.indexOf(currentPlaybackRate);
    const nextIndex = (currentIndex + 1) % playbackRateOptions.length;
    currentPlaybackRate = playbackRateOptions[nextIndex];
    
    if (audioElement) {
      audioElement.playbackRate = currentPlaybackRate;
    }
  }
  
  function formatTime(seconds: number): string {
    return formatDuration(seconds);
  }
  
  function handleClickOutside() {
    if (isPlaying && audioElement) {
      audioElement.pause();
      isPlaying = false;
    }
  }
</script>

<div class="media-player {message.media_type}" on:click|stopPropagation={handleClickOutside}>
  {#if error}
    <div class="media-error">
      <span>❌ {error}</span>
      <button on:click={loadMedia} class="retry-button">⟳ Réessayer</button>
    </div>
  {:else if isLoading}
    <div class="media-loading">
      <div class="spinner"></div>
      <span>Chargement sécurisé...</span>
    </div>
  {:else if !isLoaded && !isLoading}
    <button class="load-button" on:click={loadMedia}>
      {message.media_type === 'audio' ? '🔊' : '🎬'} Charger le média
    </button>
  {:else}
    <div class="media-content">
      {#if message.media_type === 'video'}
        <div class="video-container">
          <video 
            src={previewUrl || ''}
            controls
            class="video-element"
            on:loadedmetadata={(e) => {
              const video = e.target as HTMLVideoElement;
              duration = video.duration || message.duration || 0;
              isLoaded = true;
            }}
          >
            Votre navigateur ne supporte pas la lecture vidéo.
          </video>
        </div>
      {:else}
        <div class="audio-controls">
          <button class="play-button" on:click={togglePlay}>
            {isPlaying ? '⏸️' : '▶️'}
          </button>
          
          <div class="progress-container">
            <input
              type="range"
              min="0"
              max={duration}
              bind:value={currentTime}
              on:input={handleTimeUpdate}
              class="progress-slider"
              step="0.01"
              aria-label="Progression de lecture"
            />
            <div class="progress-bar">
              <div 
                class="progress-fill" 
                style="width: {progress}%;"
              ></div>
            </div>
          </div>
          
          <div class="time-display">
            <span>{formatTime(currentTime)}</span>
            <span>/</span>
            <span>{formatTime(duration)}</span>
          </div>
          
          <button class="speed-button" on:click={changePlaybackRate}>
            {currentPlaybackRate}x
          </button>
        </div>
      {/if}
      
      <div class="media-info">
        <span class="media-type">
          {message.media_type === 'audio' ? '🎤' : '🎥'} 
          {message.media_type === 'audio' ? 'Message vocal' : 'Message vidéo'}
        </span>
        <span class="media-duration">
          {formatDuration(message.duration)}
        </span>
      </div>
    </div>
  {/if}
</div>

<style>
  .media-player {
    margin: 0.5rem 0;
    border-radius: 16px;
    overflow: hidden;
    background: var(--media-bg);
    border: 1px solid var(--border);
    transition: all 0.2s;
  }
  
  .media-player:hover {
    border-color: var(--primary);
    transform: translateX(2px);
  }
  
  .media-error {
    padding: 1rem;
    text-align: center;
    color: #f44336;
    background: rgba(244, 67, 54, 0.1);
  }
  
  .retry-button {
    margin-top: 0.5rem;
    padding: 0.25rem 0.75rem;
    background: #f44336;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }
  
  .media-loading {
    padding: 1rem;
    text-align: center;
    color: var(--text-secondary);
  }
  
  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: var(--primary);
    animation: spin 1s linear infinite;
    margin: 0 auto 0.5rem;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .load-button {
    width: 100%;
    padding: 0.75rem;
    background: var(--button-bg);
    border: none;
    border-radius: 0;
    color: var(--text);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .load-button:hover {
    background: var(--primary);
    color: white;
  }
  
  .media-content {
    padding: 0.75rem;
  }
  
  .video-container {
    position: relative;
    width: 100%;
    max-height: 300px;
    overflow: hidden;
    border-radius: 12px;
    background: #000;
  }
  
  .video-element {
    width: 100%;
    height: auto;
    display: block;
    max-height: 300px;
  }
  
  .audio-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .play-button {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: none;
    background: var(--primary);
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.2s;
  }
  
  .play-button:hover {
    transform: scale(1.1);
    background: var(--primary-dark);
  }
  
  .progress-container {
    flex: 1;
    position: relative;
  }
  
  .progress-slider {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }
  
  .progress-bar {
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    cursor: pointer;
  }
  
  .progress-fill {
    height: 100%;
    background: var(--primary);
    border-radius: 2px;
    transition: width 0.1s linear;
  }
  
  .time-display {
    font-size: 0.85rem;
    color: var(--text-secondary);
    min-width: 70px;
    text-align: center;
  }
  
  .speed-button {
    width: 40px;
    height: 24px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: none;
    color: var(--text);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .speed-button:hover {
    background: var(--button-bg);
    border-color: var(--primary);
  }
  
  .media-info {
    display: flex;
    justify-content: space-between;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
    font-size: 0.85rem;
    color: var(--text-secondary);
  }
  
  .media-type {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  /* Thèmes */
  .theme-jardin-secret .media-player {
    background: rgba(76, 175, 80, 0.05);
    border-color: rgba(76, 175, 80, 0.3);
  }
  
  .theme-space-hub .media-player {
    background: rgba(33, 150, 243, 0.05);
    border-color: rgba(33, 150, 243, 0.3);
  }
  
  .theme-maison-chaleureuse .media-player {
    background: rgba(255, 152, 0, 0.05);
    border-color: rgba(255, 152, 0, 0.3);
  }
</style>