// frontend/src/lib/sodium-shim.ts
// Wrapper pour libsodium-wrappers avec import dynamique (côté client uniquement)

import { browser } from '$app/environment';

// Type pour les fonctions sodium (réduit pour l'exemple)
type SodiumModule = {
  ready: Promise<void>;
  randombytes_buf: (length: number) => Uint8Array;
  crypto_box_KEYBYTES: number;
  crypto_box_NONCEBYTES: number;
  crypto_secretbox_KEYBYTES: number;
  crypto_secretbox_NONCEBYTES: number;
  crypto_pwhash_SALTBYTES: number;
  crypto_pwhash_OPSLIMIT_INTERACTIVE: number;
  crypto_pwhash_MEMLIMIT_INTERACTIVE: number;
  crypto_pwhash_ALG_DEFAULT: number;
  crypto_secretbox_easy: (
    message: Uint8Array,
    nonce: Uint8Array,
    key: Uint8Array
  ) => Uint8Array;
  crypto_secretbox_open_easy: (
    ciphertext: Uint8Array,
    nonce: Uint8Array,
    key: Uint8Array
  ) => Uint8Array;
  crypto_box_easy: (
    message: Uint8Array,
    nonce: Uint8Array,
    publicKey: Uint8Array,
    privateKey: Uint8Array
  ) => Uint8Array;
  crypto_box_open_easy: (
    ciphertext: Uint8Array,
    nonce: Uint8Array,
    publicKey: Uint8Array,
    privateKey: Uint8Array
  ) => Uint8Array;
  crypto_box_keypair: () => { publicKey: Uint8Array; privateKey: Uint8Array };
  crypto_pwhash: (
    keyLength: number,
    password: string,
    salt: Uint8Array,
    opslimit: number,
    memlimit: number,
    algorithm: number
  ) => Uint8Array;
  from_string: (str: string) => Uint8Array;
  to_string: (arr: Uint8Array) => string;
  to_base64: (data: Uint8Array, variant: number) => string;
  from_base64: (data: string, variant: number) => Uint8Array;
  base64_variants: {
    ORIGINAL: number;
    URLSAFE: number;
  };
};

let sodium: SodiumModule | null = null;

export async function getSodium(): Promise<SodiumModule> {
  if (!browser) {
    throw new Error('libsodium-wrappers ne peut être utilisé que côté client');
  }
  
  if (!sodium) {
    const mod = await import('libsodium-wrappers');
    await mod.ready;
    sodium = mod as unknown as SodiumModule;
  }
  
  return sodium;
}

export async function initCrypto() {
  const s = await getSodium();
  await s.ready;
}

export async function generateKeyPair(): Promise<{ publicKey: Uint8Array; privateKey: Uint8Array }> {
  const s = await getSodium();
  return s.crypto_box_keypair();
}

export async function encryptForRecipients(
  message: string | Uint8Array,
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
): Promise<{
  encryptedContent: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>;
  nonce: Uint8Array;
}> {
  const s = await getSodium();

  const sessionKey = s.randombytes_buf(s.crypto_secretbox_KEYBYTES);
  const nonce = s.randombytes_buf(s.crypto_secretbox_NONCEBYTES);
  const contentBytes = typeof message === 'string' ? s.from_string(message) : message;
  const encryptedContent = s.crypto_secretbox_easy(contentBytes, nonce, sessionKey);

  const encryptedKeys: Record<string, Uint8Array> = {};

  recipientPublicKeys.forEach((publicKey, index) => {
    const recipientId = `recipient_${index}`;
    const asymNonce = s.randombytes_buf(s.crypto_box_NONCEBYTES);
    const encryptedKey = s.crypto_box_easy(sessionKey, asymNonce, publicKey, senderPrivateKey);
    encryptedKeys[recipientId] = new Uint8Array([...asymNonce, ...encryptedKey]);
  });

  return { encryptedContent, encryptedKeys, nonce };
}

export async function decryptMessage(
  encryptedContent: Uint8Array,
  encryptedKeyData: Uint8Array,
  senderPublicKey: Uint8Array,
  recipientPrivateKey: Uint8Array,
  nonce: Uint8Array
): Promise<string> {
  const s = await getSodium();

  const asymNonce = encryptedKeyData.slice(0, s.crypto_box_NONCEBYTES);
  const encryptedKey = encryptedKeyData.slice(s.crypto_box_NONCEBYTES);

  const sessionKey = s.crypto_box_open_easy(encryptedKey, asymNonce, senderPublicKey, recipientPrivateKey);
  const decrypted = s.crypto_secretbox_open_easy(encryptedContent, nonce, sessionKey);

  return s.to_string(decrypted);
}

export async function encryptPrivateKey(privateKey: Uint8Array, password: string): Promise<string> {
  const s = await getSodium();

  const salt = s.randombytes_buf(s.crypto_pwhash_SALTBYTES);
  const key = s.crypto_pwhash(
    s.crypto_secretbox_KEYBYTES,
    password,
    salt,
    s.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    s.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    s.crypto_pwhash_ALG_DEFAULT
  );

  const nonce = s.randombytes_buf(s.crypto_secretbox_NONCEBYTES);
  const encrypted = s.crypto_secretbox_easy(privateKey, nonce, key);

  const data = new Uint8Array([...salt, ...nonce, ...encrypted]);
  return s.to_base64(data, s.base64_variants.ORIGINAL);
}

export async function decryptPrivateKey(encryptedData: string, password: string): Promise<Uint8Array> {
  const s = await getSodium();

  const data = s.from_base64(encryptedData, s.base64_variants.ORIGINAL);
  const salt = data.slice(0, s.crypto_pwhash_SALTBYTES);
  const nonceStart = s.crypto_pwhash_SALTBYTES;
  const nonce = data.slice(nonceStart, nonceStart + s.crypto_secretbox_NONCEBYTES);
  const encrypted = data.slice(nonceStart + s.crypto_secretbox_NONCEBYTES);

  const key = s.crypto_pwhash(
    s.crypto_secretbox_KEYBYTES,
    password,
    salt,
    s.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    s.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    s.crypto_pwhash_ALG_DEFAULT
  );

  return s.crypto_secretbox_open_easy(encrypted, nonce, key);
}

// IndexedDB functions (inchangées car utilisent déjà browser APIs)
export async function storeEncryptedKeys(userId: string, encryptedPrivateKey: string, publicKey: Uint8Array) {
  const dbReq = indexedDB.open('nook_crypto', 1);

  dbReq.onupgradeneeded = (event) => {
    const db = (event.target as IDBRequest<IDBDatabase>).result;
    if (!db.objectStoreNames.contains('keys')) {
      db.createObjectStore('keys', { keyPath: 'userId' });
    }
  };

  dbReq.onsuccess = () => {
    const db = dbReq.result;
    const tx = db.transaction('keys', 'readwrite');
    const store = tx.objectStore('keys');

    store.put({
      userId,
      encryptedPrivateKey,
      publicKey: Array.from(publicKey)
    });

    tx.oncomplete = () => db.close();
  };
}

export async function getStoredKeys(userId: string): Promise<{
  encryptedPrivateKey: string;
  publicKey: Uint8Array;
} | null> {
  return new Promise((resolve) => {
    const dbReq = indexedDB.open('nook_crypto', 1);

    dbReq.onsuccess = () => {
      const db = dbReq.result;
      const tx = db.transaction('keys', 'readonly');
      const store = tx.objectStore('keys');

      const req = store.get(userId);

      req.onsuccess = () => {
        const data = req.result;
        if (!data) {
          resolve(null);
          return;
        }

        resolve({
          encryptedPrivateKey: data.encryptedPrivateKey,
          publicKey: new Uint8Array(data.publicKey)
        });
      };

      tx.oncomplete = () => db.close();
    };

    dbReq.onerror = () => resolve(null);
  });
}
