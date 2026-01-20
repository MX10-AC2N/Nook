// src/lib/sodium.svelte.js
// Helper pour charger et initialiser libsodium-wrappers avec Svelte 5 runes

import sodium from 'libsodium-wrappers';

// =====================================================================
// STATE - État du chargement de libsodium (Svelte 5 runes)
// =====================================================================
export const sodiumState = $state({
  isReady: false,
  isLoading: false,
  error: null,
});

// Instance de sodium (une fois chargée)
let sodiumInstance = null;

/**
 * Attend que libsodium soit prêt
 * @returns {Promise<Sodium>} L'instance de libsodium prête à l'emploi
 */
export async function waitForSodium() {
  // Si déjà chargé, retourner immédiatement
  if (sodiumInstance && sodiumState.isReady) {
    return sodiumInstance;
  }

  // Si en cours de chargement, attendre
  if (sodiumState.isLoading) {
    return new Promise((resolve, reject) => {
      const checkReady = setInterval(() => {
        if (sodiumState.isReady && sodiumInstance) {
          clearInterval(checkReady);
          resolve(sodiumInstance);
        }
        if (sodiumState.error) {
          clearInterval(checkReady);
          reject(new Error(sodiumState.error));
        }
      }, 100);
    });
  }

  // Sinon, charger libsodium
  sodiumState.isLoading = true;
  sodiumState.error = null;

  try {
    await sodium.ready;
    sodiumInstance = sodium;
    sodiumState.isReady = true;
    sodiumState.isLoading = false;
    return sodiumInstance;
  } catch (err) {
    console.error('Erreur lors du chargement de libsodium:', err);
    sodiumState.error = err.message || 'Échec du chargement de libsodium';
    sodiumState.isLoading = false;
    throw err;
  }
}

/**
 * Précharge libsodium (à appeler au démarrage de l'app)
 */
export async function preloadSodium() {
  try {
    await waitForSodium();
    console.log('✅ libsodium chargé et prêt');
  } catch (err) {
    console.error('❌ Échec du préchargement de libsodium:', err);
  }
}

/**
 * Vérifie si libsodium est prêt (synchrone)
 * @returns {boolean}
 */
export function isSodiumReady() {
  return sodiumState.isReady;
}

/**
 * Obtient l'instance de sodium (si prête, sinon null)
 * @returns {Sodium|null}
 */
export function getSodiumInstance() {
  return sodiumInstance;
}
