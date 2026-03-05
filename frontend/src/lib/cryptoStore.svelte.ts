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
      // ── Premier setup E2EE pour cet utilisateur ──────────────────────────
      // Aucune clé trouvée dans IndexedDB : génération initiale transparente.
      // Cela couvre :
      //   • Tout utilisateur approuvé se connectant pour la première fois
      //   • L'administrateur initial (qui ne passe plus par join/+page.svelte)
      //   • Un utilisateur dont les clés ont été effacées (clearStoredKeys)
      console.info('[cryptoStore] Aucune clé en IndexedDB → génération initiale E2EE');

      // 1. Générer la paire de clés Curve25519
      const newKeyPair = await generateKeyPair();

      // 2. Chiffrer la clé privée avec le mot de passe (XSalsa20+Argon2)
      const encryptedPrivKey = await encryptPrivateKey(newKeyPair.privateKey, password);

      // 3. Stocker dans IndexedDB (clé publique en clair, privée chiffrée)
      await storeKeysInIndexedDB(userId, newKeyPair.publicKey, encryptedPrivKey);

      // 4. Envoyer la clé publique au serveur (endpoint /api/e2ee/register-key)
      //    → les autres membres peuvent maintenant chiffrer des messages pour nous
      await registerPublicKeyOnServer(newKeyPair.publicKey);

      kp = newKeyPair;
      console.info('[cryptoStore] Clé E2EE générée, chiffrée, stockée et enregistrée ✓');
    }

    // kp est garanti non-null ici (chargé ou généré)
    _keyPair           = kp;
    cryptoStore.userId = userId;
    cryptoStore.ready  = true;
    return true;

  } catch (e: any) {
    // Causes possibles :
    //   • Mot de passe incorrect → libsodium lève une exception au déchiffrement
    //   • Échec réseau lors de registerPublicKeyOnServer
    //   • IndexedDB inaccessible (mode privé sur certains navigateurs)
    const msg = e?.message ?? String(e);
    if (msg.includes('register') || msg.includes('HTTP')) {
      cryptoStore.error = 'Erreur réseau lors de l\'enregistrement des clés. Réessayez.';
    } else {
      cryptoStore.error = 'Mot de passe incorrect ou clés corrompues.';
    }
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
