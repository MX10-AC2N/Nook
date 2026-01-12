<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, get } from 'svelte/store';
  import { recordingState, startRecording, stopRecording, sendMediaMessage } from '$lib/mediaStore';
  import { authStore } from '$lib/authStore';
  import { getStoredKeys, decryptPrivateKey } from '$lib/crypto';
  import { activeConversationId, participants } from '$lib/conversationStore';
  import { connectionError } from '$lib/chatStore';
  import { browser } from '$app/environment';

  // -----------------------------------------------------------------
  // Props
  // -----------------------------------------------------------------
  export let disabled: boolean = false;

  // -----------------------------------------------------------------
  // UI state
  // -----------------------------------------------------------------
  let isHovered = false;
  let countdown = 3;
  let showCountdown = false;
  let countdownInterval: ReturnType<typeof setInterval> | null = null;

  // Drag‑and‑drop UI
  let isDragging = false;
  let dragTimeout: ReturnType<typeof setTimeout> | null = null;

  // Permissions (audio / video)
  const hasPermission = writable<{ audio: boolean; video: boolean }>({
    audio: false,
    video: false,
  });

  // -----------------------------------------------------------------
  // Lifecycle – permissions & drag‑and‑drop listeners
  // -----------------------------------------------------------------
  onMount(() => {
    // ---- Permissions -------------------------------------------------
    if (browser) {
      // Certaines implémentations ne supportent pas `permissions.query`,
      // on ignore les erreurs éventuelles.
      Promise.allSettled([
        navigator.permissions.query({ name: 'microphone' as PermissionName }),
        navigator.permissions.query({ name: 'camera' as PermissionName })
      ]).then((results) => {
        const audioResult = results[0];
        const videoResult = results[1];

        if (audioResult.status === 'fulfilled')
          hasPermission.update((p) => ({
            ...p,
            audio: audioResult.value.state === 'granted'
          }));
        if (videoResult.status === 'fulfilled')
          hasPermission.update((p) => ({
            ...p,
            video: videoResult.value.state === 'granted'
          }));

        // Listen for changes
        if (audioResult.status === 'fulfilled')
          audioResult.value.onchange = () => {
            hasPermission.update((p) => ({
              ...p,
              audio: audioResult.value.state === 'granted'
            }));
          };
        if (videoResult.status === 'fulfilled')
          videoResult.value.onchange = () => {
            hasPermission.update((p) => ({
              ...p,
              video: videoResult.value.state === 'granted'
            }));
          };
      });
    }

    // ---- Drag & Drop ------------------------------------------------
    if (browser) {
      window.addEventListener('dragover', handleDragOver);
      window.addEventListener('dragleave', handleDragLeave);
      window.addEventListener('drop', handleDrop);
    }

    return () => {
      if (browser) {
        window.removeEventListener('dragover', handleDragOver);
        window.removeEventListener('dragleave', handleDragLeave);
        window.removeEventListener('drop', handleDrop);
      }
      cleanupDrag();
    };
  });

  // -----------------------------------------------------------------
  // Drag & Drop helpers
  // -----------------------------------------------------------------
  function cleanupDrag() {
    if (dragTimeout) {
      clearTimeout(dragTimeout);
      dragTimeout = null;
    }
    isDragging = false;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (!isDragging) isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    // When leaving the window, `relatedTarget` is null → clean up
    if (!e.relatedTarget) cleanupDrag();
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    cleanupDrag();

    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    await processDroppedFile(file);
  }

  // -----------------------------------------------------------------
  // File processing (drop OR file‑input)
  // -----------------------------------------------------------------
  async function processDroppedFile(file: File) {
    const conversationId = get(activeConversationId);
    if (!conversationId) {
      connectionError.set('Aucune conversation sélectionnée');
      return;
    }

    // ---- Vérification du type ----
    const isAudio = file.type.startsWith('audio/');
    const isVideo = file.type.startsWith('video/');
    if (!isAudio && !isVideo) {
      connectionError.set(
        'Type de fichier non supporté : seuls les fichiers audio et vidéo sont acceptés.'
      );
      return;
    }

    // ---- Taille maximale (50 Mo) ----
    if (file.size > 50 * 1024 * 1024) {
      connectionError.set('Fichier trop volumineux : la limite est de 50 Mo.');
      return;
    }

    // ---- Récupération des clés ----
    const user = get(authStore).user;
    if (!user) return;

    const stored = await getStoredKeys(user.id);
    if (!stored) {
      connectionError.set('Clés de chiffrement introuvables');
      return;
    }

    // Demander le mot de passe si besoin
    const password = user.password ?? prompt('Entrez votre mot de passe pour chiffrer le média :');
    if (!password) return;

    const privateKey = await decryptPrivateKey(stored.encryptedPrivateKey, password);

    // ---- Récupérer les clés publiques des destinataires ----
    const convParticipants = get(participants);
    const recipientPublicKeys = convParticipants
      .filter((p) => p.id !== user.id)
      .map(() => stored.publicKey); // 👉 TODO : remplacer par les vraies clés publiques du serveur

    // ---- Créer le Blob et envoyer ----
    const blob = new Blob([await file.arrayBuffer()], { type: file.type });
    await sendMediaMessage(
      blob,
      isVideo ? 'video' : 'audio',
      conversationId,
      recipientPublicKeys,
      privateKey
    );
  }

  // -----------------------------------------------------------------
  // Permission helpers
  // -----------------------------------------------------------------
  async function requestPermission(kind: 'audio' | 'video'): Promise<boolean> {
    try {
      if (kind === 'audio')
        await navigator.mediaDevices.getUserMedia({ audio: true });
      else
        await navigator.mediaDevices.getUserMedia({ video: true, audio: true });

      hasPermission.update((p) => ({
        ...p,
        [kind]: true
      }));
      return true;
    } catch (e) {
      connectionError.set(`Permission ${kind} refusée`);
      console.error(`Permission ${kind} refusée :`, e);
      return false;
    }
  }

  // -----------------------------------------------------------------
  // Gestion du bouton d’enregistrement (audio / vidéo)
  // -----------------------------------------------------------------
  async function handleRecordClick(mediaType: 'audio' | 'video') {
    const { audio, video } = get(hasPermission);

    if (mediaType === 'audio' && !audio) {
      if (!(await requestPermission('audio'))) return;
    }
    if (mediaType === 'video' && !video) {
      if (!(await requestPermission('video'))) return;
    }

    startCountdown(mediaType);
  }

  function startCountdown(mediaType: 'audio' | 'video') {
    showCountdown = true;
    countdown = 3;

    if (countdownInterval) clearInterval(countdownInterval);
    countdownInterval = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        clearInterval(countdownInterval!);
        countdownInterval = null;
        showCountdown = false;
        startRecording(mediaType);
      }
    }, 1000);
  }

  // -----------------------------------------------------------------
  // Contrôles d’enregistrement
  // -----------------------------------------------------------------
  function handleStopRecording() {
    // `true` → on envoie le message
    stopRecording(true);
  }

  function handleCancelRecording() {
    // `false` → on annule le message
    stopRecording(false);
  }

  // -----------------------------------------------------------------
  // Envoi d’un enregistrement déjà stoppé (si on veut le déclencher manuellement)
  // -----------------------------------------------------------------
  async function handleSendRecording() {
    const state = get(recordingState);
    const conversationId = get(activeConversationId);
    if (!conversationId) {
      connectionError.set('Aucune conversation sélectionnée');
      return;
    }

    const blob = new Blob(state.chunks, {
      type: state.mediaType === 'video' ? 'video/webm' : 'audio/webm'
    });

    const user = get(authStore).user;
    if (!user) return;

    const stored = await getStoredKeys(user.id);
    if (!stored) {
      connectionError.set('Clés de chiffrement introuvables');
      return;
    }

    const password = user.password ?? prompt('Entrez votre mot de passe pour chiffrer le message :');
    if (!password) return;

    const privateKey = await decryptPrivateKey(stored.encryptedPrivateKey, password);

    const convParticipants = get(participants);
    const recipientPublicKeys = convParticipants
      .filter((p) => p.id !== user.id)
      .map(() => stored.publicKey); // 👉 TODO : récupérer les vraies clés publiques

    await sendMediaMessage(
      blob,
      state.mediaType!,
      conversationId,
      recipientPublicKeys,
      privateKey
    );
  }

  // -----------------------------------------------------------------
  // Gestion du champ <input type="file">
  // -----------------------------------------------------------------
  async function handleFileUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files?.length) return;
    await processDroppedFile(input.files[0]);
    // Reset du champ pour pouvoir re‑uploader le même fichier
    input.value = '';
  }
</script>

{#if $recordingState.isRecording}
  <!-- ==================== ENREGISTREMENT EN COURS ==================== -->
  <div class="recording-controls">
    <div class="recording-info">
      <span class="recording-indicator"></span>
      <span class="recording-duration">{$recordingState.duration}s</span>
      <span class="recording-type">
        {$recordingState.mediaType === 'audio' ? '🎤' : '🎥'} Enregistrement…
      </span>
    </div>

    <div class="recording-buttons">
      <button class="cancel-button" on:click={handleCancelRecording} aria-label="Annuler l'enregistrement">
        ✕
      </button>
      <button class="stop-button" on:click={handleStopRecording} aria-label="Arrêter l'enregistrement">
        ■
      </button>
    </div>
  </div>

{:else if showCountdown}
  <!-- ==================== COMPTE À REBOURS ==================== -->
  <div class="countdown-overlay">
    <div class="countdown-circle">{countdown}</div>
  </div>

{:else}
  <!-- ==================== BOUTONS DE CONTROLE ==================== -->
  <div
    class="media-controls {isDragging ? 'dragging' : ''}"
    on:mouseenter={() => (isHovered = true)}
    on:mouseleave={() => (isHovered = false)}
  >
    {#if isDragging}
      <div class="drag-overlay">
        <div class="drag-content">
          <span class="drag-icon">📁</span>
          <p>Déposez votre fichier audio/vidéo ici</p>
          <p class="drag-subtext">Max 50 Mo – sécurisé & chiffré</p>
        </div>
      </div>
    {/if}

    <button
      class="media-button audio"
      on:click={() => handleRecordClick('audio')}
      disabled={disabled}
      aria-label="Enregistrer un message audio"
    >
      🎙️
    </button>

    <button
      class="media-button video"
      on:click={() => handleRecordClick('video')}
      disabled={disabled || !$hasPermission.video}
      aria-label="Enregistrer un message vidéo"
    >
      🎥
    </button>

    <label class="media-button file" for="media-file-input" aria-label="Uploader un fichier audio/vidéo">
      📎
      <input
        type="file"
        id="media-file-input"
        accept="audio/*,video/*"
        hidden
        on:change={handleFileUpload}
      />
    </label>
  </div>
{/if}

<style>
  /* ----------------------- ENREGISTREMENT ----------------------- */
  .recording-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
    background: rgba(255, 0, 0, 0.1);
    border-radius: 16px;
    border: 2px solid #ff4444;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.4);
    }
    70% {
      box-shadow: 0 0 0 8px rgba(255, 0, 0, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(255, 0, 0, 0);
    }
  }

  .recording-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .recording-indicator {
    width: 12px;
    height: 12px;
    background: #ff4444;
    border-radius: 50%;
    animation: blink 1s infinite;
  }

  @keyframes blink {
    50% {
      opacity: 0.5;
    }
  }

  .recording-duration {
    font-weight: bold;
    color: #ff4444;
  }

  .recording-type {
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .recording-buttons {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .cancel-button,
  .stop-button {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: none;
    font-size: 1rem;
    font-weight: bold;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: all 0.2s;
  }

  .cancel-button {
    background: #f44336;
    color: white;
  }

  .stop-button {
    background: #4caf50;
    color: white;
  }

  .cancel-button:hover {
    background: #d32f2f;
    transform: scale(1.1);
  }

  .stop-button:hover {
    background: #43a047;
    transform: scale(1.1);
  }

  /* ----------------------- COMPTE À REBOURS ----------------------- */
  .countdown-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    background: rgba(0, 0, 0, 0.7);
    border-radius: 20px;
    z-index: 10;
  }

  .countdown-circle {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: #ff4444;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 2.5rem;
    font-weight: bold;
    color: white;
    animation: scaleUp 0.5s ease-out;
  }

  @keyframes scaleUp {
    0% {
      transform: scale(0.8);
      opacity: 0.5;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  /* ----------------------- BOUTONS DE CONTROLE ----------------------- */
  .media-controls {
    display: flex;
    gap: 0.5rem;
    position: relative;
  }

  .media-button {
    width: 44px;
    height: 44px;
    border-radius: 16px;
    border: 2px solid var(--border);
    background: var(--button-bg);
    color: var(--text);
    font-size: 1.2rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .media-button:hover:not(:disabled) {
    transform: scale(1.1);
    border-color: var(--primary);
  }

  .media-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .audio {
    background: linear-gradient(135deg, #4caf50, #2e7d32);
    color: white;
  }

  .video {
    background: linear-gradient(135deg, #2196f3, #1565c0);
    color: white;
  }

  .file {
    background: linear-gradient(135deg, #ff9800, #e65100);
    color: white;
  }

  /* ----------------------- DRAG OVERLAY ----------------------- */
  .drag-overlay {
    position: absolute;
    inset: 0;
    background: