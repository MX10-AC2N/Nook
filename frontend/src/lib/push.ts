// frontend/src/lib/push.ts
// Gestion des abonnements Web Push côté client — session 39
//
// Flux :
//   1. requestPermission()     → demande la permission navigateur
//   2. subscribeToPush()       → récupère la VAPID public key + pushManager.subscribe()
//   3. POST /api/push/subscribe → enregistre l'abonnement en DB
//   4. unsubscribePush()       → se désabonne + POST /api/push/unsubscribe
//
// Usage dans settings/+page.svelte :
//   import { subscribeToPush, unsubscribePush, getPushState } from '$lib/push';

// Fetch authentifié — credentials:'include' pour envoyer le cookie auth_token

export interface PushState {
  supported:    boolean; // navigateur supporte Push + ServiceWorker
  permission:   NotificationPermission | 'unsupported';
  subscribed:   boolean;
  error:        string | null;
}

// ─────────────────────────────────────────────────────────────────────────────
// État courant (non réactif — appelé à la demande)
// ─────────────────────────────────────────────────────────────────────────────

export async function getPushState(): Promise<PushState> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator) || !('PushManager' in window)) {
    return { supported: false, permission: 'unsupported', subscribed: false, error: null };
  }

  try {
    // Check if SW is actually registered before waiting on .ready (which hangs forever if none)
    const reg = await Promise.race([
      navigator.serviceWorker.ready,
      new Promise<null>((_, reject) => setTimeout(() => reject(new Error('SW timeout')), 3000))
    ]);
    if (!reg) {
      return { supported: true, permission: Notification.permission, subscribed: false, error: 'Service Worker non enregistré' };
    }
    const sub = await reg.pushManager.getSubscription();

    return {
      supported:  true,
      permission: Notification.permission,
      subscribed: !!sub,
      error:      null,
    };
  } catch {
    return { supported: true, permission: Notification.permission, subscribed: false, error: 'Service Worker non prêt' };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Abonnement
// ─────────────────────────────────────────────────────────────────────────────

export async function subscribeToPush(): Promise<{ success: boolean; error?: string }> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator) || !('PushManager' in window)) {
    return { success: false, error: 'Push non supporté par ce navigateur' };
  }

  // 1. Demander la permission si nécessaire
  if (Notification.permission === 'denied') {
    return { success: false, error: 'Notifications bloquées dans les paramètres du navigateur' };
  }
  if (Notification.permission === 'default') {
    const perm = await Notification.requestPermission();
    if (perm !== 'granted') {
      return { success: false, error: 'Permission refusée' };
    }
  }

  // 2. Récupérer la clé VAPID publique depuis le backend
  let vapidKey: string;
  try {
    const res = await fetch('/api/push/vapid-public-key', { credentials: 'include' });
    const body = await res.json();
    vapidKey = body.public_key;
    if (!vapidKey) {
      return { success: false, error: 'VAPID non configuré sur ce serveur' };
    }
  } catch {
    return { success: false, error: 'Impossible de récupérer la clé VAPID' };
  }

  // 3. Enregistrer le SW si nécessaire
  let reg: ServiceWorkerRegistration;
  try {
    // Vérifier si un SW est déjà enregistré
    reg = await navigator.serviceWorker.getRegistration();
    if (!reg) {
      // Enregistrer le SW maintenant avec timeout
      console.log('[push] Registering SW...');
      reg = await Promise.race([
        navigator.serviceWorker.register('/service-worker.js', { scope: '/' }),
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('SW register timeout (5s) — certificat SSL probablement non approuvé')), 5000)
        )
      ]);
      console.log('[push] SW registered:', reg.scope);
    }

    // Attendre que le SW soit actif (max 10s)
    if (!reg.active) {
      console.log('[push] Waiting for SW to activate...');
      await Promise.race([
        new Promise<void>((resolve) => {
          if (reg.installing) {
            reg.installing.addEventListener('statechange', () => {
              if (reg.installing?.state === 'activated') {
                console.log('[push] SW activated');
                resolve();
              }
            });
          } else if (reg.waiting) {
            reg.waiting.addEventListener('statechange', () => {
              if (reg.waiting?.state === 'activated') {
                console.log('[push] SW activated');
                resolve();
              }
            });
          } else {
            resolve(); // Already active
          }
        }),
        new Promise<void>((_, reject) => 
          setTimeout(() => reject(new Error('SW activation timeout (10s)')), 10000)
        )
      ]);
    }

    if (!reg.active) {
      return { success: false, error: 'Service Worker non actif après 10s' };
    }

    console.log('[push] SW ready:', reg.scope);
  } catch (err: any) {
    const msg = err?.message ?? String(err);
    if (msg.includes('timeout') || msg.includes('certificat') || msg.includes('SSL')) {
      return {
        success: false,
        error: `Certificat non approuvé → ouvrez ${location.origin}/ca/help pour installer le CA (instructions incluses).`
      };
    }
    return { success: false, error: `Service Worker : ${msg}` };
  }

  // 4. S'abonner via pushManager
  let subscription: PushSubscription;
  try {
    subscription = await reg.pushManager.subscribe({
      userVisibleOnly:      true,
      applicationServerKey: urlBase64ToUint8Array(vapidKey),
    });
  } catch (err: any) {
    return { success: false, error: `Abonnement push échoué : ${err?.message ?? err}` };
  }

  // 4. Enregistrer en DB via le backend
  try {
    const { endpoint, keys } = subscription.toJSON() as {
      endpoint: string;
      keys: { p256dh: string; auth: string };
    };

    const res = await fetch('/api/push/subscribe', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        endpoint,
        keys: { p256dh: keys.p256dh, auth: keys.auth },
        user_agent: navigator.userAgent.slice(0, 200),
      }),
    });

    if (!res.ok) {
      return { success: false, error: `Serveur : HTTP ${res.status}` };
    }

    return { success: true };
  } catch (err: any) {
    return { success: false, error: `Erreur réseau : ${err?.message ?? err}` };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Désabonnement
// ─────────────────────────────────────────────────────────────────────────────

export async function unsubscribePush(): Promise<{ success: boolean; error?: string }> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
    return { success: false, error: 'Push non disponible' };
  }

  try {
    const reg = await navigator.serviceWorker.ready;
    const sub = await reg.pushManager.getSubscription();

    if (sub) {
      await sub.unsubscribe();

      // Informer le backend (best-effort, pas bloquant)
      try {
        await fetch('/api/push/unsubscribe', {
          method: 'DELETE',
          credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ endpoint: sub.endpoint }),
        });
      } catch { /* no-op */ }
    }

    return { success: true };
  } catch (err: any) {
    return { success: false, error: err?.message ?? 'Erreur désabonnement' };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Utilitaire : convertir base64url → Uint8Array (requis par pushManager.subscribe)
// ─────────────────────────────────────────────────────────────────────────────

function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64  = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw     = window.atob(base64);
  return Uint8Array.from([...raw].map(c => c.charCodeAt(0)));
}
