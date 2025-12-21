// frontend/src/lib/crypto.ts
import * as sodium from 'libsodium-wrappers';
import { authStore } from './authStore';

// Initialisation globale
let isInitialized = false;
export async function initCrypto() {
  if (isInitialized) return;
  await sodium.ready;
  isInitialized = true;
}

// Génération d'une clé publique/privée (à faire une fois à l'inscription)
export async function generateKeyPair() {
  await initCrypto();
  return sodium.crypto_box_keypair();
}

// Chiffrement pour 1 ou N destinataires (1:1 ou groupe)
export async function encryptForRecipients(
  message: string | Uint8Array,
  recipientPublicKeys: Uint8Array[], // Tableau des clés publiques des destinataires
  senderPrivateKey: Uint8Array
): Promise<{
  encryptedContent: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>; // { user_id: clé chiffrée }
  nonce: Uint8Array;
}> {
  await initCrypto();

  // 1. Générer une clé secrète aléatoire pour ce message
  const sessionKey = sodium.randombytes_buf(sodium.crypto_secretbox_KEYBYTES);

  // 2. Chiffrer le contenu avec cette clé (symétrique)
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const contentBytes = typeof message === 'string' ? sodium.from_string(message) : message;
  const encryptedContent = sodium.crypto_secretbox_easy(contentBytes, nonce, sessionKey);

  // 3. Chiffrer la clé de session avec chaque clé publique (asymétrique)
  const encryptedKeys: Record<string, Uint8Array> = {};
  
  recipientPublicKeys.forEach((publicKey, index) => {
    // Utiliser un identifiant unique par destinataire (ici index temporaire)
    // Dans la vraie implémentation, utiliser user_id
    const recipientId = `recipient_${index}`;
    
    // Nonce spécifique pour ce chiffrement asymétrique
    const asymNonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
    
    // Chiffrer la clé de session pour ce destinataire
    const encryptedKey = sodium.crypto_box_easy(
      sessionKey,
      asymNonce,
      publicKey,
      senderPrivateKey
    );
    
    // Stocker nonce + clé chiffrée
    encryptedKeys[recipientId] = new Uint8Array([...asymNonce, ...encryptedKey]);
  });

  return {
    encryptedContent,
    encryptedKeys,
    nonce
  };
}

// Déchiffrement par un destinataire
export async function decryptMessage(
  encryptedContent: Uint8Array,
  encryptedKeyData: Uint8Array, // nonce + clé chiffrée
  senderPublicKey: Uint8Array,
  recipientPrivateKey: Uint8Array,
  nonce: Uint8Array
): Promise<string> {
  await initCrypto();

  // 1. Extraire nonce et clé chiffrée
  const asymNonce = encryptedKeyData.slice(0, sodium.crypto_box_NONCEBYTES);
  const encryptedKey = encryptedKeyData.slice(sodium.crypto_box_NONCEBYTES);

  // 2. Déchiffrer la clé de session
  const sessionKey = sodium.crypto_box_open_easy(
    encryptedKey,
    asymNonce,
    senderPublicKey,
    recipientPrivateKey
  );

  // 3. Déchiffrer le contenu
  const decrypted = sodium.crypto_secretbox_open_easy(
    encryptedContent,
    nonce,
    sessionKey
  );

  return sodium.to_string(decrypted);
}

// Chiffrer la clé privée avec le mot de passe utilisateur (pour stockage)
export async function encryptPrivateKey(privateKey: Uint8Array, password: string): Promise<string> {
  await initCrypto();
  
  // Générer un sel aléatoire
  const salt = sodium.randombytes_buf(sodium.crypto_pwhash_SALTBYTES);
  
  // Dériver une clé à partir du mot de passe
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );
  
  // Chiffrer la clé privée
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const encrypted = sodium.crypto_secretbox_easy(privateKey, nonce, key);
  
  // Encoder en base64 pour stockage
  const data = new Uint8Array([...salt, ...nonce, ...encrypted]);
  return sodium.to_base64(data, sodium.base64_variants.ORIGINAL);
}

// Déchiffrer la clé privée avec le mot de passe
export async function decryptPrivateKey(encryptedData: string, password: string): Promise<Uint8Array> {
  await initCrypto();
  
  // Décoder depuis base64
  const data = sodium.from_base64(encryptedData, sodium.base64_variants.ORIGINAL);
  
  // Extraire les composants
  const salt = data.slice(0, sodium.crypto_pwhash_SALTBYTES);
  const nonceStart = sodium.crypto_pwhash_SALTBYTES;
  const nonce = data.slice(nonceStart, nonceStart + sodium.crypto_secretbox_NONCEBYTES);
  const encrypted = data.slice(nonceStart + sodium.crypto_secretbox_NONCEBYTES);
  
  // Dériver la clé
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );
  
  // Déchiffrer
  return sodium.crypto_secretbox_open_easy(encrypted, nonce, key);
}

// Stocker les clés chiffrées dans IndexedDB (sécurisé)
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

// Récupérer les clés
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