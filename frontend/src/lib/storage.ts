/**
 * Storage utilities used by Nook.
 *
 * - Initialise libsodium (`initStorage`).
 * - Chiffre/Déchiffre des objets JSON avec une clé symétrique (fourni sous forme
 *   de chaîne Base64, généralement la clé privée de l’utilisateur après dérivation).
 * - Persistance des messages chiffrés dans IndexedDB.
 *
 * Toutes les fonctions sont typées, les erreurs sont gérées et le code
 * fonctionne à la fois côté client (browser) et côté serveur (SSR) grâce à
 * des vérifications `typeof indexedDB !== 'undefined'`.
 */

import sodium from 'libsodium-wrappers'; // <-- default export (ready, crypto_*, …)

// -----------------------------------------------------------------
// 1️⃣ Initialisation de libsodium (à appeler une fois au démarrage)
// -----------------------------------------------------------------
export async function initStorage(): Promise<void> {
  await sodium.ready;
}

// -----------------------------------------------------------------
// 2️⃣ Chiffrement / déchiffrement (secretbox)
// -----------------------------------------------------------------
/**
 * Chiffre un objet JavaScript avec la clé fournie.
 *
 * @param data       Données à chiffrer (tout ce qui peut être sérialisé en JSON).
 * @param secretKeyB64  Clé symétrique encodée en Base64 (32 bytes = crypto_secretbox_KEYBYTES).
 * @returns          Chaîne `nonce|ciphertext` (Base64, séparées par `|`).
 */
export async function encryptStorage(
  data: unknown,
  secretKeyB64: string
): Promise<string> {
  await sodium.ready;

  const key = sodium.from_base64(secretKeyB64, sodium.base64_variants.ORIGINAL);
  if (key.length !== sodium.crypto_secretbox_KEYBYTES) {
    throw new Error('Clé de chiffrement invalide (taille attendue : 32 bytes)');
  }

  const plaintext = new TextEncoder().encode(JSON.stringify(data));
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const ciphertext = sodium.crypto_secretbox_easy(plaintext, nonce, key);

  // Retour sous la forme « nonce|ciphertext » (Base64)
  return (
    sodium.to_base64(nonce, sodium.base64_variants.ORIGINAL) +
    '|' +
    sodium.to_base64(ciphertext, sodium.base64_variants.ORIGINAL)
  );
}

/**
 * Déchiffre une chaîne produite par `encryptStorage`.
 *
 * @param encrypted   Chaîne `nonce|ciphertext` (Base64).
 * @param secretKeyB64  Clé symétrique encodée en Base64.
 * @returns           L’objet JavaScript d’origine.
 */
export async function decryptStorage<T = unknown>(
  encrypted: string,
  secretKeyB64: string
): Promise<T> {
  await sodium.ready;

  const key = sodium.from_base64(secretKeyB64, sodium.base64_variants.ORIGINAL);
  if (key.length !== sodium.crypto_secretbox_KEYBYTES) {
    throw new Error('Clé de chiffrement invalide (taille attendue : 32 bytes)');
  }

  const [nonceB64, cipherB64] = encrypted.split('|');
  if (!nonceB64 || !cipherB64) {
    throw new Error('Format de donnée chiffrée invalide');
  }

  const nonce = sodium.from_base64(nonceB64, sodium.base64_variants.ORIGINAL);
  const ciphertext = sodium.from_base64(cipherB64, sodium.base64_variants.ORIGINAL);

  const plaintext = sodium.crypto_secretbox_open_easy(ciphertext, nonce, key);
  if (!plaintext) {
    throw new Error('Déchiffrement échoué – données corrompues ou mauvaise clé');
  }

  const decoded = new TextDecoder().decode(plaintext);
  return JSON.parse(decoded) as T;
}

// -----------------------------------------------------------------
// 3️⃣ Persistance dans IndexedDB
// -----------------------------------------------------------------
interface EncryptedRecord {
  id: string;          // clé primaire (ex. « local »)
  encrypted: string;   // valeur chiffrée
}

/**
 * Ouvre (ou crée) la base IndexedDB « NookDB » avec un objectStore « messages ».
 *
 * @returns IDBDatabase instance ou `null` si IndexedDB n’est pas disponible.
 */
async function openDB(): Promise<IDBDatabase | null> {
  if (typeof indexedDB === 'undefined') return null;

  return new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open('NookDB', 1);

    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains('messages')) {
        db.createObjectStore('messages', { keyPath: 'id' });
      }
    };

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * Sauvegarde la liste de messages chiffrés dans IndexedDB.
 *
 * @param messages   Tableau d’objets à stocker.
 * @param secretKeyB64  Clé symétrique (Base64) utilisée pour le chiffrement.
 */
export async function saveMessages(
  messages: unknown[],
  secretKeyB64: string
): Promise<void> {
  if (typeof indexedDB === 'undefined') return; // SSR safety

  const encrypted = await encryptStorage(messages, secretKeyB64);
  const db = await openDB();
  if (!db) return;

  const tx = db.transaction('messages', 'readwrite');
  const store = tx.objectStore('messages');
  const record: EncryptedRecord = { id: 'local', encrypted };
  store.put(record);
  await tx.complete;
}

/**
 * Charge et déchiffre les messages depuis IndexedDB.
 *
 * @param secretKeyB64  Clé symétrique (Base64) utilisée pour le déchiffrement.
 * @returns            Tableau d’objets (ou tableau vide si aucune donnée).
 */
export async function loadMessages<T = unknown[]>(
  secretKeyB64: string
): Promise<T> {
  if (typeof indexedDB === 'undefined') return [] as unknown as T; // SSR safety

  const db = await openDB();
  if (!db) return [] as unknown as T;

  const tx = db.transaction('messages', 'readonly');
  const store = tx.objectStore('messages');
  const record = await store.get('local');

  if (record && typeof record.encrypted === 'string') {
    return (await decryptStorage<T>(record.encrypted, secretKeyB64)) ?? ([] as unknown as T);
  }

  return [] as unknown as T;
}