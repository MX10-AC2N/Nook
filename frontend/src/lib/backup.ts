/**
 * Backup / restore utilities for Nook.
 *
 * - `exportBackup` chiffre les messages avec la clé privée (Base64) et
 *   télécharge un fichier binaire contenant `[nonce][ciphertext]`.
 * - `importBackup` lit le fichier, déchiffre le contenu et renvoie
 *   l’objet JSON d’origine.
 *
 * Toutes les fonctions sont typées, les erreurs sont gérées et le code
 * fonctionne uniquement côté client (`browser`).
 */

// DT-01: dynamic import — sodium chargé à la demande, pas au démarrage
type SodiumType = typeof import('libsodium-wrappers').default;
let _na: SodiumType | null = null;
async function getSodium(): Promise<SodiumType> {
  if (_na) return _na;
  const { default: s } = await import('libsodium-wrappers');
  await s.ready;
  _na = s;
  return s;
}
import { browser } from '$app/environment';

/**
 * Exporte les messages chiffrés dans un fichier `.bin` téléchargeable.
 *
 * @param messages   Tableau d’objets à sauvegarder (tout ce qui peut être sérialisé en JSON).
 * @param privateKeyB64  Clé symétrique (Base64) utilisée pour le chiffrement.
 * @throws Si la clé n’est pas valide ou si l’opération échoue.
 */
export async function exportBackup(
  messages: unknown[],
  privateKeyB64: string
): Promise<void> {
  if (!browser) {
    // En SSR on ne fait rien.
    return;
  }

  const sodium = await getSodium();

  // -----------------------------------------------------------------
  // 1️⃣ Sérialisation & chiffrement
  // -----------------------------------------------------------------
  const key = sodium.from_base64(privateKeyB64, sodium.base64_variants.ORIGINAL);
  if (key.length !== sodium.crypto_secretbox_KEYBYTES) {
    throw new Error('Clé de chiffrement invalide (doit faire 32 bytes).');
  }

  // On passe directement la string JSON à libsodium (overload string → plus de problème de typage)
  const jsonString = JSON.stringify(messages);
  const nonce = sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);
  const ciphertext = sodium.crypto_secretbox_easy(jsonString, nonce, key);

  // -----------------------------------------------------------------
  // 2️⃣ Création du Blob et téléchargement (concaténation pour éviter l’erreur SharedArrayBuffer)
  // -----------------------------------------------------------------
  const combined = new Uint8Array(nonce.length + ciphertext.length);
  combined.set(nonce, 0);
  combined.set(ciphertext, nonce.length);

  const blob = new Blob([combined], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);

  const a = document.createElement('a');
  a.href = url;
  a.download = `nook-backup-${new Date().toISOString().slice(0, 10)}.bin`;
  a.click();

  // Nettoyage
  URL.revokeObjectURL(url);
}

/**
 * Importe un fichier de sauvegarde, le déchiffre et renvoie les messages.
 *
 * @param file          Le fichier `.bin` sélectionné par l’utilisateur.
 * @param privateKeyB64  Clé symétrique (Base64) utilisée pour le déchiffrement.
 * @returns              Le tableau d’objets restauré.
 * @throws               Si le fichier est corrompu ou la clé est invalide.
 */
export async function importBackup(
  file: File,
  privateKeyB64: string
): Promise<unknown[]> {
  if (!browser) {
    throw new Error('Import de backup uniquement disponible dans le navigateur.');
  }

  const sodium = await getSodium();

  const key = sodium.from_base64(privateKeyB64, sodium.base64_variants.ORIGINAL);
  if (key.length !== sodium.crypto_secretbox_KEYBYTES) {
    throw new Error('Clé de chiffrement invalide (doit faire 32 bytes).');
  }

  // -----------------------------------------------------------------
  // 1️⃣ Lecture du fichier en ArrayBuffer
  // -----------------------------------------------------------------
  const arrayBuffer = await file.arrayBuffer();

  // Vérifier que le fichier a au moins la taille du nonce
  if (arrayBuffer.byteLength < sodium.crypto_secretbox_NONCEBYTES) {
    throw new Error('Fichier de sauvegarde trop petit ou corrompu.');
  }

  // -----------------------------------------------------------------
  // 2️⃣ Séparation nonce / ciphertext
  // -----------------------------------------------------------------
  const nonce = new Uint8Array(
    arrayBuffer.slice(0, sodium.crypto_secretbox_NONCEBYTES)
  );
  const ciphertext = new Uint8Array(
    arrayBuffer.slice(sodium.crypto_secretbox_NONCEBYTES)
  );

  // -----------------------------------------------------------------
  // 3️⃣ Déchiffrement
  // -----------------------------------------------------------------
  const decrypted = sodium.crypto_secretbox_open_easy(ciphertext, nonce, key);
  if (!decrypted) {
    throw new Error('Déchiffrement échoué – clé incorrecte ou données corrompues.');
  }

  // -----------------------------------------------------------------
  // 4️⃣ Parsing JSON
  // -----------------------------------------------------------------
  const decoded = new TextDecoder().decode(decrypted);
  return JSON.parse(decoded) as unknown[];
}