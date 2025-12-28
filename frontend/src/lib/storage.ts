import { sodium } from 'libsodium-wrappers';

// Initialisation du stockage
export async function initStorage() {
  await sodium.ready;
}

// Chiffre les données avec la clé privée de l'utilisateur
export async function encryptStorage(data: any, privateKey: string): Promise<string> {
  await sodium.ready;
  const json = JSON.stringify(data);
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const ciphertext = sodium.crypto_secretbox_easy(
    new TextEncoder().encode(json),
    nonce,
    sodium.from_base64(privateKey)
  );
  return sodium.to_base64(nonce) + '|' + sodium.to_base64(ciphertext);
}

// Déchiffre les données
export async function decryptStorage(encrypted: string, privateKey: string): Promise<any> {
  await sodium.ready;
  const [nonceB64, cipherB64] = encrypted.split('|');
  const plaintext = sodium.crypto_secretbox_open_easy(
    sodium.from_base64(cipherB64),
    sodium.from_base64(nonceB64),
    sodium.from_base64(privateKey)
  );
  return JSON.parse(new TextDecoder().decode(plaintext));
}

// Sauvegarde dans IndexedDB
export async function saveMessages(messages: any[], privateKey: string) {
  if (typeof indexedDB === 'undefined') return;
  
  await sodium.ready;
  const encrypted = await encryptStorage(messages, privateKey);
  const db = await openDB();
  if (!db) return;
  
  const tx = db.transaction('messages', 'readwrite');
  tx.store.put({ id: 'local', encrypted });
  await tx.done;
}

// Restaure depuis IndexedDB
export async function loadMessages(privateKey: string): Promise<any[]> {
  if (typeof indexedDB === 'undefined') return [];
  
  await sodium.ready;
  const db = await openDB();
  if (!db) return [];
  
  const stored = await db.get('messages', 'local');
  if (stored) {
    return decryptStorage(stored.encrypted, privateKey);
  }
  return [];
}

// Ouvre la base IndexedDB
async function openDB(): Promise<IDBDatabase | null> {
  if (typeof indexedDB === 'undefined') return null;
  
  return new Promise((resolve, reject) => {
    const req = indexedDB.open('NookDB', 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains('messages')) {
        db.createObjectStore('messages', { keyPath: 'id' });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}
