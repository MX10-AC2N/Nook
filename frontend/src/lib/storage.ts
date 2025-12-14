import { sodium } from 'libsodium-wrappers';

export async function initStorage() {
  await sodium.ready;
}

// Chiffre les données avec la clé privée de l'utilisateur
export async function encryptStorage( any, privateKey: string): Promise<string> {
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
  const encrypted = await encryptStorage(messages, privateKey);
  const db = await openDB();
  const tx = db.transaction('messages', 'readwrite');
  tx.store.put({ id: 'local',  encrypted });
  await tx.done;
}

// Restaure depuis IndexedDB
export async function loadMessages(privateKey: string): Promise<any[]> {
  const db = await openDB();
  const stored = await db.get('messages', 'local');
  if (stored) {
    return decryptStorage(stored.data, privateKey);
  }
  return [];
}

// Ouvre la base IndexedDB
async function openDB() {
  return new Promise<IDBDatabase>((resolve, reject) => {
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