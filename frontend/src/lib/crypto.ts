import { sodium } from 'libsodium-wrappers';

export async function initSodium() {
  await sodium.ready;
}

export function generateKeyPair() {
  const keyPair = sodium.crypto_box_keypair();
  return {
    publicKey: sodium.to_base64(keyPair.publicKey),
    privateKey: sodium.to_base64(keyPair.privateKey)
  };
}

export function encryptMessage(
  plaintext: string,
  recipientPublicKey: string,
  senderPrivateKey: string
): string {
  const nonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
  const ciphertext = sodium.crypto_box_easy(
    new TextEncoder().encode(plaintext),
    nonce,
    sodium.from_base64(recipientPublicKey),
    sodium.from_base64(senderPrivateKey)
  );
  return sodium.to_base64(nonce) + '|' + sodium.to_base64(ciphertext);
}

export function decryptMessage(
  encrypted: string,
  senderPublicKey: string,
  myPrivateKey: string
): string {
  const [nonceB64, cipherB64] = encrypted.split('|');
  const plaintext = sodium.crypto_box_open_easy(
    sodium.from_base64(cipherB64),
    sodium.from_base64(nonceB64),
    sodium.from_base64(senderPublicKey),
    sodium.from_base64(myPrivateKey)
  );
  return new TextDecoder().decode(plaintext);
}