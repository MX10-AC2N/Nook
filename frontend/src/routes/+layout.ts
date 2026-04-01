import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
  // ──────────────────────────────────────────────────────────────
  // Manual service worker registration
  // ──────────────────────────────────────────────────────────────
  // SvelteKit static adapter + `serviceWorker.register: false` means
  // the SW is built but NEVER registered automatically.
  // This manual registration is REQUIRED for push notifications.
  if (typeof window !== 'undefined') {
    window.addEventListener('load', async () => {
      if ('serviceWorker' in navigator) {
        try {
          const registration = await navigator.serviceWorker.register('/service-worker.js', {
            scope: '/',
          });
          console.log('[SW] Service worker registered:', registration.scope);
        } catch (error) {
          console.error('[SW] Service worker registration failed:', error);
        }
      }
    });
  }

  return {};
};
