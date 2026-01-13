// src/lib/sodium.ts (Svelte 5 avec runes)
import sodium from 'libsodium-wrappers'; // ← import standard (module avec .ready)

/**
 * État réactif contenant l'instance de libsodium (ou `null` tant que le module n'est pas chargé).
 */
export let sodiumStore = $state<any>(null);

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
let sodiumPromise: Promise<any> | null = null;

/**
 * Charge libsodium-wrappers (si ce n'est pas déjà fait) et met à jour l'état.
 *
 * @returns {Promise<any>} L'instance prête de libsodium.
 */
export async function loadSodium(): Promise<any> {
  // Si le chargement a déjà été déclenché, on renvoie la même promesse.
  if (sodiumPromise) return sodiumPromise;

  sodiumPromise = (async () => {
    try {
      sodiumLoading = true;
      sodiumError = null;
      
      // `sodium.ready` est une promesse qui se résout quand le WASM est chargé.
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
 * @returns {Promise<any>} L'instance prête de libsodium.
 */
export async function getSodium(): Promise<any> {
  if (sodiumStore) return sodiumStore;
  return await loadSodium();
}

/**
 * Fonction utilitaire pour les composants qui veulent attendre sodiumReady
 * (alternative à $effect(() => { if (sodiumReady) ... }))
 */
export async function waitForSodium(): Promise<void> {
  if (!sodiumReady) {
    await loadSodium();
  }
}

/* -----------------------------------------------------------------
   Optionnel : on lance le chargement dès que le module est importé.
   Si vous préférez contrôler le moment du chargement (ex. dans +layout),
   supprimez l'appel ci-dessous.
   ----------------------------------------------------------------- */
// Décommentez si vous voulez le préchargement automatique
// loadSodium().catch((e) => console.error('Failed to preload libsodium:', e));