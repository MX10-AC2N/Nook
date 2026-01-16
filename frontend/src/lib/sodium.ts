// src/lib/sodium.ts (Svelte 5 avec runes)

import sodium from 'libsodium-wrappers';

/* -----------------------------------------------------------------
   Types exportés
   ----------------------------------------------------------------- */
export type Sodium = typeof import('libsodium-wrappers');

/* -----------------------------------------------------------------
   State réactif
   ----------------------------------------------------------------- */

/**
 * Instance de libsodium (ou `null` tant que le module n'est pas chargé).
 */
export let sodiumStore = $state<Sodium | null>(null);

/**
 * Indique si le chargement est en cours.
 */
export let sodiumLoading = $state<boolean>(true);

/**
 * Contient l'éventuelle erreur survenue pendant le chargement.
 */
export let sodiumError = $state<Error | null>(null);

/**
 * Variable dérivée qui vaut `true` dès que l'instance de libsodium est disponible.
 */
export const sodiumReady = $derived(sodiumStore !== null);

/* -----------------------------------------------------------------
   Internals – on ne veut charger le module qu'une seule fois.
   ----------------------------------------------------------------- */
let sodiumPromise: Promise<Sodium> | null = null;

/**
 * Charge `libsodium-wrappers` (si ce n'est pas déjà fait) et met à jour l'état.
 *
 * @returns {Promise<Sodium>} L'instance prête de libsodium.
 */
export async function loadSodium(): Promise<Sodium> {
  // Si le chargement a déjà été déclenché, on renvoie la même promesse.
  if (sodiumPromise) return sodiumPromise;

  // On crée la promesse unique.
  sodiumPromise = (async (): Promise<Sodium> => {
    try {
      sodiumLoading = true;
      sodiumError = null;

      // `sodium.ready` se résout quand le WASM est chargé.
      await sodium.ready;

      // À ce stade, `sodium` expose toutes les fonctions cryptographiques.
      sodiumStore = sodium;
      console.log('✅ Libsodium chargé');

      return sodium;
    } catch (rawErr) {
      const err = rawErr instanceof Error ? rawErr : new Error(String(rawErr));
      console.error('❌ Erreur lors du chargement de libsodium :', err);
      sodiumError = err;
      throw err;
    } finally {
      sodiumLoading = false;
    }
  })();

  return sodiumPromise;
}

/**
 * Retourne immédiatement l'instance de libsodium si elle est déjà chargée,
 * sinon lance le chargement et attend qu'il soit terminé.
 *
 * @returns {Promise<Sodium>} L'instance prête de libsodium.
 */
export async function getSodium(): Promise<Sodium> {
  if (sodiumStore) return sodiumStore;
  return await loadSodium();
}

/**
 * Fonction utilitaire pour les composants qui veulent attendre que
 * `sodiumReady` devienne vrai (alternative à `$effect(() => { if (sodiumReady) … })`).
 *
 * @returns {Promise<Sodium>} L'instance prête de libsodium.
 */
export async function waitForSodium(): Promise<Sodium> {
  if (!sodiumReady) {
    await loadSodium();
  }
  // À ce point, `sodiumStore` ne peut plus être nul.
  return sodiumStore as Sodium;
}

/* -----------------------------------------------------------------
   Optionnel : préchargement automatique
   ----------------------------------------------------------------- */
// Décommentez si vous voulez le préchargement dès l'import du module.
  loadSodium().catch((e) => console.error('Failed to preload libsodium:', e));