// src/lib/sodium.ts
import { writable, derived, type Writable } from 'svelte/store';
import sodium from 'libsodium-wrappers'; // ← import standard (module avec .ready)

/**
 * Store contenant l'instance de libsodium (ou `null` tant que le module n'est pas chargé).
 */
export const sodiumStore: Writable<any> = writable(null);

/**
 * Indique si le chargement est en cours.
 */
export const sodiumLoading = writable<boolean>(true);

/**
 * Contient l'éventuelle erreur survenue pendant le chargement.
 */
export const sodiumError = writable<Error | null>(null);

/**
 * Store dérivé qui vaut `true` dès que l'instance de libsodium est disponible.
 */
export const sodiumReady = derived(sodiumStore, ($s) => $s !== null);

/* -----------------------------------------------------------------
   Internals – on ne veut charger le module qu’une seule fois.
   ----------------------------------------------------------------- */
let sodiumPromise: Promise<any> | null = null;

/**
 * Charge libsodium‑wrappers (si ce n’est pas déjà fait) et met à jour les stores.
 *
 * @returns {Promise<any>} L'instance prête de libsodium.
 */
export async function loadSodium(): Promise<any> {
  // Si le chargement a déjà été déclenché, on renvoie la même promesse.
  if (sodiumPromise) return sodiumPromise;

  sodiumPromise = (async () => {
    try {
      sodiumLoading.set(true);
      // `sodium.ready` est une promesse qui se résout quand le WASM est chargé.
      await sodium.ready;

      // À ce stade, `sodium` expose toutes les fonctions cryptographiques.
      sodiumStore.set(sodium);
      console.log('✅ Libsodium chargé');
      return sodium;
    } catch (rawErr) {
      const err = rawErr instanceof Error ? rawErr : new Error(String(rawErr));
      console.error('❌ Erreur lors du chargement de libsodium :', err);
      sodiumError.set(err);
      throw err;
    } finally {
      sodiumLoading.set(false);
    }
  })();

  return sodiumPromise;
}

/**
 * Retourne immédiatement l’instance de libsodium si elle est déjà chargée,
 * sinon lance le chargement et attend qu’il soit terminé.
 *
 * @returns {Promise<any>} L’instance prête de libsodium.
 */
export async function getSodium(): Promise<any> {
  const current = sodiumStore.get();
  if (current) return current;
  return loadSodium();
}

/* -----------------------------------------------------------------
   Optionnel : on lance le chargement dès que le module est importé.
   Si vous préférez contrôler le moment du chargement (ex. dans +layout),
   supprimez l’appel ci‑dessous.
   ----------------------------------------------------------------- */
// loadSodium().catch((e) => console.error('Failed to preload libsodium:', e));