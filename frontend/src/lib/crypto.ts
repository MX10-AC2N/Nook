import { sodium } from 'libsodium-wrappers';

export const initSodium = async () => {
  await sodium.ready;
};

export const generateKeyPair = () => {
  const keyPair = sodium.crypto_box_keypair();
  return {
    publicKey: sodium.to_base64(keyPair.publicKey),
    privateKey: sodium.to_base64(keyPair.privateKey)
  };
};

export const encryptMessage = (message: string, recipientPublicKey: string, senderPrivateKey: string) => {
  const nonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
  const plaintext = new TextEncoder().encode(message);
  const pk = sodium.from_base64(recipientPublicKey);
  const sk = sodium.from_base64(senderPrivateKey);
  const ciphertext = sodium.crypto_box_easy(plaintext, nonce, pk, sk);
  return sodium.to_base64(nonce) + '|' + sodium.to_base64(ciphertext);
};

export const decryptMessage = (encrypted: string, senderPublicKey: string, recipientPrivateKey: string) => {
  const [nonceB64, cipherB64] = encrypted.split('|');
  const nonce = sodium.from_base64(nonceB64);
  const ciphertext = sodium.from_base64(cipherB64);
  const pk = sodium.from_base64(senderPublicKey);
  const sk = sodium.from_base64(recipientPrivateKey);
  const plaintext = sodium.crypto_box_open_easy(ciphertext, nonce, pk, sk);
  return new TextDecoder().decode(plaintext);
};
