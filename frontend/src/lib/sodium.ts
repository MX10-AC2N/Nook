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
      
      console.log('🚀 Chargement de libsodium-wrappers-sumo...');
      
      // Méthode 1: Import standard
      try {
        const sodiumModule = await import('libsodium-wrappers-sumo');
        await sodiumModule.ready;
        console.log('✅ Libsodium chargé avec succès (méthode standard)');
        sodiumStore.set(sodiumModule);
        resolve(sodiumModule);
        return;
      } catch (standardError) {
        console.warn('⚠️ Méthode standard échouée:', standardError.message);
      }

      // Méthode 2: Essayer différents chemins possibles
      const possiblePaths = [
        'libsodium-wrappers-sumo',
        'libsodium-wrappers-sumo/dist/modules-sumo/index.js',
        'libsodium-wrappers-sumo/dist/modules-sumo.js',
        'libsodium-wrappers-sumo/dist/browsers-sumo/combined/sodium.js'
      ];

      for (const path of possiblePaths) {
        try {
          console.log(`🔄 Tentative avec le chemin: ${path}`);
          const sodiumModule = await import(/* @vite-ignore */ path);
          
          // Gérer différents formats de module
          let sodiumInstance;
          if (sodiumModule.default) {
            sodiumInstance = sodiumModule.default;
          } else if (sodiumModule.libsodium) {
            sodiumInstance = sodiumModule.libsodium;
          } else {
            sodiumInstance = sodiumModule;
          }

          // Vérifier si ready existe ou utiliser une alternative
          if (typeof sodiumInstance.ready === 'function') {
            await sodiumInstance.ready;
          } else if (sodiumInstance.sodium_version_string) {
            // Si la bibliothèque est déjà prête
            console.log('✅ Libsodium prêt sans appel à ready()');
          } else {
            // Attendre un peu pour laisser le temps à l'initialisation
            await new Promise(resolve => setTimeout(resolve, 100));
          }

          console.log('✅ Libsodium chargé avec succès (méthode alternative)');
          sodiumStore.set(sodiumInstance);
          resolve(sodiumInstance);
          return;
        } catch (pathError) {
          console.warn(`❌ Échec avec ${path}:`, pathError.message);
          continue;
        }
      }

      // Si toutes les méthodes échouent
      throw new Error('Impossible de charger libsodium avec aucune méthode disponible');

    } catch (error) {
      console.error('❌ Erreur définitive chargement libsodium:', error);
      const errorMsg = error instanceof Error ? error : new Error('Erreur chargement libsodium');
      sodiumError.set(errorMsg);
      reject(errorMsg);
    } finally {
      sodiumLoading.set(false);
    }
  });

  return sodiumPromise;
}

// Charge automatiquement au démarrage
loadSodium().catch(console.error);