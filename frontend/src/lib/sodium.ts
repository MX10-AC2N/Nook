// frontend/src/lib/sodium.ts
import { writable } from 'svelte/store';
import sodium from 'libsodium-wrappers';  // ← Import standard

export const sodiumStore = writable<typeof sodium | null>(null);
export const sodiumLoading = writable<boolean>(true);
export const sodiumError = writable<Error | null>(null);

let sodiumPromise: Promise<typeof sodium> | null = null;

export async function loadSodium(): Promise<typeof sodium> {
  if (sodiumPromise) return sodiumPromise;

  sodiumPromise = (async () => {
    try {
      sodiumLoading.set(true);
      await sodium.ready;
      console.log('✅ Libsodium chargé');
      sodiumStore.set(sodium);
      return sodium;
    } catch (error) {
      console.error('❌ Erreur libsodium:', error);
      sodiumError.set(error instanceof Error ? error : new Error('Erreur sodium'));
      throw error;
    } finally {
      sodiumLoading.set(false);
    }
  })();

  return sodiumPromise;
}

loadSodium().catch(console.error);