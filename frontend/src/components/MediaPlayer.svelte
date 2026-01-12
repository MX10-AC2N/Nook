<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, get } from 'svelte/store';
  import { downloadAndDecryptMedia, formatDuration } from '$lib/mediaStore';
  import { connectionError } from '$lib/chatStore';

  // -----------------------------------------------------------------
  // Props
  // -----------------------------------------------------------------
  export interface ChatMessage {
    id: string;
    media_type: 'audio' | 'video' | null;
    media_url: string | null;
    encrypted_keys: Record<string, Uint8Array>;
    nonce: Uint8Array;
    sender_id: string;
    duration: number; // en secondes (0 si inconnu)
  }

  export let message: ChatMessage;
  export let isCurrentUser: boolean = false;

  // -----------------------------------------------------------------
  // Reactive state
  // -----------------------------------------------------------------
  let audioEl: HTMLAudioElement | null = null;   // pour les audios
  let previewUrl: string | null = null;         // URL blob du média déchiffré
  let isPlaying = false;
  let isLoaded = false;
  let isLoading = false;
  let error: string | null = null;

  // time / progress
  let currentTime = 0;
  let duration = 0;
  let progress = 0;

  // playback speed (audio only)
  const playbackRateOptions = [0.5, 1, 1.5, 2];
  let currentPlaybackRate = 1;

  // -----------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------
  onMount(() => {
    // Si le message possède déjà un média, on le charge immédiatement.
    if (message.media_url && !isLoaded) loadMedia();
  });

  onDestroy(() => cleanupMedia());

  // -----------------------------------------------------------------
  // Load / cleanup helpers
  // -----------------------------------------------------------------
  async function loadMedia() {
    if (isLoading || isLoaded) return;

    isLoading = true;
    error = null;

    try {
      // 1️⃣ Télécharger & déchiffrer le média
      const blob = await downloadAndDecryptMedia(
        message.media_url!,
        message.encrypted_keys,
        message.nonce,
        message.sender_id
      );

      // 2️⃣ Créer une URL blob utilisable par <audio>/<video>
      previewUrl = URL.createObjectURL(blob);
      duration = message.duration || 0;

      // 3️⃣ Instancier l’élément audio (si besoin)
      if (message.media_type === 'audio') {
        audioEl = new Audio(previewUrl);
        audioEl.preload = 'metadata';

        // Met à jour la durée dès que les métadonnées sont disponibles
        audioEl.onloadedmetadata = () => {
          duration = audioEl?.duration ?? duration;
          isLoaded = true;
          isLoading = false;
        };
        // Gestion du curseur
        audioEl.ontimeupdate = () => {
          currentTime = audioEl?.currentTime ?? 0;
          progress = duration ? (currentTime / duration) * 100 : 0;
        };
        audioEl.onended = () => {
          isPlaying = false;
          currentTime = 0;
          progress = 0;
        };
        audioEl.onerror = () => {
          error = 'Erreur lors de la lecture du média';
          isLoading = false;
          isLoaded = false;
        };
      } else {
        // Vidéo → on considère le média chargé dès que l’URL est prête
        isLoaded = true;
        isLoading = false;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement du média';
      console.error('Erreur chargement média :', e);
      isLoading = false;
    }
  }

  function cleanupMedia() {
    // Pause et détache les listeners audio
    if (audioEl) {
      audioEl.pause();
      audioEl.onloadedmetadata = null;
      audioEl.ontimeupdate = null;
      audioEl.onended = null;
      audioEl.onerror = null;
      audioEl = null;
    }

    // Révoquer l’URL blob
    if (previewUrl) {
      URL.revokeObjectURL(previewUrl);
      previewUrl = null;
    }
  }

  // -----------------------------------------------------------------
  // UI actions
  // -----------------------------------------------------------------
  function togglePlay() {
    // Si le média n’est pas encore chargé, on le charge d’abord
    if (!isLoaded && !isLoading) {
      loadMedia();
      return;
    }

    if (!audioEl || isLoading) return;

    if (isPlaying) {
      audioEl.pause();
      isPlaying = false;
    } else {
      audioEl
        .play()
        .then(() => (isPlaying = true))
        .catch((err) => {
          console.error('Erreur lecture audio :', err);
          connectionError.set('Impossible de lire le média');
        });
    }
  }

  function handleSeek(e: Event) {
    const input = e.target as HTMLInputElement;
    const sec = Number(input.value);
    currentTime = sec;
    progress = duration ? (sec / duration) * 100 : 0;
    if (audioEl) audioEl.currentTime = sec;
  }

  function changePlaybackRate() {
    const idx = playbackRateOptions.indexOf(currentPlaybackRate);
    const next = (idx + 1) % playbackRateOptions.length;
    currentPlaybackRate = playbackRateOptions[next];
    if (audioEl) audioEl.playbackRate = currentPlaybackRate;
  }

  function formatTime(sec: number): string {
    return formatDuration(sec);
  }

  // Fermer le lecteur lorsqu’on clique à l’extérieur du composant
  function handleClickOutside() {
    if (isPlaying && audioEl) {
      audioEl.pause();
      isPlaying = false;
    }
  }
</script>

<div
  class="media-player {message.media_type}"
  on:click|stopPropagation={handleClickOutside}
>
  {#if error}
    <div class="media-error">
      <span>❌ {error}</span>
      <button on:click={loadMedia} class="retry-button">⟳ Réessayer</button>
    </div>

  {:else if isLoading}
    <div class="media-loading">
      <div class="spinner"></div>
      <span>Chargement sécurisé…</span>
    </div>

  {:else if !isLoaded && !isLoading}
    <button class="load-button" on:click={loadMedia}>
      {message.media_type === 'audio' ? '🔊' : '🎬'} Charger le média
    </button>

  {:else}
    <div class="media-content">
      {#if message.media_type === 'video'}
        <!-- Vidéo -->
        <div class="video-container">
          <video
            src={previewUrl}
            controls
            class="video-element"
            on:loadedmetadata={(e) => {
              const v = e.target as HTMLVideoElement;
              duration = v.duration || message.duration || 0;
            }}
          >
            Votre navigateur ne supporte pas la lecture vidéo.
          </video>
        </div>
      {:else}
        <!-- Audio -->
        <div class="audio-controls">
          <button class="play-button" on:click={togglePlay}>
            {isPlaying ? '⏸️' : '▶️'}
          </button>

          <div class="progress-container">
            <input
              type="range"
              min="0"
              max={duration}
              step="0.01"
              bind:value={currentTime}
              on:input={handleSeek}
              class="progress-slider"
              aria-label="Progression de lecture"
            />
            <div class="progress-bar">
              <div class="progress-fill" style="width: {progress}%"></div>
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

      <!-- Infos communes -->
      <div class="media-info">
        <span class="media-type">
          {message.media_type === 'audio' ? '🎤 Message vocal' : '🎥 Message vidéo'}
        </span>
        <span class="media-duration">{formatDuration(message.duration)}</span>
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

  /* ---------- error / loading ---------- */
  .media-error,
  .media-loading {
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
    to {
      transform: rotate(360deg);
    }
  }

  .load-button {
    width: 100%;
    padding: 0.75rem;
    background: var(--button-bg);
    border: none;
    color: var(--text);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .load-button:hover {
    background: var(--primary);
    color: white;
  }

  /* ---------- content ---------- */
  .media-content {
    padding: 0.75rem;
  }

  /* ----------- video ----------- */
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

  /* ----------- audio ----------- */
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
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }

  .progress-bar {
    height: 4px;
    background: var(--border);
    border-radius: 2px;
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

  /* ---------- info ---------- */
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

  /* ---------- themes ---------- */
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