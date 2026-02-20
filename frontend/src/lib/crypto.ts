// src/lib/crypto.ts
// Svelte 5 – fonctions cryptographiques basées sur libsodium‑wrappers

import { waitForSodium } from '$lib/sodium.svelte.js';


/* -----------------------------------------------------------------
   Types exportés
   ----------------------------------------------------------------- */
export type Sodium = typeof import('libsodium-wrappers');

/* -----------------------------------------------------------------
   Helper interne – concaténation de Uint8Array (déjà utilisé plus bas)
   ----------------------------------------------------------------- */
function concatUint8(...arrays: Uint8Array[]): Uint8Array {
  const total = arrays.reduce((sum, a) => sum + a.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }
  return result;
}

/* -----------------------------------------------------------------
   0️⃣ Initialisation du système cryptographique
   ----------------------------------------------------------------- */
/**
 * Initialise le sous‑système crypto.
 *
 * - Attend que `libsodium-wrappers` soit chargé (`waitForSodium`).
 * - (Optionnel) crée une paire de clés persistante pour l’utilisateur
 *   courant afin de vérifier que tout fonctionne.
 *
 * @returns {Promise<boolean>} `true` si tout s’est bien passé,
 *                             `false` sinon.
 */
export async function initCryptoSystem(): Promise<boolean> {
  try {
    // 1️⃣ Attendre que libsodium soit prêt
    const sodium: Sodium = await waitForSodium();

    // 2️⃣ (Optionnel) Vérifier que les primitives essentielles existent.
    //    Cette étape est surtout là pour attraper d’éventuels
    //    problèmes de chargement WASM.
    if (
      !sodium.randombytes_buf ||
      !sodium.crypto_box_keypair ||
      !sodium.crypto_secretbox_easy
    ) {
      console.error('Libsodium ne fournit pas toutes les fonctions attendues.');
      return false;
    }

    // 3️⃣ (Facultatif) Générer une paire de clés temporaire pour s’assurer
    //    que la génération fonctionne. On ne persiste rien ici ; c’est juste
    //    un « smoke test ».
    const testKeyPair = sodium.crypto_box_keypair();
    if (!testKeyPair.publicKey || !testKeyPair.privateKey) {
      console.error('Échec du test de génération de clé.');
      return false;
    }

    // Tout est OK !
    return true;
  } catch (err) {
    console.error('initCryptoSystem : erreur pendant l’initialisation', err);
    return false;
  }
}

/* -----------------------------------------------------------------
   1️⃣ Génération d’une paire de clés (exemple d’usage public)
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
  recipientPublicKeys: Uint8Array[],
  senderPrivateKey: Uint8Array
): Promise<{
  encryptedContent: Uint8Array;
  encryptedKeys: Record<string, Uint8Array>;
  nonce: Uint8Array;
}> {
  const sodium = await waitForSodium();

  // Clé de session symétrique
  const sessionKey = sodium.randombytes_buf(sodium.crypto_secretbox_KEYBYTES);

  // Chiffrement du contenu
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const contentBytes =
    typeof message === 'string' ? sodium.from_string(message) : message;
  const encryptedContent = sodium.crypto_secretbox_easy(
    contentBytes,
    nonce,
    sessionKey
  );

  // Chiffrement de la clé de session pour chaque destinataire
  const encryptedKeys: Record<string, Uint8Array> = {};
  recipientPublicKeys.forEach((pubKey, idx) => {
    const recipientId = `recipient_${idx}`;
    const asymNonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
    const encryptedKey = sodium.crypto_box_easy(
      sessionKey,
      asymNonce,
      pubKey,
      senderPrivateKey
    );
    encryptedKeys[recipientId] = concatUint8(asymNonce, encryptedKey);
  });

  return { encryptedContent, encryptedKeys, nonce };
}

/* -----------------------------------------------------------------
   3️⃣ Déchiffrement côté destinataire
   ----------------------------------------------------------------- */
export async function decryptMessage(
  encryptedContent: Uint8Array,
  encryptedKeyData: Uint8Array,
  senderPublicKey: Uint8Array,
  recipientPrivateKey: Uint8Array,
  nonce: Uint8Array
): Promise<string> {
  const sodium = await waitForSodium();

  const asymNonce = encryptedKeyData.slice(0, sodium.crypto_box_NONCEBYTES);
  const encryptedKey = encryptedKeyData.slice(sodium.crypto_box_NONCEBYTES);

  const sessionKey = sodium.crypto_box_open_easy(
    encryptedKey,
    asymNonce,
    senderPublicKey,
    recipientPrivateKey
  );

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

  const salt = sodium.randombytes_buf(sodium.crypto_pwhash_SALTBYTES);
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );

  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const encrypted = sodium.crypto_secretbox_easy(privateKey, nonce, key);

  const payload = concatUint8(salt, nonce, encrypted);
  return sodium.to_base64(payload, sodium.base64_variants.ORIGINAL);
}

export async function decryptPrivateKey(
  encryptedData: string,
  password: string
): Promise<Uint8Array> {
  const sodium = await waitForSodium();

  const data = sodium.from_base64(
    encryptedData,
    sodium.base64_variants.ORIGINAL
  );

  const salt = data.slice(0, sodium.crypto_pwhash_SALTBYTES);
  const nonce = data.slice(
    sodium.crypto_pwhash_SALTBYTES,
    sodium.crypto_pwhash_SALTBYTES + sodium.crypto_secretbox_NONCEBYTES
  );
  const ciphertext = data.slice(
    sodium.crypto_pwhash_SALTBYTES + sodium.crypto_secretbox_NONCEBYTES
  );

  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );

  const privateKey = sodium.crypto_secretbox_open_easy(
    ciphertext,
    nonce,
    key
  );

  return privateKey;
}

/* -----------------------------------------------------------------
   5️⃣ Stockage et récupération des clés
   ----------------------------------------------------------------- */
export async function storeUserKeys(
  userId: string,
  publicKey: Uint8Array,
  encryptedPrivateKey: string
): Promise<void> {
  if (typeof window === 'undefined') return;

  const key = `nook_keys_${userId}`;
  const data = {
    publicKey: Array.from(publicKey),
    encryptedPrivateKey,
    timestamp: Date.now(),
  };
  localStorage.setItem(key, JSON.stringify(data));
}

export async function getStoredKeys(
  userId: string
): Promise<{ publicKey: Uint8Array; encryptedPrivateKey: string } | null> {
  if (typeof window === 'undefined') return null;

  const key = `nook_keys_${userId}`;
  const raw = localStorage.getItem(key);
  if (!raw) return null;

  try {
    const data = JSON.parse(raw);
    return {
      publicKey: new Uint8Array(data.publicKey),
      encryptedPrivateKey: data.encryptedPrivateKey,
    };
  } catch {
    return null;
  }
}

export function clearStoredKeys(userId: string): void {
  if (typeof window === 'undefined') return;
  const key = `nook_keys_${userId}`;
  localStorage.removeItem(key);
}

/* -----------------------------------------------------------------
   6️⃣ Stockage temporaire des clés (pending join – avant mot de passe)
   ----------------------------------------------------------------- */
/**
 * Stocke les clés en clair localement en attendant l’approbation.
 * À la première connexion, ces clés seront chargées, chiffrées avec un
 * mot de passe, puis stockées via storeUserKeys.
 */
export async function storePendingKeys(
  memberId: string,
  publicKey: Uint8Array,
  privateKey: Uint8Array
): Promise<void> {
  if (typeof window === 'undefined') return;

  const storageKey = `nook_pending_keys_${memberId}`;
  const data = {
    publicKey: Array.from(publicKey),
    privateKey: Array.from(privateKey),
    timestamp: Date.now(),
  };

  localStorage.setItem(storageKey, JSON.stringify(data));
  console.log('Clés pending stockées pour memberId:', memberId);
}

/**
 * Récupère les clés pending (utilisé à la première connexion)
 */
export async function getPendingKeys(
  memberId: string
): Promise<{ publicKey: Uint8Array; privateKey: Uint8Array } | null> {
  if (typeof window === 'undefined') return null;

  const storageKey = `nook_pending_keys_${memberId}`;
  const raw = localStorage.getItem(storageKey);
  if (!raw) return null;

  try {
    const data = JSON.parse(raw);
    return {
      publicKey: new Uint8Array(data.publicKey),
      privateKey: new Uint8Array(data.privateKey),
    };
  } catch {
    return null;
  }
}

/**
 * Supprime les clés pending après migration vers le stockage chiffré
 */
export function clearPendingKeys(memberId: string): void {
  if (typeof window === 'undefined') return;
  const storageKey = `nook_pending_keys_${memberId}`;
  localStorage.removeItem(storageKey);
}
