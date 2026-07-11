// src/lib/crypto.ts
//
// Source unique de vérité pour toute la cryptographie Nook.
// Architecture : clé de session par message (crypto_box_easy)
//
// Flux complet :
//   1. À l'inscription  → generateKeyPair() → storePendingKeys()
//                         La pubkey est envoyée au serveur dans join.
//   2. Au 1er login     → getPendingKeys() + mot de passe
//                       → encryptPrivateKey(privKey, password)
//                       → storeKeysInIndexedDB(userId, pubKey, encPrivKey)
//                       → registerPublicKey(pubKey) — appel API
//   3. À chaque login   → loadKeysFromIndexedDB(userId)
//                       → decryptPrivateKey(encPrivKey, password)
//                       → clés prêtes en mémoire (cryptoStore)
//   4. Envoi message    → fetchMemberPubkeys(convId)
//                       → encryptForRecipients(text, pubkeys, myPrivKey)
//                       → POST /api/conversations/{id}/messages avec encrypted_keys
//   5. Réception        → GET /api/conversations/{id}/my-encrypted-key/{msgId}
//                       → decryptSessionKey(encKey, senderPubKey, myPrivKey)
//                       → decryptContent(ciphertext, nonce, sessionKey)
//
// DT-05 : Versionnement de clés. Chaque rotation de clé incrémente un compteur.
//         Le sender_key_version est stocké dans message_keys pour permettre
//         le déchiffrement des anciens messages après rotation.

// DT-01 : pas d'import statique — dynamic import dans ensureSodium()
// Le chunk libsodium (938 kB WASM) n'est téléchargé qu'au premier appel crypto.

// ─────────────────────────────────────────────────────────────────────────────
// Initialisation libsodium (singleton, dynamic import)
// ─────────────────────────────────────────────────────────────────────────────
import { waitForSodium, getSodiumInstance } from './sodium.svelte.js';

type SodiumType = typeof import('libsodium-wrappers').default;

async function ensureSodium(): Promise<SodiumType> {
  // Use shared instance from sodium.svelte.js
  const existing = getSodiumInstance();
  if (existing) return existing;
  return await waitForSodium();
}

/** Smoke-test libsodium au démarrage. Retourne true si OK. */
export async function initCryptoSystem(): Promise<boolean> {
  try {
    const na = await ensureSodium();
    // Vérifier que les primitives utilisées sont disponibles
    if (
      !na.randombytes_buf ||
      !na.crypto_box_keypair ||
      !na.crypto_box_easy ||
      !na.crypto_secretbox_easy
    ) {
      console.error('[crypto] Primitives manquantes dans libsodium');
      return false;
    }
    return true;
  } catch (e) {
    console.error('[crypto] initCryptoSystem:', e);
    return false;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export interface KeyPair {
  publicKey:  Uint8Array;  // 32 bytes X25519
  privateKey: Uint8Array;  // 32 bytes X25519
}

export interface EncryptedMessage {
  /** Ciphertext du contenu (secretbox) en base64 */
  ciphertext: string;
  /** Nonce secretbox en base64 (24 bytes) */
  nonce: string;
  /** Clé de session chiffrée pour chaque destinataire : user_id → base64 */
  encryptedKeys: Record<string, string>;
}

/** Entrée d'archive de clé — une ancienne version après rotation */
export interface ArchivedKeyEntry {
  version: number;
  publicKeyB64: string;
  encryptedPrivateKeyB64: string; // salt(16) || nonce(24) || ciphertext (chiffré avec mot de passe)
  createdAt: number;
}

/** Format V2 du document IndexedDB */
export interface StoredKeysV2 {
  userId: string;
  schemaVersion: 2;
  currentVersion: number;
  publicKeyB64: string;
  encryptedPrivKeyB64: string;
  keyHistory: ArchivedKeyEntry[];
  passwordSalt: string; // base64 — salt fixe pour archive key derivation
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Génération de paire de clés
// ─────────────────────────────────────────────────────────────────────────────
export async function generateKeyPair(): Promise<KeyPair> {
  const na = await ensureSodium();
  const kp = na.crypto_box_keypair();
  return { publicKey: kp.publicKey, privateKey: kp.privateKey };
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Chiffrement d'un message pour plusieurs destinataires
//    Utilise crypto_box_easy (X25519 + XSalsa20-Poly1305) pour chaque clé de session
//    et crypto_secretbox_easy pour le contenu.
// ─────────────────────────────────────────────────────────────────────────────
export async function encryptForRecipients(
  plaintext: string,
  /** { userId: base64PublicKey } — clés publiques des destinataires */
  recipientPubkeys: Record<string, string>,
  senderKeyPair: KeyPair
): Promise<EncryptedMessage> {
  const na = await ensureSodium();

  // Clé de session éphémère 32 bytes
  const sessionKey = na.randombytes_buf(na.crypto_secretbox_KEYBYTES);

  // Chiffrement symétrique du contenu
  const nonce      = na.randombytes_buf(na.crypto_secretbox_NONCEBYTES);
  const msgBytes   = na.from_string(plaintext);
  const ciphertext = na.crypto_secretbox_easy(msgBytes, nonce, sessionKey);

  // Chiffrement de la clé de session pour chaque destinataire
  const encryptedKeys: Record<string, string> = {};
  for (const [userId, pubKeyB64] of Object.entries(recipientPubkeys)) {
    try {
      const recipientPub = na.from_base64(pubKeyB64, na.base64_variants.ORIGINAL);
      const asymNonce    = na.randombytes_buf(na.crypto_box_NONCEBYTES);
      const boxed        = na.crypto_box_easy(
        sessionKey,
        asymNonce,
        recipientPub,
        senderKeyPair.privateKey
      );
      // Stocker : asymNonce(24) || boxed — le destinataire les sépare à la réception
      const combined = new Uint8Array(asymNonce.length + boxed.length);
      combined.set(asymNonce, 0);
      combined.set(boxed, asymNonce.length);
      encryptedKeys[userId] = na.to_base64(combined, na.base64_variants.ORIGINAL);
    } catch (e) {
      console.warn('[encryptForRecipients] Échec chiffrement pour', userId, e?.message);
    }
  }

  return {
    ciphertext: na.to_base64(ciphertext, na.base64_variants.ORIGINAL),
    nonce:      na.to_base64(nonce,      na.base64_variants.ORIGINAL),
    encryptedKeys,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Déchiffrement d'un message reçu
// ─────────────────────────────────────────────────────────────────────────────
/**
 * Déchiffre la clé de session chiffrée pour ce destinataire.
 * @param encKeyB64    base64(asymNonce[24] || boxCiphertext) — depuis message_keys
 * @param senderPubB64 Clé publique X25519 de l'expéditeur en base64
 * @param myPrivKey    Clé privée du destinataire
 */
export async function decryptSessionKey(
  encKeyB64:    string,
  senderPubB64: string,
  myPrivKey:    Uint8Array
): Promise<Uint8Array> {
  const na        = await ensureSodium();
  const combined  = na.from_base64(encKeyB64,    na.base64_variants.ORIGINAL);
  const senderPub = na.from_base64(senderPubB64, na.base64_variants.ORIGINAL);

  const asymNonce = combined.slice(0, na.crypto_box_NONCEBYTES);
  const boxed     = combined.slice(na.crypto_box_NONCEBYTES);

  if (boxed.length < na.crypto_secretbox_MACBYTES) {
    console.error('[crypto] decryptSessionKey: boxed trop court', boxed.length, 'attendu au moins', na.crypto_secretbox_MACBYTES);
  }

  return na.crypto_box_open_easy(boxed, asymNonce, senderPub, myPrivKey);
}

/**
 * Déchiffre le contenu d'un message avec la clé de session.
 */
export async function decryptContent(
  ciphertextB64: string,
  nonceB64:      string,
  sessionKey:    Uint8Array
): Promise<string> {
  const na         = await ensureSodium();
  const ciphertext = na.from_base64(ciphertextB64, na.base64_variants.ORIGINAL);
  const nonce      = na.from_base64(nonceB64,      na.base64_variants.ORIGINAL);
  const decrypted  = na.crypto_secretbox_open_easy(ciphertext, nonce, sessionKey);
  return na.to_string(decrypted);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Chiffrement / déchiffrement de la clé privée avec mot de passe (pwhash)
//    Stockage : IndexedDB via openCryptoStore()
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Chiffre la clé privée avec le mot de passe de l'utilisateur.
 * Retourne une chaîne base64 : salt(16) || nonce(24) || ciphertext
 */
export async function encryptPrivateKey(
  privateKey: Uint8Array,
  password:   string
): Promise<string> {
  const na   = await ensureSodium();
  const salt = na.randombytes_buf(16);

  // Derive key using BLAKE2b (crypto_pwhash not available in this build)
  const passwordBytes = new TextEncoder().encode(password);
  const saltedPw = new Uint8Array(passwordBytes.length + salt.length);
  saltedPw.set(passwordBytes);
  saltedPw.set(salt, passwordBytes.length);
  const key = na.crypto_generichash(na.crypto_secretbox_KEYBYTES, saltedPw);

  const nonce     = na.randombytes_buf(na.crypto_secretbox_NONCEBYTES);
  const encrypted = na.crypto_secretbox_easy(privateKey, nonce, key);

  const payload = new Uint8Array(salt.length + nonce.length + encrypted.length);
  payload.set(salt,      0);
  payload.set(nonce,     salt.length);
  payload.set(encrypted, salt.length + nonce.length);

  return na.to_base64(payload, na.base64_variants.ORIGINAL);
}

/**
 * Déchiffre la clé privée à partir du blob base64 et du mot de passe.
 */
export async function decryptPrivateKey(
  encryptedB64: string,
  password:     string
): Promise<Uint8Array> {
  const na   = await ensureSodium();
  const data = na.from_base64(encryptedB64, na.base64_variants.ORIGINAL);

  const salt       = data.slice(0, 16);
  const nonce      = data.slice(16, 16 + na.crypto_secretbox_NONCEBYTES);
  const ciphertext = data.slice(16 + na.crypto_secretbox_NONCEBYTES);

  // Derive key using BLAKE2b
  const passwordBytes = new TextEncoder().encode(password);
  const saltedPw = new Uint8Array(passwordBytes.length + salt.length);
  saltedPw.set(passwordBytes);
  saltedPw.set(salt, passwordBytes.length);
  const key = na.crypto_generichash(na.crypto_secretbox_KEYBYTES, saltedPw);

  return na.crypto_secretbox_open_easy(ciphertext, nonce, key);
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. IndexedDB — stockage des clés chiffrées (v2: key archive)
// ─────────────────────────────────────────────────────────────────────────────

const IDB_NAME    = 'NookCrypto';
const IDB_VERSION = 2;  // v2: key archive support
const IDB_STORE   = 'keys';

function openCryptoStore(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, IDB_VERSION);
    req.onupgradeneeded = (event) => {
      const db = req.result;
      if (!db.objectStoreNames.contains(IDB_STORE)) {
        db.createObjectStore(IDB_STORE, { keyPath: 'userId' });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });
}

interface StoredKeys {
  userId:              string;
  publicKeyB64:        string;  // base64 — 32 bytes X25519
  encryptedPrivKeyB64: string;  // base64 — salt||nonce||ciphertext (voir encryptPrivateKey)
}

/**
 * Stocke la paire de clés (pubkey en clair, privkey chiffrée) dans IndexedDB.
 * Appelé une seule fois lors du changement de mot de passe initial.
 * Write V2 format with currentVersion=1.
 */
export async function storeKeysInIndexedDB(
  userId:              string,
  publicKey:           Uint8Array,
  encryptedPrivKeyB64: string
): Promise<void> {
  if (typeof indexedDB === 'undefined') return;
  const na = await ensureSodium();
  const db = await openCryptoStore();

  // Generate a stable password salt for archive key derivation
  const salt = na.randombytes_buf(16);

  const v2record: StoredKeysV2 = {
    userId,
    schemaVersion: 2,
    currentVersion: 1,
    publicKeyB64: na.to_base64(publicKey, na.base64_variants.ORIGINAL),
    encryptedPrivKeyB64,
    keyHistory: [],
    passwordSalt: na.to_base64(salt, na.base64_variants.ORIGINAL),
  };

  await new Promise<void>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readwrite');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.put(v2record);
    req.onsuccess = () => resolve();
    req.onerror   = () => reject(req.error);
  });
}

/**
 * Charge les clés chiffrées depuis IndexedDB et les déchiffre avec le mot de passe.
 * Retourne null si aucune clé n'est stockée pour cet utilisateur.
 * Supporte V1 et V2.
 */
export async function loadKeysFromIndexedDB(
  userId:   string,
  password: string
): Promise<KeyPair | null> {
  if (typeof indexedDB === 'undefined') return null;
  const na = await ensureSodium();

  const db = await openCryptoStore();
  const record = await new Promise<StoredKeys | StoredKeysV2 | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result as StoredKeys | StoredKeysV2 | undefined);
    req.onerror   = () => reject(req.error);
  });

  if (!record) return null;

  let publicKeyB64: string;
  let encryptedPrivKeyB64: string;

  if ('schemaVersion' in record && record.schemaVersion === 2) {
    const v2 = record as StoredKeysV2;
    publicKeyB64 = v2.publicKeyB64;
    encryptedPrivKeyB64 = v2.encryptedPrivKeyB64;
  } else {
    const v1 = record as StoredKeys;
    publicKeyB64 = v1.publicKeyB64;
    encryptedPrivKeyB64 = v1.encryptedPrivKeyB64;
  }

  const privateKey = await decryptPrivateKey(encryptedPrivKeyB64, password);
  const publicKey  = na.from_base64(publicKeyB64, na.base64_variants.ORIGINAL);
  return { publicKey, privateKey };
}

/**
 * Vérifie si des clés sont stockées pour cet utilisateur (sans les déchiffrer).
 */
export async function hasStoredKeys(userId: string): Promise<boolean> {
  if (typeof indexedDB === 'undefined') return false;
  const db = await openCryptoStore();
  const record = await new Promise<StoredKeys | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result as StoredKeys | undefined);
    req.onerror   = () => reject(req.error);
  });
  return record !== undefined;
}

/**
 * Supprime les clés d'un utilisateur (logout complet ou réinitialisation).
 */
export async function clearStoredKeys(userId: string): Promise<void> {
  if (typeof indexedDB === 'undefined') return;
  const db = await openCryptoStore();
  await new Promise<void>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readwrite');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.delete(userId);
    req.onsuccess = () => resolve();
    req.onerror   = () => reject(req.error);
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// 5b. V2 — Key version & archive helpers
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Lit la version actuelle de la clé depuis IndexedDB.
 * Retourne 1 par défaut si le store est en format V1.
 */
export async function getCurrentKeyVersion(userId: string): Promise<number> {
  if (typeof indexedDB === 'undefined') return 1;
  const db = await openCryptoStore();
  const record = await new Promise<any | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });
  if (!record) return 1;
  if ('schemaVersion' in record && record.schemaVersion === 2) {
    return (record as StoredKeysV2).currentVersion;
  }
  return 1; // V1 legacy
}

/**
 * Met à jour les clés dans IndexedDB après une rotation.
 * Archive l'ancienne clé dans keyHistory, stocke la nouvelle.
 */
export async function saveKeyRotation(
  userId:          string,
  newPublicKey:    Uint8Array,
  newEncryptedPrivB64: string,
  oldEncryptedPrivB64: string | null,
  oldPublicKeyB64: string | null,
  newVersion:      number
): Promise<void> {
  if (typeof indexedDB === 'undefined') return;
  const na = await ensureSodium();
  const db = await openCryptoStore();

  // Load existing record
  const existing = await new Promise<StoredKeysV2 | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });

  const history: ArchivedKeyEntry[] = existing?.keyHistory ?? [];
  const passwordSalt = existing?.passwordSalt ?? na.to_base64(na.randombytes_buf(16), na.base64_variants.ORIGINAL);

  // Archive old key if provided
  if (oldEncryptedPrivB64 && oldPublicKeyB64) {
    history.push({
      version: newVersion - 1,
      publicKeyB64: oldPublicKeyB64,
      encryptedPrivateKeyB64: oldEncryptedPrivB64,
      createdAt: Date.now(),
    });
  }

  const v2record: StoredKeysV2 = {
    userId,
    schemaVersion: 2,
    currentVersion: newVersion,
    publicKeyB64: na.to_base64(newPublicKey, na.base64_variants.ORIGINAL),
    encryptedPrivKeyB64: newEncryptedPrivB64,
    keyHistory: history,
    passwordSalt,
  };

  await new Promise<void>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readwrite');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.put(v2record);
    req.onsuccess = () => resolve();
    req.onerror   = () => reject(req.error);
  });
}

/**
 * Retourne la liste des clés archivées (sans la clé privée déchiffrée).
 */
export async function getKeyHistoryFromStore(userId: string): Promise<ArchivedKeyEntry[]> {
  if (typeof indexedDB === 'undefined') return [];
  const db = await openCryptoStore();
  const record = await new Promise<StoredKeysV2 | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });
  if (!record || !('schemaVersion' in record) || record.schemaVersion !== 2) return [];
  return record.keyHistory ?? [];
}

/**
 * Récupère et déchiffre une clé privée archivée par version.
 * Retourne null si la version n'existe pas dans l'archive locale.
 */
export async function getArchivedPrivateKey(
  userId:   string,
  version:  number,
  password: string
): Promise<Uint8Array | null> {
  const history = await getKeyHistoryFromStore(userId);
  const entry = history.find(e => e.version === version);
  if (!entry) return null;
  try {
    return await decryptPrivateKey(entry.encryptedPrivateKeyB64, password);
  } catch {
    console.error('[crypto] Échec déchiffrement clé archivée version', version);
    return null;
  }
}

/**
 * Migre un store V1 vers V2 (ajoute currentVersion=1, keyHistory=[], passwordSalt).
 */
export async function migrateKeyStoreToV2(userId: string): Promise<void> {
  if (typeof indexedDB === 'undefined') return;
  const na = await ensureSodium();
  const db = await openCryptoStore();

  const existing = await new Promise<any | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });

  if (!existing) return;
  if ('schemaVersion' in existing && existing.schemaVersion === 2) return; // Already V2

  // V1 → V2 migration
  const v1 = existing as StoredKeys;
  const v2: StoredKeysV2 = {
    userId: v1.userId,
    schemaVersion: 2,
    currentVersion: 1,
    publicKeyB64: v1.publicKeyB64,
    encryptedPrivKeyB64: v1.encryptedPrivKeyB64,
    keyHistory: [],
    passwordSalt: na.to_base64(na.randombytes_buf(16), na.base64_variants.ORIGINAL),
  };

  await new Promise<void>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readwrite');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.put(v2);
    req.onsuccess = () => resolve();
    req.onerror   = () => reject(req.error);
  });

  console.info('[crypto] Store V1 migré vers V2 pour', userId);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4b. Archive helpers — chiffrement/déchiffrement avec passphrase dédié
//     Format identique à encryptPrivateKey: salt(16) || nonce(24) || ciphertext
//     L'archive B64 est auto-suffisante (le salt est préfixé).
//     Utile pour backup cross-device (Task 6) et export manuel.
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Chiffre la clé privée pour archivage/backup avec un passphrase dédié.
 * Retourne une chaîne base64 auto-suffisante : salt(16) || nonce(24) || ciphertext
 * Le salt est généré aléatoirement à chaque appel.
 */
export async function encryptPrivateKeyForArchive(
  privateKey: Uint8Array,
  archivePassphrase: string
): Promise<string> {
  return encryptPrivateKey(privateKey, archivePassphrase);
}

/**
 * Déchiffre une clé privée depuis un blob d'archive.
 * @param archiveB64  Le blob base64 (salt||nonce||ciphertext) produit par encryptPrivateKeyForArchive
 * @param passphrase  Le passphrase utilisé lors du chiffrement
 */
export async function decryptPrivateKeyFromArchive(
  archiveB64: string,
  passphrase: string
): Promise<Uint8Array> {
  return decryptPrivateKey(archiveB64, passphrase);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4c. decryptSessionKeyV2 — version-aware session key decryption
//     DT-05: si le déchiffrement avec la clé courante échoue pour cause de
//     rotation, tente avec la clé archivée correspondante.
// ─────────────────────────────────────────────────────────────────────────────

export interface DecryptSessionKeyV2Options {
  /** Version de la clé émettrice (lu depuis message_keys.sender_key_version) */
  senderKeyVersion?: number;
  /**
   * Fonction de rappel pour récupérer une clé privée archivée par version.
   * Appelée uniquement si le déchiffrement avec myPrivKey échoue
   * ET que senderKeyVersion est fourni.
   */
  archivedKeyLookup?: (version: number) => Promise<Uint8Array | null>;
}

export interface DecryptSessionKeyV2Result {
  sessionKey: Uint8Array;
  usedArchivedKey: boolean;
}

/**
 * Déchiffre la clé de session avec support de version de clé.
 * Stratégie :
 *   1. Tente le déchiffrement avec myPrivKey (clé courante).
 *   2. En cas d'échec et si senderKeyVersion + archivedKeyLookup sont fournis,
 *      tente de récupérer la clé archivée et réessaye.
 *   3. Si les deux échouent, lève une exception.
 */
export async function decryptSessionKeyV2(
  encKeyB64:    string,
  senderPubB64: string,
  myPrivKey:    Uint8Array,
  options?:     DecryptSessionKeyV2Options
): Promise<DecryptSessionKeyV2Result> {
  // 1. Try current key first
  try {
    const sessionKey = await decryptSessionKey(encKeyB64, senderPubB64, myPrivKey);
    return { sessionKey, usedArchivedKey: false };
  } catch (err) {
    // If no version info or lookup function, rethrow
    if (!options?.senderKeyVersion || !options?.archivedKeyLookup) {
      throw err;
    }
    // 2. Try archived key
    console.info('[crypto] decryptSessionKeyV2: fallback to archived key version', options.senderKeyVersion);
    const archivedPriv = await options.archivedKeyLookup(options.senderKeyVersion);
    if (!archivedPriv) {
      throw new Error(
        `[crypto] decryptSessionKeyV2: archived key version ${options.senderKeyVersion} not found`
      );
    }
    const sessionKey = await decryptSessionKey(encKeyB64, senderPubB64, archivedPriv);
    return { sessionKey, usedArchivedKey: true };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Clés "pending" (localStorage) — utilisées entre l'inscription et
//    le premier changement de mot de passe.
//    La clé privée est en clair temporairement (< 1 session).
// ─────────────────────────────────────────────────────────────────────────────

export function storePendingKeys(
  memberId:   string,
  publicKey:  Uint8Array,
  privateKey: Uint8Array
): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(
    `nook_pending_keys_${memberId}`,
    JSON.stringify({
      publicKey:  Array.from(publicKey),
      privateKey: Array.from(privateKey),
    })
  );
}

export function getPendingKeys(
  memberId: string
): { publicKey: Uint8Array; privateKey: Uint8Array } | null {
  if (typeof localStorage === 'undefined') return null;
  const raw = localStorage.getItem(`nook_pending_keys_${memberId}`);
  if (!raw) return null;
  try {
    const data = JSON.parse(raw);
    return {
      publicKey:  new Uint8Array(data.publicKey),
      privateKey: new Uint8Array(data.privateKey),
    };
  } catch {
    return null;
  }
}

export function clearPendingKeys(memberId: string): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.removeItem(`nook_pending_keys_${memberId}`);
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. API helper — enregistrer la clé publique sur le serveur
// ─────────────────────────────────────────────────────────────────────────────
export async function registerPublicKeyOnServer(publicKey: Uint8Array): Promise<void> {
  const na = await ensureSodium();
  const b64 = na.to_base64(publicKey, na.base64_variants.ORIGINAL);
  const res = await fetch('/api/auth/public-key', {
    method:      'POST',
    credentials: 'include',
    headers:     { 'Content-Type': 'application/json' },
    body:        JSON.stringify({ public_key: b64 }),
  });
  if (!res.ok) {
    throw new Error(`[crypto] registerPublicKey: HTTP ${res.status}`);
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. API helper — récupérer les clés publiques des membres d'une conversation
// ─────────────────────────────────────────────────────────────────────────────
export async function fetchMemberPubkeys(
  conversationId: string
): Promise<Record<string, string>> {
  const res = await fetch(
    `/api/auth/public-keys?conversation_id=${encodeURIComponent(conversationId)}`,
    { credentials: 'include' }
  );
  if (!res.ok) throw new Error(`[crypto] fetchMemberPubkeys: HTTP ${res.status}`);
  const members: { user_id: string; public_key: string }[] = await res.json();
  return Object.fromEntries(members.map(m => [m.user_id, m.public_key]));
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. API helper — rotation de clé côté serveur
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Appelle POST /api/auth/rotate-key pour archiver l'ancienne clé sur le serveur
 * et enregistrer la nouvelle.
 */
export async function rotateKeyOnServer(
  newPublicKeyB64:     string,
  newEncryptedPrivB64: string,
  password:            string
): Promise<{ success: boolean; version: number }> {
  const res = await fetch('/api/auth/rotate-key', {
    method:      'POST',
    credentials: 'include',
    headers:     { 'Content-Type': 'application/json' },
    body:        JSON.stringify({
      public_key: newPublicKeyB64,
      encrypted_private_key: newEncryptedPrivB64,
      password,
    }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`[crypto] rotateKey: HTTP ${res.status} — ${text}`);
  }
  return res.json();
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. API helpers — key history
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Récupère l'historique des clés depuis le serveur.
 */
export async function fetchKeyHistoryFromServer(): Promise<{
  version: number;
  public_key: string;
  created_at: number;
  revoked_at: number | null;
}[]> {
  const res = await fetch('/api/auth/key-history', { credentials: 'include' });
  if (!res.ok) throw new Error(`[crypto] key-history: HTTP ${res.status}`);
  return res.json();
}

/**
 * Récupère une clé privée archivée depuis le serveur.
 */
export async function fetchArchivedPrivateKeyFromServer(version: number): Promise<string | null> {
  const res = await fetch(`/api/auth/key-history/${version}`, { credentials: 'include' });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`[crypto] key-history/${version}: HTTP ${res.status}`);
  const data = await res.json();
  return (data as { encrypted_private_key: string }).encrypted_private_key;
}
