import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
  // ──────────────────────────────────────────────────────────────
  // Manual service worker registration
  // ──────────────────────────────────────────────────────────────
  // SvelteKit static adapter + `serviceWorker.register: false` means
  // the SW is built but NEVER registered automatically.
  // This manual registration is REQUIRED for push notifications.
  if (typeof window !== 'undefined') {
    // Enregistrer immédiatement (pas besoin d'attendre le load event)
    if ('serviceWorker' in navigator) {
      try {
        const registration = await navigator.serviceWorker.register('/service-worker.js', {
          scope: '/',
        });
        console.log('[SW] Service worker registered:', registration.scope);
        
        // Vérifier si le SW est actif
        if (registration.active) {
          console.log('[SW] Service worker is active');
        } else if (registration.installing) {
          console.log('[SW] Service worker is installing...');
          registration.installing.addEventListener('statechange', () => {
            console.log('[SW] Service worker state:', registration.installing?.state);
          });
        }
      } catch (error) {
        console.error('[SW] Service worker registration failed:', error);
      }
    } else {
      console.warn('[SW] Service workers not supported');
    }
  }

  return {};
};
