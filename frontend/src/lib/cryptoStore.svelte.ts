// src/lib/cryptoStore.svelte.ts
//
// Store Svelte 5 Runes — conserve les clés en mémoire après déchiffrement.
// N'expose JAMAIS la clé privée en dehors de ce module.
//
// Usage :
//   cryptoStore.ready          → boolean réactif
//   cryptoStore.error          → string | null
//   await cryptoStore.unlock(userId, password)
//   await cryptoStore.encryptAndSend(text, convId)
//   await cryptoStore.decryptMessage(msg)
//   cryptoStore.lock()         → efface les clés de la mémoire

import {
  loadKeysFromIndexedDB,
  encryptForRecipients,
  decryptSessionKey,
  decryptContent,
  fetchMemberPubkeys,
  type KeyPair,
  type EncryptedMessage,
} from '$lib/crypto';

// ─────────────────────────────────────────────────────────────────────────────
// State
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
// unlock — appeler après login avec le mot de passe
// ─────────────────────────────────────────────────────────────────────────────
export async function unlockCrypto(userId: string, password: string): Promise<boolean> {
  cryptoStore.error  = null;
  cryptoStore.ready  = false;
  cryptoStore.userId = null;
  _keyPair           = null;

  try {
    const kp = await loadKeysFromIndexedDB(userId, password);
    if (!kp) {
      // Pas de clés stockées — utilisateur sans E2EE (ou premier login)
      cryptoStore.error = 'Aucune clé trouvée pour cet utilisateur.';
      return false;
    }
    _keyPair           = kp;
    cryptoStore.userId = userId;
    cryptoStore.ready  = true;
    return true;
  } catch (e: any) {
    // Mot de passe incorrect → crypto_secretbox_open_easy lève une exception
    cryptoStore.error = 'Mot de passe incorrect ou clés corrompues.';
    console.error('[cryptoStore] unlock:', e);
    return false;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// lock — efface les clés de la mémoire (appelé au logout)
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
  messageId:      string;
  conversationId: string;
  ciphertext:     string;
  nonce:          string;
  senderPubkeyB64: string;
}): Promise<string> {
  if (!_keyPair) throw new Error('[cryptoStore] Clés non chargées.');
  if (!cryptoStore.userId) throw new Error('[cryptoStore] userId absent.');

  // Récupérer la clé de session chiffrée pour moi
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
