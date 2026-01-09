// frontend/src/lib/sodium.ts
import { writable } from 'svelte/store';

export const sodiumStore = writable<any>(null);
export const sodiumLoading = writable<boolean>(true);
export const sodiumError = writable<Error | null>(null);

let sodiumPromise: Promise<any> | null = null;

export function loadSodium() {
  if (sodiumPromise) return sodiumPromise;

  sodiumPromise = new Promise(async (resolve, reject) => {
    try {
      sodiumLoading.set(true);
      sodiumError.set(null);
      
      // Chargement dynamique pour éviter les problèmes d'initialisation
      const sodiumModule = await import('libsodium-wrappers-sumo');
      
      // Attendre que la bibliothèque soit prête
      await sodiumModule.ready;
      
      sodiumStore.set(sodiumModule);
      console.log('✅ Libsodium chargé avec succès');
      resolve(sodiumModule);
    } catch (error) {
      console.error('❌ Erreur chargement libsodium:', error);
      sodiumError.set(error instanceof Error ? error : new Error('Erreur chargement libsodium'));
      reject(error);
    } finally {
      sodiumLoading.set(false);
    }
  });

  return sodiumPromise;
}

// Charge automatiquement au démarrage
loadSodium();
