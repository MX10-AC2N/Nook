// src/lib/crypto.rotation.test.ts
//
// DT-05-bis regression test — forward secrecy after X25519 key rotation.
//
// Scenario: Bob (recipient) receives a message encrypted to his v1 public key.
// Bob then rotates his keypair → his in-memory _keyPair.privateKey becomes v2.
// When decrypting the OLD message, decryptSessionKeyV2 MUST fall back to the
// archived v1 private key (via archivedKeyLookup) instead of failing with the
// current v2 private key. This is the exact path that was broken by the
// archivedKeyLookup stub returning null in cryptoStore.decryptMessage.
//
// We exercise the real crypto primitives (libsodium) so the test proves the
// end-to-end box/open round-trip, not just a mock.

import { describe, it, expect, beforeAll } from 'vitest';
import {
  generateKeyPair,
  encryptForRecipients,
  decryptSessionKeyV2,
  decryptContent,
  encryptPrivateKey,
  decryptPrivateKey,
  type KeyPair,
} from '$lib/crypto';
import { waitForSodium } from '$lib/sodium.svelte.js';

const PASSWORD = 'correct horse battery staple';

function b64(k: Uint8Array): string {
  let bin = '';
  for (const b of k) bin += String.fromCharCode(b);
  return btoa(bin);
}

describe('DT-05-bis: forward secrecy after key rotation', () => {
  let alice: KeyPair;        // sender (key never rotates in this test)
  let bobV1: KeyPair;        // recipient, original keypair
  let bobV2: KeyPair;        // recipient, after rotation
  let bobV1EncPriv: string;  // archived encrypted private key blob (salt||nonce||ct)

  beforeAll(async () => {
    await waitForSodium();
    alice = await generateKeyPair();
    bobV1 = await generateKeyPair();
    bobV2 = await generateKeyPair();
    // Archive bob's v1 private key the same way the store does (password-derived).
    bobV1EncPriv = await encryptPrivateKey(bobV1.privateKey, PASSWORD);
  });

  it('decrypts a v1 message after rotation using the archived v1 private key', async () => {
    const plaintext = 'Message secret chiffré avec la clé v1 de Bob';

    // Alice encrypts TO Bob's v1 public key while Bob still uses v1.
    const encMsg = await encryptForRecipients(plaintext, { bob: b64(bobV1.publicKey) }, alice);
    const encKey = encMsg.encryptedKeys['bob'];
    expect(encKey).toBeTruthy();

    // Bob has rotated: his current in-memory private key is now v2.
    const bobCurrentPriv = bobV2.privateKey;

    // archivedKeyLookup returns the v1 private key (decrypted from archive).
    const archivedKeyLookup = async (version: number): Promise<Uint8Array | null> => {
      if (version === 1) return await decryptPrivateKey(bobV1EncPriv, PASSWORD);
      return null;
    };

    const result = await decryptSessionKeyV2(
      encKey,
      b64(alice.publicKey),
      bobCurrentPriv, // current (v2) key — must FAIL to open a v1 message
      { senderKeyVersion: 1, archivedKeyLookup }
    );

    // Forward secrecy preserved: fallback to archived key succeeded.
    expect(result.usedArchivedKey).toBe(true);

    const decrypted = await decryptContent(encMsg.ciphertext, encMsg.nonce, result.sessionKey);
    expect(decrypted).toBe(plaintext);
  });

  it('throws (and does NOT silently return garbage) when no archived key is available', async () => {
    const plaintext = 'Autre message v1';
    const encMsg = await encryptForRecipients(plaintext, { bob: b64(bobV1.publicKey) }, alice);
    const encKey = encMsg.encryptedKeys['bob'];

    // archivedKeyLookup returns null → must throw (cannot open with v2 key).
    await expect(
      decryptSessionKeyV2(encKey, b64(alice.publicKey), bobV2.privateKey, {
        senderKeyVersion: 1,
        archivedKeyLookup: async () => null,
      })
    ).rejects.toThrow();
  });
});
