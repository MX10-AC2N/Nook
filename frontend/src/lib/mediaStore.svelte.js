// src/lib/mediaStore.svelte.js
// Store pour la gestion des médias (audio/vidéo) avec chiffrement E2EE

import { waitForSodium } from '$lib/sodium.svelte.js';
import { encryptForRecipients } from './crypto';
import { authStore } from './authStore.svelte.js';
import { activeConversationId } from './conversationStore.svelte.ts';
import { setConnectionError } from './chatStore.svelte.ts';

// =====================================================================
// CONSTANTES
// =====================================================================
const CHUNK_SIZE = 1024 * 1024; // 1MB chunks pour l'envoi
const MAX_RECORDING_TIME = 300000; // 5 minutes max (en ms)
const SUPPORTED_MIME_TYPES = [
  'audio/webm;codecs=opus',
  'audio/webm',
  'audio/ogg;codecs=opus',
  'audio/mp4',
  'video/webm;codecs=vp8,opus',
  'video/webm',
];

// =====================================================================
// STATE - État de l'enregistrement (Svelte 5 runes)
// =====================================================================
export const recordingState = $state({
  isRecording: false,
  isPaused: false,
  duration: 0,
  error: null,
  mediaType: null, // 'audio' ou 'video'
  chunks: [], // Chunks enregistrés
});

// Variables privées pour MediaRecorder
let mediaRecorder = null;
let mediaStream = null;
let recordingTimer = null;
let startTime = 0;

// =====================================================================
// FONCTIONS UTILITAIRES
// =====================================================================

/**
 * Trouve le type MIME supporté par le navigateur
 * @param {string} type - 'audio' ou 'video'
 */
function getSupportedMimeType(type = 'audio') {
  const types = type === 'video'
    ? SUPPORTED_MIME_TYPES.filter(t => t.startsWith('video'))
    : SUPPORTED_MIME_TYPES.filter(t => t.startsWith('audio'));

  for (const mimeType of types) {
    if (MediaRecorder.isTypeSupported(mimeType)) {
      return mimeType;
    }
  }
  return null;
}

/**
 * Formate une durée en secondes au format MM:SS
 * @param {number} seconds - Durée en secondes
 * @returns {string} Durée formatée
 */
export function formatDuration(seconds) {
  if (!seconds || isNaN(seconds)) return '00:00';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Convertit un Blob en ArrayBuffer
 * @param {Blob} blob
 * @returns {Promise<ArrayBuffer>}
 */
async function blobToArrayBuffer(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = reject;
    reader.readAsArrayBuffer(blob);
  });
}

/**
 * Convertit un ArrayBuffer en Blob
 * @param {ArrayBuffer} buffer
 * @param {string} mimeType
 * @returns {Blob}
 */
function arrayBufferToBlob(buffer, mimeType) {
  return new Blob([buffer], { type: mimeType });
}

// =====================================================================
// ENREGISTREMENT MÉDIA
// =====================================================================

/**
 * Démarre l'enregistrement audio/vidéo
 * @param {string} mediaType - 'audio' ou 'video'
 * @returns {Promise<void>}
 */
export async function startRecording(mediaType = 'audio') {
  try {
    if (!navigator.mediaDevices?.getUserMedia) {
      throw new Error('Accès microphone/caméra non disponible. Sur HTTP LAN, utilisez : chrome://flags/#unsafely-treat-insecure-origin-as-secure → ajoutez http://192.168.1.192:6300');
    }
    // Vérifier les permissions
    const constraints = {
      audio: true,
      video: mediaType === 'video' ? { 
        width: { ideal: 1280 },
        height: { ideal: 720 }
      } : false,
    };

    // Obtenir le stream média
    mediaStream = await navigator.mediaDevices.getUserMedia(constraints);

    // Trouver le type MIME supporté
    const mimeType = getSupportedMimeType(mediaType);
    if (!mimeType) {
      throw new Error(`Aucun format ${mediaType} supporté par ce navigateur`);
    }

    // Initialiser MediaRecorder
    const options = {
      mimeType,
    };
    
    if (mediaType === 'audio') {
      options.audioBitsPerSecond = 128000;
    } else {
      options.videoBitsPerSecond = 2500000;
    }

    mediaRecorder = new MediaRecorder(mediaStream, options);

    recordingState.chunks = [];

    // Gérer les données enregistrées
    mediaRecorder.ondataavailable = (event) => {
      if (event.data && event.data.size > 0) {
        recordingState.chunks.push(event.data);
      }
    };

    // Gérer les erreurs
    mediaRecorder.onerror = (event) => {
      console.error('MediaRecorder error:', event.error);
      recordingState.error = 'Erreur durant l\'enregistrement';
      cancelRecording();
    };

    // Démarrer l'enregistrement
    mediaRecorder.start(1000); // Chunk toutes les secondes
    startTime = Date.now();

    // Mettre à jour l'état
    recordingState.isRecording = true;
    recordingState.isPaused = false;
    recordingState.duration = 0;
    recordingState.error = null;
    recordingState.mediaType = mediaType;

    // Timer pour mettre à jour la durée
    recordingTimer = setInterval(() => {
      if (recordingState.isRecording && !recordingState.isPaused) {
        recordingState.duration = Math.floor((Date.now() - startTime) / 1000);

        // Arrêter automatiquement après le temps max
        if ((Date.now() - startTime) >= MAX_RECORDING_TIME) {
          stopRecording(true);
        }
      }
    }, 100);

  } catch (err) {
    console.error('Error starting recording:', err);
    recordingState.error = err.name === 'NotAllowedError' 
      ? 'Permission refusée pour accéder au microphone/caméra'
      : 'Erreur lors du démarrage de l\'enregistrement';
    stopMediaStream();
    throw err;
  }
}

/**
 * Arrête le stream média
 */
function stopMediaStream() {
  if (mediaStream) {
    mediaStream.getTracks().forEach(track => track.stop());
    mediaStream = null;
  }
}

/**
 * Met en pause l'enregistrement
 */
export function pauseRecording() {
  if (mediaRecorder && recordingState.isRecording) {
    mediaRecorder.pause();
    recordingState.isPaused = true;
  }
}

/**
 * Reprend l'enregistrement
 */
export function resumeRecording() {
  if (mediaRecorder && recordingState.isPaused) {
    mediaRecorder.resume();
    recordingState.isPaused = false;
  }
}

/**
 * Arrête l'enregistrement
 * @param {boolean} shouldSend - Si true, envoie le message. Si false, annule.
 * @returns {Promise<Blob|null>}
 */
export async function stopRecording(shouldSend = true) {
  return new Promise((resolve, reject) => {
    if (!mediaRecorder) {
      recordingState.isRecording = false;
      reject(new Error('Aucun enregistrement en cours'));
      return;
    }

    const chunks = [...recordingState.chunks];
    const mimeType = mediaRecorder.mimeType;

    mediaRecorder.onstop = async () => {
      try {
        clearInterval(recordingTimer);
        stopMediaStream();

        if (shouldSend && chunks.length > 0) {
          // Créer le blob final
          const blob = new Blob(chunks, { type: mimeType });
          
          // Réinitialiser l'état
          recordingState.isRecording = false;
          recordingState.isPaused = false;
          recordingState.duration = 0;
          recordingState.mediaType = null;
          recordingState.chunks = [];
          
          mediaRecorder = null;

          resolve(blob);
        } else {
          // Annulation
          recordingState.isRecording = false;
          recordingState.isPaused = false;
          recordingState.duration = 0;
          recordingState.mediaType = null;
          recordingState.chunks = [];
          
          mediaRecorder = null;
          resolve(null);
        }
      } catch (err) {
        console.error('Error stopping recording:', err);
        recordingState.error = 'Erreur lors de l\'arrêt de l\'enregistrement';
        reject(err);
      }
    };

    mediaRecorder.stop();
  });
}

/**
 * Annule l'enregistrement en cours
 */
export function cancelRecording() {
  if (mediaRecorder) {
    clearInterval(recordingTimer);
    mediaRecorder.stop();
    stopMediaStream();
    
    recordingState.isRecording = false;
    recordingState.isPaused = false;
    recordingState.duration = 0;
    recordingState.error = null;
    recordingState.mediaType = null;
    recordingState.chunks = [];
    
    mediaRecorder = null;
  }
}

// =====================================================================
// ENVOI DE MÉDIA CHIFFRÉ
// =====================================================================

/**
 * Envoie un message média chiffré
 * @param {Blob} mediaBlob - Le blob média à envoyer
 * @param {string} mediaType - Type de média ('audio' ou 'video')
 * @param {string} conversationId - ID de la conversation
 * @param {Array<Uint8Array>} recipientPublicKeys - Clés publiques des destinataires
 * @param {Uint8Array} senderPrivateKey - Clé privée de l'expéditeur
 * @returns {Promise<void>}
 */
export async function sendMediaMessage(mediaBlob, mediaType, conversationId, recipientPublicKeys, senderPrivateKey) {
  try {
    const sodium = await waitForSodium();

    // Convertir le blob en Uint8Array
    const arrayBuffer = await blobToArrayBuffer(mediaBlob);
    const uint8Array = new Uint8Array(arrayBuffer);

    // Chiffrer le média en utilisant encryptForRecipients de crypto.ts
    const { encryptedContent, encryptedKeys, nonce } = await encryptForRecipients(
      uint8Array,
      recipientPublicKeys,
      senderPrivateKey
    );

    // Préparer les métadonnées
    const metadata = {
      type: mediaType,
      duration: recordingState.duration || 0,
      size: mediaBlob.size,
      mimeType: mediaBlob.type,
      timestamp: Date.now(),
    };

    // Convertir en base64 pour l'envoi
    const base64Media = sodium.to_base64(encryptedContent);
    const base64Nonce = sodium.to_base64(nonce);

    // Convertir les clés chiffrées en base64
    const base64EncryptedKeys = {};
    for (const [keyId, encKey] of Object.entries(encryptedKeys)) {
      base64EncryptedKeys[keyId] = sodium.to_base64(encKey);
    }

    // Envoyer via l'API
    const response = await fetch(`/api/conversations/${conversationId}/media`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({
        media_data: base64Media,
        nonce: base64Nonce,
        encrypted_keys: base64EncryptedKeys,
        metadata,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Erreur lors de l\'envoi du média');
    }

    return await response.json();

  } catch (err) {
    console.error('Error sending media message:', err);
    setConnectionError('Erreur lors de l\'envoi du média');
    throw err;
  }
}

// =====================================================================
// RÉCEPTION ET LECTURE DE MÉDIA
// =====================================================================

/**
 * Télécharge et déchiffre un média
 * @stub — implémentation complète prévue en S38 avec cryptoStore
 */
export async function downloadAndDecryptMedia(mediaUrl, encryptedKeys, nonce, senderId) {
  // TODO S38 : implémenter avec cryptoStore.decryptMessage()
  // Les primitives getStoredKeys/decryptMessage ont été retirées de crypto.ts
  // et sont maintenant dans cryptoStore.svelte.ts
  throw new Error('[mediaStore] downloadAndDecryptMedia : non implémenté — utiliser cryptoStore.decryptMessage()');
}

export function createMediaObjectURL(blob) {
  return URL.createObjectURL(blob);
}

/**
 * Libère un URL object
 * @param {string} url
 */
export function revokeMediaObjectURL(url) {
  if (url) {
    URL.revokeObjectURL(url);
  }
}

// =====================================================================
// GESTION DES PERMISSIONS
// =====================================================================

/**
 * Vérifie si les permissions média sont accordées
 * @param {string} type - 'audio' ou 'video'
 * @returns {Promise<boolean>}
 */
export async function checkMediaPermissions(type = 'audio') {
  try {
    const permissionName = type === 'audio' ? 'microphone' : 'camera';
    const result = await navigator.permissions.query({ name: permissionName });
    return result.state === 'granted';
  } catch (err) {
    // Fallback si permissions API non disponible
    return true;
  }
}

/**
 * Demande les permissions média
 * @param {string} type - 'audio' ou 'video'
 * @returns {Promise<boolean>}
 */
export async function requestMediaPermissions(type = 'audio') {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      audio: type === 'audio',
      video: type === 'video',
    });
    
    // Arrêter immédiatement le stream
    stream.getTracks().forEach(track => track.stop());
    
    return true;
  } catch (err) {
    console.error('Permission denied:', err);
    return false;
  }
}
