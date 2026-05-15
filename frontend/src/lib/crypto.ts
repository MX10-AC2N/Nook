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
    const recipientPub = na.from_base64(pubKeyB64, na.base64_variants.ORIGINAL);
    // console.log('[encryptForRecipients] userId:', userId, 'pubKeyBytes:', recipientPub.length, 'privKeyBytes:', senderKeyPair.privateKey.length);
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
    // console.log('[encryptForRecipients] encryptedKeys[' + userId + '] len:', encryptedKeys[userId].length);
  }

  // console.log('[encryptForRecipients] FINAL encryptedKeys count:', Object.keys(encryptedKeys).length);
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
 * Retourne une chaîne base64 : salt(32) || nonce(24) || ciphertext
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

  // Derive key using BLAKE2b (crypto_pwhash not available in this build)
  const passwordBytes = new TextEncoder().encode(password);
  const saltedPw = new Uint8Array(passwordBytes.length + salt.length);
  saltedPw.set(passwordBytes);
  saltedPw.set(salt, passwordBytes.length);
  const key = na.crypto_generichash(na.crypto_secretbox_KEYBYTES, saltedPw);

  return na.crypto_secretbox_open_easy(ciphertext, nonce, key);
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. IndexedDB — stockage des clés chiffrées
// ─────────────────────────────────────────────────────────────────────────────

const IDB_NAME    = 'NookCrypto';
const IDB_VERSION = 1;
const IDB_STORE   = 'keys';

function openCryptoStore(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, IDB_VERSION);
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(IDB_STORE)) {
        req.result.createObjectStore(IDB_STORE, { keyPath: 'userId' });
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
 */
export async function storeKeysInIndexedDB(
  userId:              string,
  publicKey:           Uint8Array,
  encryptedPrivKeyB64: string
): Promise<void> {
  if (typeof indexedDB === 'undefined') return;
  const na = await ensureSodium();
  const db = await openCryptoStore();
  await new Promise<void>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readwrite');
    const store = tx.objectStore(IDB_STORE);
    const record: StoredKeys = {
      userId,
      publicKeyB64:        na.to_base64(publicKey, na.base64_variants.ORIGINAL),
      encryptedPrivKeyB64,
    };
    const req = store.put(record);
    req.onsuccess = () => resolve();
    req.onerror   = () => reject(req.error);
  });
}

/**
 * Charge les clés chiffrées depuis IndexedDB et les déchiffre avec le mot de passe.
 * Retourne null si aucune clé n'est stockée pour cet utilisateur.
 */
export async function loadKeysFromIndexedDB(
  userId:   string,
  password: string
): Promise<KeyPair | null> {
  if (typeof indexedDB === 'undefined') return null;
  const na = await ensureSodium();

  const db = await openCryptoStore();
  const record = await new Promise<StoredKeys | undefined>((resolve, reject) => {
    const tx    = db.transaction(IDB_STORE, 'readonly');
    const store = tx.objectStore(IDB_STORE);
    const req   = store.get(userId);
    req.onsuccess = () => resolve(req.result as StoredKeys | undefined);
    req.onerror   = () => reject(req.error);
  });

  if (!record) return null;

  const privateKey = await decryptPrivateKey(record.encryptedPrivKeyB64, password);
  const publicKey  = na.from_base64(record.publicKeyB64, na.base64_variants.ORIGINAL);
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
