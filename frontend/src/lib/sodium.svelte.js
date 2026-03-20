// src/lib/sodium.svelte.js
// Helper pour charger et initialiser libsodium-wrappers avec Svelte 5 runes
//
// DT-01 FIX : import dynamique — libsodium (938 kB WASM) n'est chargé
// que lorsqu'il est réellement nécessaire, pas au démarrage de l'app.
// Gain : layout visible ~500ms plus tôt sur les pages qui n'utilisent
// pas le chiffrement (chess, calendar, polls, help…).

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

// Promise de chargement partagée — évite les doubles imports simultanés
let loadingPromise = null;

/**
 * Attend que libsodium soit prêt.
 * Premier appel : déclenche le dynamic import (lazy).
 * Appels suivants : retourne l'instance déjà chargée immédiatement.
 * @returns {Promise<Sodium>}
 */
export async function waitForSodium() {
  // Déjà prêt → retour synchrone-ish
  if (sodiumInstance && sodiumState.isReady) {
    return sodiumInstance;
  }

  // Chargement déjà en cours → attacher à la même promise
  if (loadingPromise) {
    return loadingPromise;
  }

  // Premier appel : déclencher le chargement
  sodiumState.isLoading = true;
  sodiumState.error = null;

  loadingPromise = (async () => {
    try {
      // Import dynamique — le chunk libsodium n'est téléchargé qu'ici
      const { default: sodium } = await import('libsodium-wrappers');
      await sodium.ready;
      sodiumInstance = sodium;
      sodiumState.isReady = true;
      sodiumState.isLoading = false;
      return sodiumInstance;
    } catch (err) {
      console.error('Erreur lors du chargement de libsodium:', err);
      sodiumState.error = err.message || 'Échec du chargement de libsodium';
      sodiumState.isLoading = false;
      loadingPromise = null; // Permettre une nouvelle tentative
      throw err;
    }
  })();

  return loadingPromise;
}

/**
 * Précharge libsodium (à appeler en arrière-plan après login, pas au démarrage).
 * Ne bloque pas le rendu si ça échoue.
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
