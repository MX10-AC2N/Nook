// src/lib/crypto.ts (Svelte 5 avec runes)
import { getSodium, waitForSodium } from './sodium';
import { authUser } from './authStore';

/* -----------------------------------------------------------------
   Helpers internes
   ----------------------------------------------------------------- */

/**
 * Attend que l'instance de libsodium soit disponible.
 * Retourne immédiatement si l'instance est déjà disponible.
 *
 * @returns {Promise<any>} Instance de libsodium
 */

  // Si sodium n'est pas encore chargé, attendre
  return new Promise((resolve) => {
    let resolved = false;
    
    // Créer un effet pour surveiller le chargement
    const checkSodium = () => {
      if (sodium && !resolved) {
        resolved = true;
        resolve(sodium);
      }
    };
    
    // Vérifier immédiatement
    checkSodium();
    
    // Si toujours pas chargé, attendre le prochain tick
    if (!resolved) {
      setTimeout(checkSodium, 10);
    }
  });
}

/* -----------------------------------------------------------------
   1️⃣ Génération d'une paire de clés (à faire à l'inscription)
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
    // Dans une vraie appli, utilisez l'ID réel du destinataire.
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

  // Extraire salt, nonce et ciphertext
  const salt = data.slice(0, sodium.crypto_pwhash_SALTBYTES);
  const nonce = data.slice(
    sodium.crypto_pwhash_SALTBYTES,
    sodium.crypto_pwhash_SALTBYTES + sodium.crypto_secretbox_NONCEBYTES
  );
  const ciphertext = data.slice(
    sodium.crypto_pwhash_SALTBYTES + sodium.crypto_secretbox_NONCEBYTES
  );

  // Dériver la clé à partir du mot de passe
  const key = sodium.crypto_pwhash(
    sodium.crypto_secretbox_KEYBYTES,
    password,
    salt,
    sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
    sodium.crypto_pwhash_ALG_DEFAULT
  );

  // Déchiffrer la clé privée
  const privateKey = sodium.crypto_secretbox_open_easy(ciphertext, nonce, key);
  
  return privateKey;
}

/* -----------------------------------------------------------------
   5️⃣ Stockage et récupération des clés
   ----------------------------------------------------------------- */

/**
 * Stocke les clés de l'utilisateur dans le localStorage.
 */
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

/**
 * Récupère les clés de l'utilisateur depuis le localStorage.
 */
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

/**
 * Supprime les clés de l'utilisateur du localStorage.
 */
export function clearStoredKeys(userId: string): void {
  if (typeof window === 'undefined') return;
  
  const key = `nook_keys_${userId}`;
  localStorage.removeItem(key);
}