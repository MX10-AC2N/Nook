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
  decryptContent,
  fetchMemberPubkeys,
  type KeyPair,
  type EncryptedMessage,
} from '$lib/crypto';

// ─────────────────────────────────────────────────────────────────────────────
// State réactif (Svelte 5 Runes)
// ─────────────────────────────────────────────────────────────────────────────
interface CryptoStoreState {
  ready:    boolean;
  error:    string | null;
  userId:   string | null;
}

export const cryptoStore = $state<CryptoStoreState>({
  ready:  false,
  error:  null,
  userId: null,
});

// Clé privée en mémoire uniquement — jamais sérialisée, jamais exportée
let _keyPair: KeyPair | null = null;

// ─────────────────────────────────────────────────────────────────────────────
// Restauration automatique depuis sessionStorage (reload sans mot de passe)
// ─────────────────────────────────────────────────────────────────────────────
(function restoreFromSessionStorage() {
  try {
    const encPriv = sessionStorage.getItem('nook_privkey');
    const encPub = sessionStorage.getItem('nook_pubkey');
    const uid = sessionStorage.getItem('nook_userid');
    if (encPriv && encPub && uid) {
      const priv = Uint8Array.from(atob(encPriv), c => c.charCodeAt(0));
      const pub = Uint8Array.from(atob(encPub), c => c.charCodeAt(0));
      _keyPair = { privateKey: priv, publicKey: pub };
      cryptoStore.userId = uid;
      cryptoStore.ready = true;
      console.log('[cryptoStore] Clés restaurées depuis sessionStorage (session sans mot de passe)');
    }
  } catch (e) {
    console.warn('[cryptoStore] Échec restauration sessionStorage:', e);
  }
})();

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
  cryptoStore.error  = null;
  cryptoStore.ready  = false;
  cryptoStore.userId = null;
  _keyPair           = null;

  try {
    let kp = await loadKeysFromIndexedDB(userId, password);

    if (!kp) {
      const keysExist = await hasStoredKeys(userId);

      if (keysExist) {
        // Keys exist but failed to load (wrong password, corrupted data)
        cryptoStore.error = 'Clés E2EE existent mais impossibles à déchiffrer — vérifiez votre mot de passe ou régénérez les clés.';
        console.error('[cryptoStore] Keys exist but load failed for userId:', userId);
        return false;
      }
      
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

      // Persister les clés E2EE en sessionStorage (volatile) pour restauration sans mot de passe
      try {
        sessionStorage.setItem('nook_privkey', btoa(String.fromCharCode(..._keyPair.privateKey)));
        sessionStorage.setItem('nook_pubkey', btoa(String.fromCharCode(..._keyPair.publicKey)));
        sessionStorage.setItem('nook_userid', userId);
      } catch (e) {
        console.warn('[cryptoStore] Impossible de stocker les clés en sessionStorage:', e);
      }

      console.info('[cryptoStore] Première paire de clés générée et activée ✓');
      return true;
    }

    // kp est garanti non-null ici (chargé depuis IndexedDB)

    // Persister les clés E2EE en sessionStorage (volatile) pour restauration sans mot de passe
    try {
      sessionStorage.setItem('nook_privkey', btoa(String.fromCharCode(...kp.privateKey)));
      sessionStorage.setItem('nook_pubkey', btoa(String.fromCharCode(...kp.publicKey)));
      sessionStorage.setItem('nook_userid', userId);
    } catch (e) {
      console.warn('[cryptoStore] sessionStorage:', e);
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
    console.info('[cryptoStore] unlockCrypto DONE — ready=true');

  } catch (e: any) {
    // Seules vraies erreurs : mot de passe incorrect ou IndexedDB inaccessible
    const msg = e?.message ?? String(e);
    cryptoStore.error = msg.includes('IndexedDB') || msg.includes('storage')
      ? 'Stockage local inaccessible (mode privé ?).'
      : 'Clés inaccessibles — vérifiez votre mot de passe.';
    console.error('[cryptoStore] unlock:', e);
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
    registerPublicKeyOnServer(newKeyPair.publicKey)
      .then(() => console.info('[cryptoStore] Nouvelle clé publique enregistrée sur le serveur ✓'))
      .catch((e) => console.warn('[cryptoStore] Échec enregistrement clé publique :', e?.message));

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
// ─────────────────────────────────────────────────────────────────────────────
export async function encryptMessage(
  plaintext:      string,
  conversationId: string
): Promise<EncryptedMessage> {
  if (!_keyPair) throw new Error('[cryptoStore] Clés non chargées — appelez unlockCrypto() d\'abord.');
  const pubkeys = await fetchMemberPubkeys(conversationId);
  return encryptForRecipients(plaintext, pubkeys, _keyPair);
}

// ─────────────────────────────────────────────────────────────────────────────
// decryptMessage — déchiffre un message reçu
// ─────────────────────────────────────────────────────────────────────────────
export async function decryptMessage(params: {
  messageId:       string;
  conversationId:  string;
  ciphertext:      string;
  nonce:           string;
  senderPubkeyB64: string;
}): Promise<string> {
  if (!_keyPair)           throw new Error('[cryptoStore] Clés non chargées.');
  if (!cryptoStore.userId) throw new Error('[cryptoStore] userId absent.');

  const res = await fetch(
    `/api/conversations/${params.conversationId}/my-encrypted-key/${params.messageId}`,
    { credentials: 'include' }
  );
  if (!res.ok) throw new Error(`[cryptoStore] get encrypted key: HTTP ${res.status}`);
  const { encrypted_key } = await res.json();

  const sessionKey = await decryptSessionKey(
    encrypted_key,
    params.senderPubkeyB64,
    _keyPair.privateKey
  );
  return decryptContent(params.ciphertext, params.nonce, sessionKey);
}

// ─────────────────────────────────────────────────────────────────────────────
// getPublicKey — expose la clé publique (pas la privée)
// ─────────────────────────────────────────────────────────────────────────────
export function getPublicKey(): Uint8Array | null {
  return _keyPair?.publicKey ?? null;
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
