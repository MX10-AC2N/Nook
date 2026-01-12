// src/lib/crypto.ts
import { sodiumStore, loadSodium } from './sodium';
import { authStore } from './authStore';
import { get } from 'svelte/store';

/* -----------------------------------------------------------------
   Helpers internes
   ----------------------------------------------------------------- */

/**
 * Attend que le store `sodiumStore` contienne une instance de libsodium.
 * Retourne immédiatement si l’instance est déjà disponible.
 *
 * @returns {Promise<any>} Instance de libsodium (type any car libsodium n’a pas de typings TS natives)
 */
async function waitForSodium(): Promise<any> {
  const maybe = get(sodiumStore);
  if (maybe) return maybe;

  // Sinon, on attend la première mise à jour du store
  return new Promise((resolve) => {
    const unsub = sodiumStore.subscribe((sodium) => {
      if (sodium) {
        unsub();
        resolve(sodium);
      }
    });
  });
}

/* -----------------------------------------------------------------
   1️⃣ Génération d’une paire de clés (à faire à l’inscription)
   ----------------------------------------------------------------- */
export async function generateKeyPair(): Promise<{
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}> {
  const sodium = await waitForSodium();
  return sodium.crypto_box_keypair();
}

/* -----------------------------------------------------------------
   2️⃣ Chiffrement pour un ou plusieurs destinataires
   ----------------------------------------------------------------- */
export async function encryptForRecipients(
  message: string | Uint8Array,
  recipientPublicKeys: Uint8Array[], // tableau des clés publiques des destinataires
  senderPrivateKey: Uint8Array
): Promise<{
  encryptedContent: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>; // { recipientId: nonce+encryptedKey }
  nonce: Uint8Array;
}> {
  const sodium = await waitForSodium();

  // 1️⃣ Générer une clé de session symétrique (random)
  const sessionKey = sodium.randombytes_buf(sodium.crypto_secretbox_KEYBYTES);

  // 2️⃣ Chiffrer le contenu avec cette clé (crypto_secretbox)
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const contentBytes =
    typeof message === 'string' ? sodium.from_string(message) : message;
  const encryptedContent = sodium.crypto_secretbox_easy(
    contentBytes,
    nonce,
    sessionKey
  );

  // 3️⃣ Chiffrer la clé de session pour chaque destinataire (crypto_box)
  const encryptedKeys: Record<string, Uint8Array> = {};

  recipientPublicKeys.forEach((pubKey, idx) => {
    // Dans une vraie appli, utilisez l’ID réel du destinataire.
    const recipientId = `recipient_${idx}`;

    // Nonce dédié au chiffrement asymétrique
    const asymNonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);

    // Chiffrement de la clé de session
    const encryptedKey = sodium.crypto_box_easy(
      sessionKey,
      asymNonce,
      pubKey,
      senderPrivateKey
    );

    // Stockage du nonce + clé chiffrée concaténés
    encryptedKeys[recipientId] = new Uint8Array([
      ...asymNonce,
      ...encryptedKey,
    ]);
  });

  return {
    encryptedContent,
    encryptedKeys,
    nonce,
  };
}

/* -----------------------------------------------------------------
   3️⃣ Déchiffrement côté destinataire
   ----------------------------------------------------------------- */
export async function decryptMessage(
  encryptedContent: Uint8Array,
  encryptedKeyData: Uint8Array, // nonce + clé chiffrée
  senderPublicKey: Uint8Array,
  recipientPrivateKey: Uint8Array,
  nonce: Uint8Array
): Promise<string> {
  const sodium = await waitForSodium();

  // Extraire le nonce et la clé chiffrée
  const asymNonce = encryptedKeyData.slice(0, sodium.crypto_box_NONCEBYTES);
  const encryptedKey = encryptedKeyData.slice(sodium.crypto_box_NONCEBYTES);

  // Déchiffrer la clé de session
  const sessionKey = sodium.crypto_box_open_easy(
    encryptedKey,
    asymNonce,
    senderPublicKey,
    recipientPrivateKey
  );

  // Déchiffrer le contenu du message
  const decrypted = sodium.crypto_secretbox_open_easy(
    encryptedContent,
    nonce,
    sessionKey
  );

  return sodium.to_string(decrypted);
}

/* -----------------------------------------------------------------
   4️⃣ Chiffrement / déchiffrement de la clé privée avec le mot de passe
   ----------------------------------------------------------------- */
export async function encryptPrivateKey(
  privateKey: Uint8Array,
  password: string
): Promise<string> {
  const sodium = await waitForSodium();

  // 1️⃣ Sel aléatoire
  const salt = sodium.randombytes_buf(sodium.crypto_pwhash_SALTBYTES);

  // 2️⃣ Dérivation de la clé à partir du mot de passe
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );

  // 3️⃣ Chiffrement de la clé privée
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const encrypted = sodium.crypto_secretbox_easy(privateKey, nonce, key);

  // 4️⃣ Encodage base64 (salt + nonce + ciphertext)
  const payload = new Uint8Array([...salt, ...nonce, ...encrypted]);
  return sodium.to_base64(payload, sodium.base64_variants.ORIGINAL);
}

export async function decryptPrivateKey(
  encryptedData: string,
  password: string
): Promise<Uint8Array> {
  const sodium = await waitForSodium();

  // Décodage base64
  const data = sodium.from_base64(
    encryptedData,
    sodium.base64_variants.ORIGINAL
  );

  // Extraction des composants
  const salt = data.slice(0, sodium.crypto_pwhash_SALTBYTES);
  const nonceStart = sodium.crypto_pwhash_SALTBYTES;
  const nonce = data.slice(
    nonceStart,
    nonceStart + sodium.crypto_secretbox_NONCEBYTES
  );
  const encrypted = data.slice(nonceStart + sodium.crypto_secretbox_NONCEBYTES);

  // Dérivation de la clé à partir du mot de passe
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );

  // Déchiffrement
  return sodium.crypto_secretbox_open_easy(encrypted, nonce, key);
}

/* -----------------------------------------------------------------
   5️⃣ Stockage sécurisé des clés dans IndexedDB
   ----------------------------------------------------------------- */
export async function storeEncryptedKeys(
  userId: string,
  encryptedPrivateKey: string,
  publicKey: Uint8Array
): Promise<void> {
  return new Promise((resolve, reject) => {
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

      const putReq = store.put({
        userId,
        encryptedPrivateKey,
        publicKey: Array.from(publicKey), // IndexedDB ne stocke pas directement Uint8Array
      });

      putReq.onsuccess = () => {
        tx.oncomplete = () => {
          db.close();
          resolve();
        };
      };

      putReq.onerror = () => {
        db.close();
        reject(putReq.error);
      };
    };

    dbReq.onerror = () => reject(dbReq.error);
  });
}

/**
 * Récupère les clés stockées pour un utilisateur donné.
 *
 * @param {string} userId
 * @returns {Promise<{ encryptedPrivateKey: string; publicKey: Uint8Array } | null>}
 */
export async function getStoredKeys(
  userId: string
): Promise<
  | {
      encryptedPrivateKey: string;
      publicKey: Uint8Array;
    }
  | null
> {
  return new Promise((resolve, reject) => {
    const dbReq = indexedDB.open('nook_crypto', 1);

    dbReq.onsuccess = () => {
      const db = dbReq.result;
      const tx = db.transaction('keys', 'readonly');
      const store = tx.objectStore('keys');

      const getReq = store.get(userId);

      getReq.onsuccess = () => {
        const data = getReq.result;
        if (!data) {
          resolve(null);
          return;
        }
        resolve({
          encryptedPrivateKey: data.encryptedPrivateKey,
          publicKey: new Uint8Array(data.publicKey),
        });
      };

      getReq.onerror = () => reject(getReq.error);
      tx.oncomplete = () => db.close();
    };

    dbReq.onerror = () => reject(dbReq.error);
  });
}

/* -----------------------------------------------------------------
   6️⃣ Initialisation globale du système cryptographique
   ----------------------------------------------------------------- */
export async function initCryptoSystem(): Promise<boolean> {
  try {
    // `loadSodium` provient du module `sodium.ts` et garantit que
    // libsodium est chargé avant toute utilisation.
    await loadSodium();
    console.log('✅ Système cryptographique initialisé');
    return true;
  } catch (err) {
    console.error('❌ Erreur d\'initialisation du système crypto :', err);
    return false;
  }
}