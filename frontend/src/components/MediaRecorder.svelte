<!-- frontend/src/components/MediaRecorder.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, get } from 'svelte/store';
  import { recordingState, startRecording, stopRecording, sendMediaMessage } from '$lib/mediaStore';
  import { authStore } from '$lib/authStore';
  import { getStoredKeys, decryptPrivateKey } from '$lib/crypto';
  import { activeConversationId, participants } from '$lib/conversationStore';
  import { connectionError } from '$lib/chatStore';
  
  export let disabled: boolean = false;
  
  let isHovered = false;
  let countdown = 3;
  let showCountdown = false;
  let recordingTimer: ReturnType<typeof setTimeout> | null = null;
  
  // Gestion du drag & drop pour les fichiers médias
  let isDragging = false;
  let dragTimeout: ReturnType<typeof setTimeout> | null = null;
  
  const hasPermission = writable<{
    audio: boolean;
    video: boolean;
  }>({ audio: false, video: false });
  
  onMount(async () => {
    // Vérifier les permissions au chargement
    try {
      const audioStatus = await navigator.permissions.query({ name: 'microphone' as PermissionName });
      const videoStatus = await navigator.permissions.query({ name: 'camera' as PermissionName });
      
      hasPermission.set({
        audio: audioStatus.state === 'granted',
        video: videoStatus.state === 'granted'
      });
      
      audioStatus.onchange = () => {
        hasPermission.update(h => ({ ...h, audio: audioStatus.state === 'granted' }));
      };
      
      videoStatus.onchange = () => {
        hasPermission.update(h => ({ ...h, video: videoStatus.state === 'granted' }));
      };
    } catch (err) {
      console.warn('Impossible de vérifier les permissions:', err);
    }
    
    // Écouter les événements de drag & drop
    window.addEventListener('dragover', handleDragOver);
    window.addEventListener('dragleave', handleDragLeave);
    window.addEventListener('drop', handleDrop);
    
    return () => {
      window.removeEventListener('dragover', handleDragOver);
      window.removeEventListener('dragleave', handleDragLeave);
      window.removeEventListener('drop', handleDrop);
      cleanupDrag();
    };
  });
  
  function cleanupDrag() {
    if (dragTimeout) {
      clearTimeout(dragTimeout);
      dragTimeout = null;
    }
    isDragging = false;
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (!isDragging) {
      isDragging = true;
    }
  }
  
  function handleDragLeave(e: DragEvent) {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      cleanupDrag();
    }
  }
  
  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    cleanupDrag();
    
    if (!e.dataTransfer?.files?.length) return;
    
    const file = e.dataTransfer.files[0];
    const conversationId = get(activeConversationId);
    
    if (!conversationId) {
      connectionError.set('Aucune conversation sélectionnée');
      return;
    }
    
    // Vérifier le type de fichier
    const isValidAudio = file.type.startsWith('audio/');
    const isValidVideo = file.type.startsWith('video/');
    
    if (!isValidAudio && !isValidVideo) {
      connectionError.set('Type de fichier non supporté. Seuls les fichiers audio et vidéo sont acceptés.');
      return;
    }
    
    if (file.size > 50 * 1024 * 1024) { // 50 Mo
      connectionError.set('Fichier trop volumineux. La limite est de 50 Mo.');
      return;
    }
    
    // Obtenir les clés pour le chiffrement
    const user = get(authStore).user;
    if (!user) return;
    
    const storedKeys = await getStoredKeys(user.id);
    if (!storedKeys) {
      connectionError.set('Clés de chiffrement non trouvées');
      return;
    }
    
    const password = user.password || prompt('Entrez votre mot de passe pour chiffrer le média:');
    if (!password) return;
    
    try {
      const privateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
      
      // Obtenir les clés publiques des destinataires
      const convParticipants = get(participants);
      const recipientPublicKeys = convParticipants
        .filter(p => p.id !== user.id)
        .map(p => {
          // Dans une vraie implémentation, récupérer la clé publique depuis le backend
          return storedKeys.publicKey; // Placeholder
        });
      
      // Envoyer le média
      const blob = new Blob([await file.arrayBuffer()], { type: file.type });
      await sendMediaMessage(
        blob,
        isValidVideo ? 'video' : 'audio',
        conversationId,
        recipientPublicKeys,
        privateKey
      );
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du fichier');
      console.error('Erreur envoi fichier média:', err);
    }
  }
  
  async function startCountdown(mediaType: 'audio' | 'video') {
    showCountdown = true;
    countdown = 3;
    
    const countdownInterval = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        clearInterval(countdownInterval);
        showCountdown = false;
        startRecording(mediaType);
      }
    }, 1000);
  }
  
  async function handleRecordClick(mediaType: 'audio' | 'video') {
    const { audio, video } = get(hasPermission);
    
    if (mediaType === 'video' && !video) {
      try {
        // Demander explicitement la permission vidéo
        await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        hasPermission.update(h => ({ ...h, video: true }));
        startCountdown('video');
      } catch (err) {
        connectionError.set('Permission caméra refusée');
        console.error('Permission caméra refusée:', err);
      }
      return;
    }
    
    if (mediaType === 'audio' && !audio) {
      try {
        // Demander explicitement la permission audio
        await navigator.mediaDevices.getUserMedia({ audio: true });
        hasPermission.update(h => ({ ...h, audio: true }));
        startCountdown('audio');
      } catch (err) {
        connectionError.set('Permission microphone refusée');
        console.error('Permission microphone refusée:', err);
      }
      return;
    }
    
    startCountdown(mediaType);
  }
  
  function handleStopRecording() {
    stopRecording(true);
  }
  
  async function handleSendRecording() {
    const state = get(recordingState);
    const conversationId = get(activeConversationId);
    
    if (!conversationId) {
      connectionError.set('Aucune conversation sélectionnée');
      return;
    }
    
    if (state.chunks.length === 0) return;
    
    // Créer un blob à partir des chunks
    const blob = new Blob(state.chunks, { 
      type: state.mediaType === 'video' ? 'video/webm' : 'audio/webm' 
    });
    
    // Obtenir les clés pour le chiffrement
    const user = get(authStore).user;
    if (!user) return;
    
    const storedKeys = await getStoredKeys(user.id);
    if (!storedKeys) {
      connectionError.set('Clés de chiffrement non trouvées');
      return;
    }
    
    const password = user.password || prompt('Entrez votre mot de passe pour chiffrer le message:');
    if (!password) return;
    
    try {
      const privateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
      
      // Obtenir les clés publiques des destinataires
      const convParticipants = get(participants);
      const recipientPublicKeys = convParticipants
        .filter(p => p.id !== user.id)
        .map(p => {
          // Dans une vraie implémentation, récupérer la clé publique depuis le backend
          return storedKeys.publicKey; // Placeholder
        });
      
      // Envoyer le média
      await sendMediaMessage(
        blob,
        state.mediaType!,
        conversationId,
        recipientPublicKeys,
        privateKey
      );
    } catch (err) {
      connectionError.set('Erreur lors de l\'envoi du message vocal');
      console.error('Erreur envoi message vocal:', err);
    }
  }
  
  function handleCancelRecording() {
    stopRecording(false);
  }
</script>

{#if get(recordingState).isRecording}
  <div class="recording-controls">
    <div class="recording-info">
      <span class="recording-indicator"></span>
      <span class="recording-duration">
        {get(recordingState).duration}s
      </span>
      <span class="recording-type">
        {get(recordingState).mediaType === 'audio' ? '🎤' : '🎥'} Enregistrement...
      </span>
    </div>
    
    <div class="recording-buttons">
      <button 
        class="cancel-button" 
        on:click={handleCancelRecording}
        aria-label="Annuler l'enregistrement"
      >
        ✕
      </button>
      <button 
        class="stop-button" 
        on:click={handleStopRecording}
        aria-label="Arrêter l'enregistrement"
      >
        ■
      </button>
    </div>
  </div>
{:else if showCountdown}
  <div class="countdown-overlay">
    <div class="countdown-circle">
      {countdown}
    </div>
  </div>
{:else}
  <div 
    class="media-controls {isDragging ? 'dragging' : ''}"
    on:mouseenter={() => isHovered = true}
    on:mouseleave={() => isHovered = false}
  >
    {#if isDragging}
      <div class="drag-overlay">
        <div class="drag-content">
          <span class="drag-icon">📁</span>
          <p>Déposez votre fichier audio/vidéo ici</p>
          <p class="drag-subtext">Max 50 Mo - Sécurisé et chiffré</p>
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
      disabled={disabled || !get(hasPermission).video}
      aria-label="Enregistrer un message vidéo"
    >
      🎥
    </button>
    
    <label 
      class="media-button file" 
      for="media-file-input"
      aria-label="Uploader un fichier audio/vidéo"
    >
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
    0% { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.4); }
    70% { box-shadow: 0 0 0 8px rgba(255, 0, 0, 0); }
    100% { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0); }
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
    50% { opacity: 0.5; }
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

  .cancel-button, .stop-button {
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
    background: #4CAF50;
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

  .countdown-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
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
    0% { transform: scale(0.8); opacity: 0.5; }
    100% { transform: scale(1); opacity: 1; }
  }

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
    position: relative;
  }

  .media-button:hover:not(:disabled) {
    transform: scale(1.1);
    border-color: var(--primary);
  }

  .media-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .audio { background: linear-gradient(135deg, #4CAF50, #2E7D32); color: white; }
  .video { background: linear-gradient(135deg, #2196F3, #1565C0); color: white; }
  .file { background: linear-gradient(135deg, #FF9800, #E65100); color: white; }

  .drag-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(76, 175, 80, 0.9);
    border-radius: 20px;
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 20;
    border: 3px dashed white;
    animation: fadeIn 0.3s ease-out;
  }

  .drag-content {
    text-align: center;
    color: white;
    padding: 1rem;
  }

  .drag-icon {
    font-size: 3rem;
    margin-bottom: 0.5rem;
    animation: bounce 1s infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  .drag-subtext {
    font-size: 0.85rem;
    opacity: 0.9;
    margin-top: 0.25rem;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: scale(0.9); }
    to { opacity: 1; transform: scale(1); }
  }

  /* Thèmes */
  .theme-jardin-secret .audio { background: linear-gradient(135deg, #4CAF50, #2E7D32); }
  .theme-jardin-secret .video { background: linear-gradient(135deg, #2196F3, #0D47A1); }
  .theme-jardin-secret .file { background: linear-gradient(135deg, #FF9800, #E65100); }

  .theme-space-hub .audio { background: linear-gradient(135deg, #9C27B0, #6A1B9A); }
  .theme-space-hub .video { background: linear-gradient(135deg, #3F51B5, #1A237E); }
  .theme-space-hub .file { background: linear-gradient(135deg, #FF5722, #D84315); }

  .theme-maison-chaleureuse .audio { background: linear-gradient(135deg, #E91E63, #C2185B); }
  .theme-maison-chaleureuse .video { background: linear-gradient(135deg, #FF5722, #D84315); }
  .theme-maison-chaleureuse .file { background: linear-gradient(135deg, #FF9800, #E65100); }
</style>