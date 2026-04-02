// src/lib/file-transfer.svelte.ts
// Gestion du transfert de fichiers via WebRTC DataChannel.
// ── Stub — implémentation complète prévue en S47 ──

export interface FileTransferMessage {
  type: 'file-chunk' | 'file-start' | 'file-end' | 'file-cancel';
  fileId?: string;
  fileName?: string;
  fileSize?: number;
  chunkIndex?: number;
  totalChunks?: number;
  data?: ArrayBuffer;
}

/**
 * Traite un message entrant du canal 'file-transfer'.
 * @param data - ArrayBuffer reçu du DataChannel
 */
export async function handleFileTransferMessage(data: ArrayBuffer | Blob): Promise<void> {
  // TODO S47: implémenter le traitement des chunks de fichier
  // 1. Parser le message FileTransferMessage
  // 2. Bufferiser les chunks par fileId
  // 3. Reconstruire le fichier à completion (file-end)
  // 4. Décrypter si nécessaire (XChaCha20-Poly1305)
  // 5. Proposer le téléchargement au client
  console.warn('[file-transfer] handleFileTransferMessage: non implémenté (S47)');
}

/**
 * Envoie un fichier via le DataChannel de transfert.
 * @param file - Blob à envoyer
 * @param channel - RTCDataChannel actif
 * @param onProgress - Callback de progression (0-100)
 */
export async function sendFile(
  file: Blob,
  channel: RTCDataChannel,
  onProgress?: (pct: number) => void,
): Promise<void> {
  // TODO S47: implémenter l'envoi par chunks
  // 1. Chiffrer le fichier (XChaCha20-Poly1305)
  // 2. Découper en chunks de 16KB (limite DataChannel)
  // 3. Envoyer file-start → chunks → file-end
  // 4. Gérer l'ack du récepteur et les retransmissions
  console.warn('[file-transfer] sendFile: non implémenté (S47)');
  if (onProgress) onProgress(0);
}
