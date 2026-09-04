// src/lib/cryptoStore.svelte.ts
//
// Store Svelte 5 Runes — conserve les clés en mémoire après déchiffrement.
// N'expose JAMAIS la clé privée en dehors de ce module.
//
// Flux E2EE :
//   Premier login (aucune clé en IndexedDB)
//     → unlockCrypto génère la paire, chiffre la privée, stocke dans IndexedDB,
//       envoie la publique au serveur → cryptoStore.ready = true
//
//   Login suivant (clés déjà en IndexedDB)
//     → unlockCrypto déchiffre avec le mot de passe → cryptoStore.ready = true
//
//   Mot de passe incorrect
//     → libsodium lève une exception → cryptoStore.error = message explicite
//
// API publique :
//   cryptoStore.ready          → boolean réactif
//   cryptoStore.error          → string | null
//   cryptoStore.userId         → string | null
//   await unlockCrypto(userId, password) → boolean
//   lockCrypto()               → efface les clés de la mémoire
//   await encryptMessage(text, convId)
//   await decryptMessage(params)
//   getPublicKey()             → Uint8Array | null

import {
  generateKeyPair,
  encryptPrivateKey,
  storeKeysInIndexedDB,
  registerPublicKeyOnServer,
  loadKeysFromIndexedDB,
  hasStoredKeys,
  clearStoredKeys,
  encryptForRecipients,
  decryptSessionKey,
  decryptSessionKeyV2,
  decryptContent,
  fetchMemberPubkeys,
  getCurrentKeyVersion,
  migrateKeyStoreToV2,
  getArchivedPrivateKey,
  decryptPrivateKey,
  decryptWithGroupKey,
  encryptWithGroupKey,
  type KeyPair,
  type EncryptedMessage,
  type DecryptSessionKeyV2Options,
  type DecryptSessionKeyV2Result,
} from '$lib/crypto';
import { e2ee } from '$lib/e2ee';

// ─────────────────────────────────────────────────────────────────────────────
// State réactif (Svelte 5 Runes)
// ─────────────────────────────────────────────────────────────────────────────
interface CryptoStoreState {
  ready:    boolean;
  error:    string | null;
  userId:   string | null;
  currentKeyVersion: number;
}

export const cryptoStore = $state<CryptoStoreState>({
  ready:  false,
  error:  null,
  userId: null,
  currentKeyVersion: 1,
});

// Clé privée en mémoire uniquement — jamais sérialisée, jamais exportée
let _keyPair: KeyPair | null = null;

// ─────────────────────────────────────────────────────────────────────────────
// Restauration depuis localStorage (appelée explicitement côté client)
// ─────────────────────────────────────────────────────────────────────────────
export function restoreFromLocalStorage(): boolean {
  if (typeof localStorage === 'undefined') return false;
  // Garde anti état fantôme : ready=true mais _keyPair=null → on nettoie
  if (cryptoStore.ready && !_keyPair) {
    console.warn('[cryptoStore] Ghost ready state detected — resetting');
    cryptoStore.ready = false;
    localStorage.removeItem('nook_privkey');
    localStorage.removeItem('nook_pubkey');
    localStorage.removeItem('nook_userid');
  }
  if (cryptoStore.ready && _keyPair) {
    console.log('[cryptoStore] restoreFromLocalStorage skip — déjà ready');
    return true;
  }
  try {
    const encPriv = localStorage.getItem('nook_privkey');
    const encPub = localStorage.getItem('nook_pubkey');
    const uid = localStorage.getItem('nook_userid');
    if (encPriv && encPub && uid) {
      const priv = Uint8Array.from(atob(encPriv), c => c.charCodeAt(0));
      const pub = Uint8Array.from(atob(encPub), c => c.charCodeAt(0));
      // X25519 clés font 32 bytes — rejeter données corrompues
      if (priv.length !== 32 || pub.length !== 32) throw new Error('Clés de taille invalide');
      _keyPair = { privateKey: priv, publicKey: pub };
      cryptoStore.userId = uid;
      cryptoStore.ready = true;
      console.log('[cryptoStore] Clés restaurées depuis localStorage');
      return true;
    }
  } catch (e) {
    console.warn('[cryptoStore] Échec restauration localStorage:', e);
    localStorage.removeItem('nook_privkey');
    localStorage.removeItem('nook_pubkey');
    localStorage.removeItem('nook_userid');
  }
  return false;
}

// ─────────────────────────────────────────────────────────────────────────────
// Restauration depuis sessionStorage (fallback, rapide si disponible)
// ─────────────────────────────────────────────────────────────────────────────
export function restoreFromSessionStorage(): boolean {
  if (typeof sessionStorage === 'undefined') return false;
  // Garde anti état fantôme : ready=true mais _keyPair=null → on nettoie
  if (cryptoStore.ready && !_keyPair) {
    console.warn('[cryptoStore] Ghost ready state detected — resetting');
    cryptoStore.ready = false;
    sessionStorage.removeItem('nook_privkey');
    sessionStorage.removeItem('nook_pubkey');
    sessionStorage.removeItem('nook_userid');
  }
  if (cryptoStore.ready && _keyPair) {
    console.log('[cryptoStore] restoreFromSessionStorage skip — déjà ready');
    return true;
  }
  try {
    const encPriv = sessionStorage.getItem('nook_privkey');
    const encPub = sessionStorage.getItem('nook_pubkey');
    const uid = sessionStorage.getItem('nook_userid');
    if (encPriv && encPub && uid) {
      const priv = Uint8Array.from(atob(encPriv), c => c.charCodeAt(0));
      const pub = Uint8Array.from(atob(encPub), c => c.charCodeAt(0));
      // X25519 clés font 32 bytes — rejeter données corrompues
      if (priv.length !== 32 || pub.length !== 32) throw new Error('Clés de taille invalide');
      _keyPair = { privateKey: priv, publicKey: pub };
      cryptoStore.userId = uid;
      cryptoStore.ready = true;
      console.log('[cryptoStore] Clés restaurées depuis sessionStorage');
      return true;
    }
  } catch (e) {
    console.warn('[cryptoStore] Échec restauration sessionStorage:', e);
    sessionStorage.removeItem('nook_privkey');
    sessionStorage.removeItem('nook_pubkey');
    sessionStorage.removeItem('nook_userid');
  }
  return false;
}

// ─────────────────────────────────────────────────────────────────────────────
// unlockCrypto — appeler après login avec le mot de passe en clair
//
// Comportement :
//   1. Cherche les clés dans IndexedDB (via loadKeysFromIndexedDB)
//   2. Si trouvées → déchiffre avec le mot de passe, active le store
//   3. Si absentes → génération initiale : crée la paire, chiffre la privée,
//      stocke dans IndexedDB, envoie la publique au serveur, active le store
//   4. Si déchiffrement échoue (mauvais mot de passe) → exception catch → error
// ─────────────────────────────────────────────────────────────────────────────
export async function unlockCrypto(userId: string, password: string): Promise<boolean> {
  console.log('[cryptoStore] unlockCrypto called for userId:', userId);
  // Garde anti état fantôme : ready=true mais _keyPair=null → on nettoie
  if (cryptoStore.ready && !_keyPair) {
    console.warn('[cryptoStore] Ghost ready state detected in unlockCrypto — resetting');
    cryptoStore.ready = false;
    cryptoStore.userId = null;
    cryptoStore.error = null;
  }
  // Déjà déverrouillé via sessionStorage? skip pour éviter regen de clés
  if (cryptoStore.ready && _keyPair) {
    console.log('[cryptoStore] unlockCrypto skip — déjà ready');
    return true;
  }
  cryptoStore.error  = null;
  cryptoStore.ready  = false;
  cryptoStore.userId = null;
  _keyPair           = null;

  try {
    console.log('[cryptoStore] loading keys from IndexedDB...');
    let kp = await loadKeysFromIndexedDB(userId, password);

    if (!kp) {
      console.log('[cryptoStore] no keys loaded from IndexedDB');
      const keysExist = await hasStoredKeys(userId);
      console.log('[cryptoStore] keysExist:', keysExist);

      if (keysExist) {
        // Keys exist but failed to load (wrong password, corrupted data)
        cryptoStore.error = 'Clés E2EE existent mais impossibles à déchiffrer — vérifiez votre mot de passe ou régénérez les clés.';
        console.error('[cryptoStore] Keys exist but load failed for userId:', userId);
        return false;
      }
      // ...
      // ── Premier setup E2EE pour cet utilisateur (aucune clé trouvée) ─────
      console.info('[cryptoStore] Premier setup E2EE — génération paire de clés');

      // 1. Générer la paire de clés Curve25519
      const newKeyPair = await generateKeyPair();

      // 2. Chiffrer la clé privée avec le mot de passe
      const encryptedPrivKey = await encryptPrivateKey(newKeyPair.privateKey, password);

      // 3. Stocker dans IndexedDB
      await storeKeysInIndexedDB(userId, newKeyPair.publicKey, encryptedPrivKey);

      // 4. Enregistrer la clé publique sur le serveur AVANT d'activer le store
      //    pour éviter une race où les messages sont envoyés avec une clé non synchronisée
      await registerPublicKeyOnServer(newKeyPair.publicKey);

      // 5. Activer le store maintenant que la clé publique est sur le serveur
      _keyPair           = newKeyPair;
      cryptoStore.userId = userId;
      cryptoStore.ready  = true;

      // Persister les clés E2EE en sessionStorage (cache rapide)
      try {
        sessionStorage.setItem('nook_privkey', btoa(String.fromCharCode(...newKeyPair.privateKey)));
        sessionStorage.setItem('nook_pubkey', btoa(String.fromCharCode(...newKeyPair.publicKey)));
        sessionStorage.setItem('nook_userid', userId);
      } catch (e) {
        console.warn('[cryptoStore] Impossible de stocker les clés en sessionStorage:', e);
      }
      // Persister aussi en localStorage (survit fermeture navigateur)
      try {
        localStorage.setItem('nook_privkey', btoa(String.fromCharCode(...newKeyPair.privateKey)));
        localStorage.setItem('nook_pubkey', btoa(String.fromCharCode(...newKeyPair.publicKey)));
        localStorage.setItem('nook_userid', userId);
      } catch (e) {
        console.warn('[cryptoStore] Impossible de stocker les clés en localStorage:', e);
      }

      console.info('[cryptoStore] Première paire de clés générée et activée ✓');
      return true;
    }

    // kp est garanti non-null ici (chargé depuis IndexedDB)

    // Persister les clés E2EE en sessionStorage (cache rapide)
    try {
      sessionStorage.setItem('nook_privkey', btoa(String.fromCharCode(...kp.privateKey)));
      sessionStorage.setItem('nook_pubkey', btoa(String.fromCharCode(...kp.publicKey)));
      sessionStorage.setItem('nook_userid', userId);
    } catch (e) {
      console.warn('[cryptoStore] sessionStorage:', e);
    }
    // Persister aussi en localStorage (survit fermeture navigateur)
    try {
      localStorage.setItem('nook_privkey', btoa(String.fromCharCode(...kp.privateKey)));
      localStorage.setItem('nook_pubkey', btoa(String.fromCharCode(...kp.publicKey)));
      localStorage.setItem('nook_userid', userId);
    } catch (e) {
      console.warn('[cryptoStore] localStorage:', e);
    }

    // Await public key registration BEFORE activating the store
    // to prevent sending messages with a pubkey not yet on the server
    if (kp.publicKey) {
      await registerPublicKeyOnServer(kp.publicKey);
      console.info('[cryptoStore] Clé publique synchronisée ✓');
    }

    // Activer le store maintenant que la clé publique est garantie sur le serveur
    _keyPair           = kp;
    cryptoStore.userId = userId;
    cryptoStore.ready  = true;

    // Load current key version from IndexedDB (V2 format)
    try {
      const kv = await getCurrentKeyVersion(userId);
      cryptoStore.currentKeyVersion = kv;
    } catch {
      cryptoStore.currentKeyVersion = 1;
    }
    // Background migrate V1→V2 if needed
    migrateKeyStoreToV2(userId).catch(() => {});

    console.info('[cryptoStore] unlockCrypto DONE — ready=true', { keyVersion: cryptoStore.currentKeyVersion });

  } catch (e: any) {
    // Seules vraies erreurs : mot de passe incorrect ou IndexedDB inaccessible
    const msg = e?.message ?? String(e);
    cryptoStore.error = msg.includes('IndexedDB') || msg.includes('storage')
      ? 'Stockage local inaccessible (mode privé ?).'
      : 'Clés inaccessibles — vérifiez votre mot de passe.';
    console.error('[cryptoStore] unlock:', e);
    // Garde de sécurité : si on arrive ici, ready doit être false et _keyPair null
    cryptoStore.ready = false;
    _keyPair = null;
    return false;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// lockCrypto — efface les clés de la mémoire (logout)
// ─────────────────────────────────────────────────────────────────────────────
export function lockCrypto(): void {
  _keyPair           = null;
  cryptoStore.ready  = false;
  cryptoStore.userId = null;
  cryptoStore.error  = null;
  // Note: on ne supprime PAS d'IndexedDB (les clés doivent persister entre sessions)
}

// ─────────────────────────────────────────────────────────────────────────────
// resetCrypto — force la régénération d'une nouvelle paire de clés.
//   À utiliser quand les clés sont corrompues ou ne correspondent plus.
//   ATTENTION : rend les vieux messages illisibles !
// ─────────────────────────────────────────────────────────────────────────────
export async function resetCrypto(userId: string, password: string): Promise<boolean> {
  cryptoStore.error  = null;
  cryptoStore.ready  = false;
  cryptoStore.userId = null;
  _keyPair           = null;

  try {
    // 1. Supprimer les anciennes clés d'IndexedDB
    await clearStoredKeys(userId);
    console.info('[cryptoStore] Anciennes clés supprimées d\'IndexedDB');

    // 2. Générer une nouvelle paire de clés
    const newKeyPair = await generateKeyPair();

    // 3. Chiffrer la clé privée avec le mot de passe
    const encryptedPrivKey = await encryptPrivateKey(newKeyPair.privateKey, password);

    // 4. Stocker dans IndexedDB
    await storeKeysInIndexedDB(userId, newKeyPair.publicKey, encryptedPrivKey);

    // 5. Activer le store
    _keyPair           = newKeyPair;
    cryptoStore.userId = userId;
    cryptoStore.ready  = true;

    // 6. Enregistrer la nouvelle clé publique sur le serveur
    // BUG-005 FIX: await pour garantir l'enregistrement avant tout envoi de message
    await registerPublicKeyOnServer(newKeyPair.publicKey);
    console.info('[cryptoStore] Nouvelle clé publique enregistrée sur le serveur ✓');

    console.info('[cryptoStore] Nouvelle clé E2EE générée et activée ✓');
    return true;

  } catch (e: any) {
    const msg = e?.message ?? String(e);
    cryptoStore.error = 'Échec de la régénération des clés : ' + msg;
    console.error('[cryptoStore] resetCrypto:', e);
    return false;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// encryptMessage — chiffre un message pour tous les membres d'une conversation
// Pour default_global (groupe global), utilise la group key
// ─────────────────────────────────────────────────────────────────────────────
export async function encryptMessage(
  plaintext:      string,
  conversationId: string
): Promise<EncryptedMessage | { ciphertext: string; nonce: string; group_key_version: number }> {
  if (!_keyPair) throw new Error('[cryptoStore] Clés non chargées — appelez unlockCrypto() d\'abord.');

  // Pour default_global, utiliser la group key
  if (conversationId === 'default_global') {
    const groupKey = await e2ee.loadGroupKey(conversationId);
    const version = e2ee.currentVersion(conversationId);
    const result = await encryptWithGroupKey(plaintext, groupKey);
    return { ...result, group_key_version: version };
  }

  const pubkeys = await fetchMemberPubkeys(conversationId);
  console.info('[cryptoStore] encryptMessage conv:', conversationId, 'destinataires:', Object.keys(pubkeys).length, 'mes clés:', !!_keyPair);
  return encryptForRecipients(plaintext, pubkeys, _keyPair);
}

// ─────────────────────────────────────────────────────────────────────────────
// decryptMessage — déchiffre un message reçu
// Supporte 2 formats :
//   (a) Ancien : nonce + encrypted_keys (HashMap) -> decryptSessionKeyV2
//   (b) Nouveau : nonce + group_key_version -> déchiffrement avec group key
// ─────────────────────────────────────────────────────────────────────────────
export async function decryptMessage(params: {
  messageId:       string;
  conversationId:  string;
  ciphertext:      string;
  nonce:           string;
  senderPubkeyB64: string;
  senderKeyVersion?: number;
  groupKeyVersion?: number;
}): Promise<string> {
  if (!_keyPair)           throw new Error('[cryptoStore] Clés non chargées.');
  if (!cryptoStore.userId) throw new Error('[cryptoStore] userId absent.');

  // Format nouveau : group_key_version présent -> déchiffrement avec group key
  if (params.groupKeyVersion !== undefined && params.groupKeyVersion !== null) {
    const groupKey = await e2ee.loadGroupKey(params.conversationId);
    return decryptWithGroupKey(params.ciphertext, params.nonce, groupKey);
  }

  // Format ancien : encrypted_keys -> déchiffrement par destinataire
  const res = await fetch(
    `/api/conversations/${params.conversationId}/my-encrypted-key/${params.messageId}`,
    { credentials: 'include' }
  );
  if (!res.ok) throw new Error(`[cryptoStore] get encrypted key: HTTP ${res.status}`);
  const { encrypted_key } = await res.json();

  // Archived key lookup — DT-05-bis: forward secrecy after X25519 rotation.
  const sessionPwd = (typeof sessionStorage !== 'undefined')
    ? (sessionStorage.getItem('nook_crypto_key') || localStorage.getItem('nook_crypto_key'))
    : null;

  const archivedKeyLookup = params.senderKeyVersion
    ? async (version: number): Promise<Uint8Array | null> => {
        if (cryptoStore.userId && sessionPwd) {
          try {
            const local = await getArchivedPrivateKey(cryptoStore.userId, version, sessionPwd);
            if (local) return local;
          } catch (e) {
            console.warn('[cryptoStore] archivedKeyLookup local archive miss:', e);
          }
        }
        try {
          const res = await fetch(`/api/auth/key-history/${version}`, { credentials: 'include' });
          if (!res.ok) {
            console.warn('[cryptoStore] archivedKeyLookup: HTTP', res.status, 'for version', version);
            return null;
          }
          const data = await res.json() as { encrypted_private_key?: string };
          const encryptedPriv = data.encrypted_private_key;
          if (!encryptedPriv) return null;
          if (!sessionPwd) {
            console.warn('[cryptoStore] archivedKeyLookup: session password unavailable — cannot decrypt');
            return null;
          }
          return await decryptPrivateKey(encryptedPriv, sessionPwd);
        } catch (e) {
          console.error('[cryptoStore] archivedKeyLookup failed:', e);
          return null;
        }
      }
    : undefined;

  const result = await decryptSessionKeyV2(
    encrypted_key,
    params.senderPubkeyB64,
    _keyPair.privateKey,
    {
      senderKeyVersion: params.senderKeyVersion,
      archivedKeyLookup,
    }
  );
  return decryptContent(params.ciphertext, params.nonce, result.sessionKey);
}

// ─────────────────────────────────────────────────────────────────────────────
// getPublicKey — expose la clé publique (pas la privée)
// ─────────────────────────────────────────────────────────────────────────────
export function getPublicKey(): Uint8Array | null {
  return _keyPair?.publicKey ?? null;
}

// ─────────────────────────────────────────────────────────────────────────────
// hasKeys — vérifie si les clés E2EE sont chargées en mémoire
// ─────────────────────────────────────────────────────────────────────────────
export function hasKeys(): boolean {
  return _keyPair !== null;
}

// ─────────────────────────────────────────────────────────────────────────────
// reRegisterPublicKey — force la synchronisation de la clé publique avec le serveur
// Utile si la clé n'a pas été enregistrée correctement lors du login
// ─────────────────────────────────────────────────────────────────────────────
export async function reRegisterPublicKey(): Promise<boolean> {
  if (!_keyPair) {
    console.warn('[cryptoStore] Pas de clé pour ré-enregistrement');
    return false;
  }
  
  try {
    await registerPublicKeyOnServer(_keyPair.publicKey);
    console.info('[cryptoStore] Clé publique ré-enregistrée sur le serveur ✓');
    return true;
  } catch (e) {
    console.error('[cryptoStore] Échec ré-enregistrement clé publique:', e);
    return false;
  }
}
