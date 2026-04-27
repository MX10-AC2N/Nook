// src/lib/file-transfer.svelte.ts
// Gestion du transfert de fichiers P2P via WebRTC DataChannel.
// Chiffrement E2EE avec XChaCha20-Poly1305 (via libsodium).

import sodium from 'libsodium-wrappers';
import { get as idbGet, set as idbSet } from 'idb-keyval';
import { e2ee } from './e2ee';

// Types
export interface FileTransferMessage {
  type: 'file-start' | 'file-chunk' | 'file-end' | 'file-cancel' | 'file-ack' | 'file-error';
  fileId?: string;
  fileName?: string;
  fileSize?: number;
  fileType?: string;
  chunkIndex?: number;
  totalChunks?: number;
  data?: ArrayBuffer;
  error?: string;
  nonce?: string; // Base64 du nonce pour le chunk
}

interface FileTransferState {
  fileId: string;
  fileName: string;
  fileSize: number;
  fileType: string;
  totalChunks: number;
  receivedChunks: Map<number, { data: Uint8Array; nonce: Uint8Array }>;
  startTime: number;
  progress: number;
  cancelled: boolean;
}

interface FileSendState {
  fileId: string;
  file: File;
  encryptedChunks: Array<{ data: Uint8Array; nonce: Uint8Array }>;
  sentChunks: number;
  acknowledged: boolean;
  startTime: number;
  progress: number;
}

// Constantes
const CHUNK_SIZE = 16 * 1024; // 16KB par chunk (limite DataChannel)
const MAX_RETRIES = 3;
const ACK_TIMEOUT = 5000; // 5 secondes pour attendre un ACK
const PROGRESS_THROTTLE = 100; // Mettre à jour le progrès toutes les 100ms

// État global
const activeTransfers = new Map<string, FileTransferState>();
const activeSends = new Map<string, FileSendState>();
let sodiumReady = false;

// Initialiser libsodium
async function initSodium(): Promise<void> {
  if (!sodiumReady) {
    await sodium.ready;
    sodiumReady = true;
  }
}

// Générer un ID unique pour le transfert
function generateFileId(): string {
  return `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

// Chiffrer un chunk avec XChaCha20-Poly1305
async function encryptChunk(
  chunk: Uint8Array,
  convoId: string
): Promise<{ ciphertext: Uint8Array; nonce: Uint8Array }> {
  await initSodium();
  
  // Récupérer la clé de groupe pour la conversation
  const groupKey = await getGroupKey(convoId);
  if (!groupKey) {
    throw new Error('Clé de groupe non disponible pour cette conversation');
  }
  
  // Générer un nonce unique pour ce chunk
  const nonce = sodium.randombytes_buf(sodium.crypto_aead_xchacha20poly1305_ietf_NPUBBYTES);
  
  // Chiffrer le chunk
  const ciphertext = sodium.crypto_aead_xchacha20poly1305_ietf_encrypt(
    chunk,
    null, // pas d'AD
    null, // pas de secret nonce
    nonce,
    groupKey
  );
  
  return { ciphertext, nonce };
}

// Déchiffrer un chunk avec XChaCha20-Poly1305
async function decryptChunk(
  ciphertext: Uint8Array,
  nonce: Uint8Array,
  convoId: string
): Promise<Uint8Array> {
  await initSodium();
  
  // Récupérer la clé de groupe pour la conversation
  const groupKey = await getGroupKey(convoId);
  if (!groupKey) {
    throw new Error('Clé de groupe non disponible pour cette conversation');
  }
  
  // Déchiffrer le chunk
  const decrypted = sodium.crypto_aead_xchacha20poly1305_ietf_decrypt(
    ciphertext,
    null, // pas d'AD
    null, // pas de secret nonce
    nonce,
    groupKey
  );
  
  return new Uint8Array(decrypted);
}

// Récupérer la clé de groupe depuis E2EE
async function getGroupKey(convoId: string): Promise<Uint8Array | null> {
  try {
    // Utiliser l'instance e2ee exportée depuis ./e2ee
    const groupKey = e2ee.groupKeys.get(convoId) || await e2ee.loadGroupKey(convoId);
    if (!groupKey) {
      throw new Error('Clé de groupe non disponible pour cette conversation');
    }
    return groupKey;
  } catch (error) {
    console.error('[file-transfer] Erreur récupération clé groupe:', error);
    return null;
  }
}

// Format taille fichier lisible
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

// ──────────────────────────────────────────────
// API PUBLIQUE
// ──────────────────────────────────────────────

/**
 * Envoie un fichier via le DataChannel de transfert.
 * @param file - Fichier à envoyer
 * @param channel - RTCDataChannel actif
 * @param convoId - ID de la conversation (pour la clé de groupe)
 * @param onProgress - Callback de progression (0-100)
 * @param onComplete - Callback de fin de transfert
 * @param onError - Callback d'erreur
 */
export async function sendFile(
  file: File,
  channel: RTCDataChannel,
  convoId: string,
  onProgress?: (pct: number, speed: number) => void,
  onComplete?: (fileId: string) => void,
  onError?: (error: string) => void
): Promise<void> {
  await initSodium();
  
  const fileId = generateFileId();
  const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
  
  console.log(`[file-transfer] Sending file: ${file.name} (${formatFileSize(file.size)}, ${totalChunks} chunks)`);
  
  // Créer l'état d'envoi
  const sendState: FileSendState = {
    fileId,
    file,
    encryptedChunks: [],
    sentChunks: 0,
    acknowledged: false,
    startTime: Date.now(),
    progress: 0
  };
  
  activeSends.set(fileId, sendState);
  
  try {
    // Lire et chiffrer le fichier par chunks
    const arrayBuffer = await file.arrayBuffer();
    const fileData = new Uint8Array(arrayBuffer);
    
    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, file.size);
      const chunk = fileData.slice(start, end);
      
      // Chiffrer le chunk
      const { ciphertext, nonce } = await encryptChunk(chunk, convoId);
      sendState.encryptedChunks.push({ data: ciphertext, nonce });
      
      // Mettre à jour le progrès de chiffrement
      const progress = (i + 1) / totalChunks * 100;
      if (onProgress) {
        onProgress(progress * 0.3, 0); // 30% pour le chiffrement
      }
    }
    
    // Envoyer le message de début
    const startMsg: FileTransferMessage = {
      type: 'file-start',
      fileId,
      fileName: file.name,
      fileSize: file.size,
      fileType: file.type || 'application/octet-stream',
      totalChunks
    };
    
    channel.send(JSON.stringify(startMsg));
    
    // Attendre l'ACK du début
    await waitForAck(channel, fileId, 'file-start');
    
    // Envoyer les chunks
    for (let i = 0; i < totalChunks; i++) {
      if (sendState.cancelled) {
        throw new Error('Transfer cancelled');
      }
      
      const { data, nonce } = sendState.encryptedChunks[i];
      
      // Envoyer le chunk avec son nonce
      const chunkMsg: FileTransferMessage = {
        type: 'file-chunk',
        fileId,
        chunkIndex: i,
        totalChunks,
        data: data.buffer as ArrayBuffer,
        nonce: sodium.to_base64(nonce)
      };
      
      channel.send(JSON.stringify(chunkMsg));
      sendState.sentChunks = i + 1;
      
      // Mettre à jour le progrès
      const now = Date.now();
      const elapsed = (now - sendState.startTime) / 1000;
      const bytesSent = (i + 1) * CHUNK_SIZE;
      const speed = bytesSent / elapsed / 1024; // KB/s
      
      const progress = 30 + ((i + 1) / totalChunks * 70); // 70% pour l'envoi
      sendState.progress = progress;
      
      if (onProgress) {
        onProgress(progress, speed);
      }
      
      // Petit délai pour ne pas surcharger le DataChannel
      // Réduit à 1ms pour les gros fichiers (était 10ms)
      await new Promise(resolve => setTimeout(resolve, 1));
    }
    
    // Envoyer le message de fin
    const endMsg: FileTransferMessage = {
      type: 'file-end',
      fileId,
      fileName: file.name,
      totalChunks
    };
    
    channel.send(JSON.stringify(endMsg));
    
    // Attendre l'ACK de fin
    await waitForAck(channel, fileId, 'file-end');
    
    sendState.acknowledged = true;
    console.log(`[file-transfer] File sent successfully: ${file.name}`);
    
    if (onComplete) {
      onComplete(fileId);
    }
    
  } catch (error) {
    console.error('[file-transfer] Send error:', error);
    
    // Envoyer un message d'annulation
    try {
      const cancelMsg: FileTransferMessage = {
        type: 'file-cancel',
        fileId,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
      channel.send(JSON.stringify(cancelMsg));
    } catch {}
    
    if (onError) {
      onError(error instanceof Error ? error.message : 'Transfer failed');
    }
    
    throw error;
    
  } finally {
    // Nettoyer l'état
    activeSends.delete(fileId);
  }
}

/**
 * Annuler un transfert en cours
 */
export function cancelTransfer(fileId: string): void {
  const sendState = activeSends.get(fileId);
  if (sendState) {
    sendState.cancelled = true;
    activeSends.delete(fileId);
  }
  
  const receiveState = activeTransfers.get(fileId);
  if (receiveState) {
    receiveState.cancelled = true;
    activeTransfers.delete(fileId);
  }
}

/**
 * Traite un message entrant du canal 'file-transfer'.
 * @param data - Données reçues du DataChannel
 * @param convoId - ID de la conversation (pour la clé de groupe)
 * @param channel - RTCDataChannel pour envoyer des ACKs
 * @param onFileReady - Callback quand le fichier est complet
 */
export async function handleFileTransferMessage(
  data: ArrayBuffer | string,
  convoId: string,
  channel: RTCDataChannel,
  onFileReady?: (file: File, fileId: string) => void
): Promise<void> {
  await initSodium();
  
  let message: FileTransferMessage;
  
  try {
    if (typeof data === 'string') {
      message = JSON.parse(data);
    } else {
      // C'est un message binaire (ne devrait pas arriver avec notre protocole)
      console.warn('[file-transfer] Received binary message, expected JSON');
      return;
    }
  } catch (e) {
    console.error('[file-transfer] Failed to parse message:', e);
    return;
  }
  
  switch (message.type) {
    case 'file-start':
      await handleFileStart(message, channel);
      break;
      
    case 'file-chunk':
      await handleFileChunk(message, convoId, channel, onFileReady);
      break;
      
    case 'file-end':
      await handleFileEnd(message, convoId, channel, onFileReady);
      break;
      
    case 'file-cancel':
      handleFileCancel(message);
      break;
      
    case 'file-ack':
      handleFileAck(message);
      break;
      
    case 'file-error':
      handleFileError(message);
      break;
      
    default:
      console.warn(`[file-transfer] Unknown message type: ${message.type}`);
  }
}

// ──────────────────────────────────────────────
// HANDLERS PRIVÉS
// ──────────────────────────────────────────────

async function handleFileStart(
  message: FileTransferMessage,
  channel: RTCDataChannel
): Promise<void> {
  const { fileId, fileName, fileSize, fileType, totalChunks } = message;
  
  if (!fileId || !fileName || fileSize === undefined || totalChunks === undefined) {
    console.error('[file-transfer] Invalid file-start message:', message);
    return;
  }
  
  console.log(`[file-transfer] Receiving file: ${fileName} (${formatFileSize(fileSize)}, ${totalChunks} chunks)`);
  
  // Créer l'état de réception
  const state: FileTransferState = {
    fileId,
    fileName,
    fileSize,
    fileType: fileType || 'application/octet-stream',
    totalChunks,
    receivedChunks: new Map(),
    startTime: Date.now(),
    progress: 0,
    cancelled: false
  };
  
  activeTransfers.set(fileId, state);
  
  // Envoyer l'ACK
  const ackMsg: FileTransferMessage = {
    type: 'file-ack',
    fileId,
    chunkIndex: -1, // -1 = ACK pour le début
    error: undefined
  };
  
  channel.send(JSON.stringify(ackMsg));
}

async function handleFileChunk(
  message: FileTransferMessage,
  convoId: string,
  channel: RTCDataChannel,
  onFileReady?: (file: File, fileId: string) => void
): Promise<void> {
  const { fileId, chunkIndex, totalChunks, data, nonce } = message;
  
  if (!fileId || chunkIndex === undefined || !data || !nonce) {
    console.error('[file-transfer] Invalid file-chunk message:', message);
    return;
  }
  
  const state = activeTransfers.get(fileId);
  if (!state) {
    console.error(`[file-transfer] No transfer state for fileId: ${fileId}`);
    return;
  }
  
  if (state.cancelled) {
    return;
  }
  
  try {
    // Déchiffrer le chunk
    const ciphertext = new Uint8Array(data);
    const nonceBytes = sodium.from_base64(nonce);
    const decrypted = await decryptChunk(ciphertext, nonceBytes, convoId);
    
    // Stocker le chunk déchiffré
    state.receivedChunks.set(chunkIndex, { data: decrypted, nonce: nonceBytes });
    
    // Mettre à jour le progrès
    const receivedCount = state.receivedChunks.size;
    state.progress = (receivedCount / state.totalChunks) * 100;
    
    console.log(`[file-transfer] Chunk ${chunkIndex + 1}/${state.totalChunks} received (${state.progress.toFixed(1)}%)`);
    
    // Envoyer l'ACK pour ce chunk
    const ackMsg: FileTransferMessage = {
      type: 'file-ack',
      fileId,
      chunkIndex,
      error: undefined
    };
    
    channel.send(JSON.stringify(ackMsg));
    
    // Vérifier si tous les chunks sont reçus
    if (receivedCount === state.totalChunks) {
      await assembleFile(state, onFileReady);
    }
    
  } catch (error) {
    console.error(`[file-transfer] Failed to decrypt chunk ${chunkIndex}:`, error);
    
    // Envoyer une erreur
    const errorMsg: FileTransferMessage = {
      type: 'file-error',
      fileId,
      chunkIndex,
      error: `Failed to decrypt chunk: ${error instanceof Error ? error.message : 'Unknown error'}`
    };
    
    channel.send(JSON.stringify(errorMsg));
  }
}

async function handleFileEnd(
  message: FileTransferMessage,
  convoId: string,
  channel: RTCDataChannel,
  onFileReady?: (file: File, fileId: string) => void
): Promise<void> {
  const { fileId, fileName, totalChunks } = message;
  
  if (!fileId || !fileName || totalChunks === undefined) {
    console.error('[file-transfer] Invalid file-end message:', message);
    return;
  }
  
  const state = activeTransfers.get(fileId);
  if (!state) {
    console.error(`[file-transfer] No transfer state for fileId: ${fileId}`);
    return;
  }
  
  // Vérifier si tous les chunks sont reçus
  if (state.receivedChunks.size < state.totalChunks) {
    console.warn(`[file-transfer] Missing chunks: ${state.receivedChunks.size}/${state.totalChunks}`);
    // On continue quand même, on pourrait implémenter une demande de retransmission ici
  }
  
  // Assembler le fichier
  await assembleFile(state, onFileReady);
  
  // Envoyer l'ACK final
  const ackMsg: FileTransferMessage = {
    type: 'file-ack',
    fileId,
    chunkIndex: -2, // -2 = ACK pour la fin
    error: undefined
  };
  
  channel.send(JSON.stringify(ackMsg));
}

function handleFileCancel(message: FileTransferMessage): void {
  const { fileId, error } = message;
  
  if (!fileId) {
    console.error('[file-transfer] Invalid file-cancel message:', message);
    return;
  }
  
  console.log(`[file-transfer] Transfer cancelled: ${fileId}${error ? ` (${error})` : ''}`);
  
  activeTransfers.delete(fileId);
  activeSends.delete(fileId);
}

function handleFileAck(message: FileTransferMessage): void {
  const { fileId, chunkIndex, error } = message;
  
  if (!fileId) {
    console.error('[file-transfer] Invalid file-ack message:', message);
 return;
  }
  
  if (error) {
    console.error(`[file-transfer] ACK error for ${fileId}: ${error}`);
    return;
  }
  
  const sendState = activeSends.get(fileId);
  if (sendState) {
    if (chunkIndex === -1) {
      console.log(`[file-transfer] Start ACK received for ${fileId}`);
    } else if (chunkIndex === -2) {
      console.log(`[file-transfer] End ACK received for ${fileId}`);
      sendState.acknowledged = true;
    } else {
      console.log(`[file-transfer] Chunk ${chunkIndex} ACK received`);
    }
  }
}

function handleFileError(message: FileTransferMessage): void {
  const { fileId, chunkIndex, error } = message;
  
  console.error(`[file-transfer] Error for ${fileId} (chunk ${chunkIndex}): ${error}`);
  
  // Annuler le transfert
  cancelTransfer(fileId);
}

// ──────────────────────────────────────────────
// UTILITAIRES PRIVÉS
// ──────────────────────────────────────────────

async function assembleFile(
  state: FileTransferState,
  onFileReady?: (file: File, fileId: string) => void
): Promise<void> {
  console.log(`[file-transfer] Assembling file: ${state.fileName}`);
  
  // Trier les chunks par index
  const sortedChunks = Array.from(state.receivedChunks.entries())
    .sort((a, b) => a[0] - b[0])
    .map(([_, chunk]) => chunk.data);
  
  // Calculer la taille totale
  let totalSize = 0;
  for (const chunk of sortedChunks) {
    totalSize += chunk.length;
  }
  
  // Créer un Uint8Array pour le fichier complet
  const fileData = new Uint8Array(totalSize);
  let offset = 0;
  
  for (const chunk of sortedChunks) {
    fileData.set(chunk, offset);
    offset += chunk.length;
  }
  
  // Créer un Blob et un File
  const blob = new Blob([fileData], { type: state.fileType });
  const file = new File([blob], state.fileName, { type: state.fileType });
  
  // Calculer la vitesse de transfert
  const elapsed = (Date.now() - state.startTime) / 1000;
  const speed = state.fileSize / elapsed / 1024; // KB/s
  
  console.log(`[file-transfer] File assembled: ${state.fileName} (${formatFileSize(state.fileSize)}) in ${elapsed.toFixed(1)}s (${speed.toFixed(1)} KB/s)`);
  
  // Stocker le fichier dans IndexedDB pour persistance
  await storeFileInIndexedDB(state.fileId, file);
  
  // Nettoyer l'état
  activeTransfers.delete(state.fileId);
  
  // Callback
  if (onFileReady) {
    onFileReady(file, state.fileId);
  }
}

async function waitForAck(
  channel: RTCDataChannel,
  fileId: string,
  expectedType: 'file-start' | 'file-end'
): Promise<void> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error(`Timeout waiting for ${expectedType} ACK`));
    }, ACK_TIMEOUT);
    
    const originalOnMessage = channel.onmessage;
    
    const cleanup = () => {
      clearTimeout(timeout);
      channel.onmessage = originalOnMessage;
    };
    
    channel.onmessage = (ev) => {
      try {
        const data = typeof ev.data === 'string' ? ev.data : new TextDecoder().decode(ev.data);
        const message = JSON.parse(data);
        
        if (message.type === 'file-ack' && message.fileId === fileId) {
          if (expectedType === 'file-start' && message.chunkIndex === -1) {
            cleanup();
            resolve();
          } else if (expectedType === 'file-end' && message.chunkIndex === -2) {
            cleanup();
            resolve();
          } else if (message.error) {
            cleanup();
            reject(new Error(message.error));
          }
        }
        
        // Passer au handler original si ce n'est pas notre ACK
        if (originalOnMessage) {
          originalOnMessage.call(channel, ev);
        }
      } catch (e) {
        // Ignorer les erreurs de parsing
        if (originalOnMessage) {
          originalOnMessage.call(channel, ev);
        }
      }
    };
  });
}

async function storeFileInIndexedDB(fileId: string, file: File): Promise<void> {
  try {
    await idbSet(`file_${fileId}`, {
      name: file.name,
      type: file.type,
      size: file.size,
      lastModified: file.lastModified,
      data: await file.arrayBuffer()
    });
    console.log(`[file-transfer] File stored in IndexedDB: ${fileId}`);
  } catch (error) {
    console.error('[file-transfer] Failed to store file in IndexedDB:', error);
  }
}

// ──────────────────────────────────────────────
// API DE RÉCUPÉRATION
// ──────────────────────────────────────────────

/**
 * Récupérer un fichier depuis IndexedDB
 */
export async function getFileFromIndexedDB(fileId: string): Promise<File | null> {
  try {
    const stored = await idbGet(`file_${fileId}`);
    if (!stored) return null;
    
    const blob = new Blob([stored.data], { type: stored.type });
    return new File([blob], stored.name, { type: stored.type });
  } catch (error) {
    console.error('[file-transfer] Failed to retrieve file from IndexedDB:', error);
    return null;
  }
}

/**
 * Supprimer un fichier de IndexedDB
 */
export async function deleteFileFromIndexedDB(fileId: string): Promise<void> {
  try {
    const { del } = await import('idb-keyval');
    await del(`file_${fileId}`);
    console.log(`[file-transfer] File deleted from IndexedDB: ${fileId}`);
  } catch (error) {
    console.error('[file-transfer] Failed to delete file from IndexedDB:', error);
  }
}

/**
 * Obtenir la liste des transferts actifs
 */
export function getActiveTransfers(): {
  sending: Array<{ fileId: string; fileName: string; progress: number }>;
  receiving: Array<{ fileId: string; fileName: string; progress: number }>;
} {
  const sending = Array.from(activeSends.values()).map(state => ({
    fileId: state.fileId,
    fileName: state.file.name,
    progress: state.progress
  }));
  
  const receiving = Array.from(activeTransfers.values()).map(state => ({
    fileId: state.fileId,
    fileName: state.fileName,
    progress: state.progress
  }));
  
  return { sending, receiving };
}

// Initialiser libsodium au chargement du module
initSodium().catch(console.error);
