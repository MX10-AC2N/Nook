import { sodium } from 'libsodium-wrappers';

export async function exportBackup(messages, privateKey) {
  await sodium.ready;
  const json = JSON.stringify(messages);
  const key = sodium.from_base64(privateKey);
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const ciphertext = sodium.crypto_secretbox_easy(
    new TextEncoder().encode(json),
    nonce,
    key
  );
  const blob = new Blob([nonce, ciphertext], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `nook-backup-${new Date().toISOString().slice(0,10)}.bin`;
  a.click();
  URL.revokeObjectURL(url);
}

export async function importBackup(file, privateKey) {
  await sodium.ready;
  const arrayBuffer = await file.arrayBuffer();
  const nonce = arrayBuffer.slice(0, sodium.crypto_secretbox_NONCEBYTES);
  const ciphertext = arrayBuffer.slice(sodium.crypto_secretbox_NONCEBYTES);
  const key = sodium.from_base64(privateKey);
  const plaintext = sodium.crypto_secretbox_open_easy(ciphertext, nonce, key);
  return JSON.parse(new TextDecoder().decode(plaintext));
}