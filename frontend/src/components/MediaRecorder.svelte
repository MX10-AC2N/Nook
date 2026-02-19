<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { recordingState, startRecording, stopRecording, sendMediaMessage } from '$lib/mediaStore.svelte.js';
  import { authStore } from '$lib/authStore.svelte.js';
  import { getStoredKeys, decryptPrivateKey } from '$lib/crypto';
  import { conversationStore } from '$lib/conversationStore.svelte.ts';
  import { setConnectionError } from '$lib/chatStore.svelte.ts';
  import { browser } from '$app/environment';

  // -----------------------------------------------------------------
  // Props — syntaxe Svelte 5
  // -----------------------------------------------------------------
  interface Props { disabled?: boolean; }
  let { disabled = false }: Props = $props();

  // -----------------------------------------------------------------
  // UI state (Svelte 5)
  // -----------------------------------------------------------------
  let isHovered = $state(false);
  let countdown = $state(3);
  let showCountdown = $state(false);
  let countdownInterval: ReturnType<typeof setInterval> | null = null;

  let isDragging = $state(false);
  let dragTimeout: ReturnType<typeof setTimeout> | null = null;

  // Permissions — objet $state au lieu de writable store
  let hasPermission = $state({ audio: false, video: false });

  // -----------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------
  onMount(() => {
    if (browser) {
      Promise.allSettled([
        navigator.permissions.query({ name: 'microphone' as PermissionName }),
        navigator.permissions.query({ name: 'camera' as PermissionName }),
      ]).then((results) => {
        const [audioResult, videoResult] = results;

        if (audioResult.status === 'fulfilled') {
          hasPermission.audio = audioResult.value.state === 'granted';
          audioResult.value.onchange = () => {
            hasPermission.audio = audioResult.value.state === 'granted';
          };
        }
        if (videoResult.status === 'fulfilled') {
          hasPermission.video = videoResult.value.state === 'granted';
          videoResult.value.onchange = () => {
            hasPermission.video = videoResult.value.state === 'granted';
          };
        }
      });

      window.addEventListener('dragover', handleDragOver);
      window.addEventListener('dragleave', handleDragLeave);
      window.addEventListener('drop', handleDrop);
    }
  });

  onDestroy(() => {
    if (browser) {
      window.removeEventListener('dragover', handleDragOver);
      window.removeEventListener('dragleave', handleDragLeave);
      window.removeEventListener('drop', handleDrop);
    }
    cleanupDrag();
  });

  // -----------------------------------------------------------------
  // Drag & Drop
  // -----------------------------------------------------------------
  function cleanupDrag() {
    if (dragTimeout) { clearTimeout(dragTimeout); dragTimeout = null; }
    isDragging = false;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (!isDragging) isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    if (!e.relatedTarget) cleanupDrag();
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    cleanupDrag();
    const file = e.dataTransfer?.files?.[0];
    if (file) await processFile(file);
  }

  // -----------------------------------------------------------------
  // Traitement fichier (drop ou input)
  // -----------------------------------------------------------------
  async function processFile(file: File) {
    const conversationId = conversationStore.activeConversationId;
    if (!conversationId) {
      setConnectionError('Aucune conversation sélectionnée');
      return;
    }

    const isAudio = file.type.startsWith('audio/');
    const isVideo = file.type.startsWith('video/');
    if (!isAudio && !isVideo) {
      setConnectionError('Type de fichier non supporté : seuls les fichiers audio et vidéo sont acceptés.');
      return;
    }

    if (file.size > 50 * 1024 * 1024) {
      setConnectionError('Fichier trop volumineux : la limite est de 50 Mo.');
      return;
    }

    const user = authStore.user;
    if (!user) return;

    const stored = await getStoredKeys(user.id);
    if (!stored) {
      setConnectionError('Clés de chiffrement introuvables');
      return;
    }

    const password = (user as any).password ?? prompt('Entrez votre mot de passe pour chiffrer le média :');
    if (!password) return;

    const privateKey = await decryptPrivateKey(stored.encryptedPrivateKey, password);

    const convParticipants = conversationStore.participants;
    const recipientPublicKeys = convParticipants
      .filter((p) => p.id !== user.id)
      .map(() => stored.publicKey); // TODO: récupérer les vraies clés publiques

    const blob = new Blob([await file.arrayBuffer()], { type: file.type });
    await sendMediaMessage(blob, isVideo ? 'video' : 'audio', conversationId, recipientPublicKeys, privateKey);
  }

  // -----------------------------------------------------------------
  // Permissions
  // -----------------------------------------------------------------
  async function requestPermission(kind: 'audio' | 'video'): Promise<boolean> {
    try {
      await navigator.mediaDevices.getUserMedia(
        kind === 'audio' ? { audio: true } : { video: true, audio: true }
      );
      hasPermission[kind] = true;
      return true;
    } catch (e) {
      setConnectionError(`Permission ${kind} refusée`);
      console.error(`Permission ${kind} refusée :`, e);
      return false;
    }
  }

  // -----------------------------------------------------------------
  // Enregistrement
  // -----------------------------------------------------------------
  async function handleRecordClick(mediaType: 'audio' | 'video') {
    if (mediaType === 'audio' && !hasPermission.audio) {
      if (!(await requestPermission('audio'))) return;
    }
    if (mediaType === 'video' && !hasPermission.video) {
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

  function handleStopRecording() { stopRecording(true); }
  function handleCancelRecording() { stopRecording(false); }

  async function handleSendRecording() {
    const conversationId = conversationStore.activeConversationId;
    if (!conversationId) {
      setConnectionError('Aucune conversation sélectionnée');
      return;
    }

    const blob = new Blob(recordingState.chunks, {
      type: recordingState.mediaType === 'video' ? 'video/webm' : 'audio/webm',
    });

    const user = authStore.user;
    if (!user) return;

    const stored = await getStoredKeys(user.id);
    if (!stored) {
      setConnectionError('Clés de chiffrement introuvables');
      return;
    }

    const password = (user as any).password ?? prompt('Entrez votre mot de passe pour chiffrer le message :');
    if (!password) return;

    const privateKey = await decryptPrivateKey(stored.encryptedPrivateKey, password);

    const convParticipants = conversationStore.participants;
    const recipientPublicKeys = convParticipants
      .filter((p) => p.id !== user.id)
      .map(() => stored.publicKey); // TODO: vraies clés publiques

    await sendMediaMessage(blob, recordingState.mediaType!, conversationId, recipientPublicKeys, privateKey);
  }

  async function handleFileUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files?.length) return;
    await processFile(input.files[0]);
    input.value = '';
  }
</script>

{#if recordingState.isRecording}
  <div class="recording-controls">
    <div class="recording-info">
      <span class="recording-indicator"></span>
      <span class="recording-duration">{recordingState.duration}s</span>
      <span class="recording-type">
        {recordingState.mediaType === 'audio' ? '🎤' : '🎥'} Enregistrement…
      </span>
    </div>
    <div class="recording-buttons">
      <button class="cancel-button" onclick={handleCancelRecording} aria-label="Annuler l'enregistrement">✕</button>
      <button class="stop-button" onclick={handleStopRecording} aria-label="Arrêter l'enregistrement">■</button>
    </div>
  </div>

{:else if showCountdown}
  <div class="countdown-overlay">
    <div class="countdown-circle">{countdown}</div>
  </div>

{:else}
  <div
    class="media-controls {isDragging ? 'dragging' : ''}"
    onmouseenter={() => (isHovered = true)}
    onmouseleave={() => (isHovered = false)}
  >
    {#if isDragging}
      <div class="drag-overlay">
        <div class="drag-content">
          <span class="drag-icon">📁</span>
          <p>Déposez votre fichier audio/vidéo ici</p>
          <p class="drag-subtext">Max 50 Mo – sécurisé & chiffré</p>
        </div>
      </div>
    {/if}

    <button
      class="media-button audio"
      onclick={() => handleRecordClick('audio')}
      {disabled}
      aria-label="Enregistrer un message audio"
    >🎙️</button>

    <button
      class="media-button video"
      onclick={() => handleRecordClick('video')}
      disabled={disabled || !hasPermission.video}
      aria-label="Enregistrer un message vidéo"
    >🎥</button>

    <label class="media-button file" for="media-file-input" aria-label="Uploader un fichier audio/vidéo">
      📎
      <input
        type="file"
        id="media-file-input"
        accept="audio/*,video/*"
        hidden
        onchange={handleFileUpload}
      />
    </label>
  </div>
{/if}

<style>
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
    0%   { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.4); }
    70%  { box-shadow: 0 0 0 8px rgba(255, 0, 0, 0); }
    100% { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0); }
  }

  .recording-info { display: flex; align-items: center; gap: 0.5rem; }

  .recording-indicator {
    width: 12px; height: 12px;
    background: #ff4444; border-radius: 50%;
    animation: blink 1s infinite;
  }

  @keyframes blink { 50% { opacity: 0.5; } }

  .recording-duration { font-weight: bold; color: #ff4444; }
  .recording-type { font-size: 0.9rem; color: var(--text-secondary); }
  .recording-buttons { display: flex; gap: 0.5rem; margin-left: auto; }

  .cancel-button, .stop-button {
    width: 36px; height: 36px;
    border-radius: 50%; border: none;
    font-size: 1rem; font-weight: bold;
    cursor: pointer;
    display: flex; justify-content: center; align-items: center;
    transition: all 0.2s;
  }

  .cancel-button { background: #f44336; color: white; }
  .stop-button   { background: #4caf50; color: white; }
  .cancel-button:hover { background: #d32f2f; transform: scale(1.1); }
  .stop-button:hover   { background: #43a047; transform: scale(1.1); }

  .countdown-overlay {
    position: absolute; inset: 0;
    display: flex; justify-content: center; align-items: center;
    background: rgba(0, 0, 0, 0.7);
    border-radius: 20px; z-index: 10;
  }

  .countdown-circle {
    width: 80px; height: 80px;
    border-radius: 50%; background: #ff4444;
    display: flex; justify-content: center; align-items: center;
    font-size: 2.5rem; font-weight: bold; color: white;
    animation: scaleUp 0.5s ease-out;
  }

  @keyframes scaleUp {
    0%   { transform: scale(0.8); opacity: 0.5; }
    100% { transform: scale(1);   opacity: 1; }
  }

  .media-controls { display: flex; gap: 0.5rem; position: relative; }

  .media-button {
    width: 44px; height: 44px;
    border-radius: 16px;
    border: 2px solid var(--border);
    background: var(--button-bg);
    color: var(--text);
    font-size: 1.2rem; cursor: pointer;
    transition: all 0.2s;
    display: flex; justify-content: center; align-items: center;
  }

  .media-button:hover:not(:disabled) { transform: scale(1.1); border-color: var(--primary); }
  .media-button:disabled { opacity: 0.5; cursor: not-allowed; }

  .audio { background: linear-gradient(135deg, #4caf50, #2e7d32); color: white; }
  .video { background: linear-gradient(135deg, #2196f3, #1565c0); color: white; }
  .file  { background: linear-gradient(135deg, #ff9800, #e65100); color: white; }

  .drag-overlay {
    position: absolute; inset: 0;
    background: rgba(255, 152, 0, 0.95);
    border-radius: 16px;
    display: flex; justify-content: center; align-items: center;
    z-index: 5; animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

  .drag-content { text-align: center; color: white; }
  .drag-icon { font-size: 3rem; display: block; margin-bottom: 0.5rem; }
  .drag-content p { margin: 0.25rem 0; font-weight: 600; }
  .drag-subtext { font-size: 0.85rem; opacity: 0.9; }
</style>
