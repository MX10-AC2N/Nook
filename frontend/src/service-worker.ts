/// <reference no-default-lib="true" />
/// <reference lib="webworker" />

// Icônes (assure-toi qu'elles existent dans /static ou /public)
const ICONS = {
  default: '/icon-192.png',
  badge: '/icon-72.png',
  reply: '/icons/reply.svg',
  dark: '/icon-192-dark.png',
  badgeDark: '/icon-72-dark.png'
} as const;

// Vibration subtile et chaleureuse (comme un petit cœur qui bat)
const VIBRATION_PATTERN = [100, 50, 100, 50, 200];

// Type assertion helpers
const swSelf = self as unknown as ServiceWorkerGlobalScope;

swSelf.addEventListener('push', (event: PushEvent) => {
  let data = {
    title: 'Nook',
    body: 'Nouveau message dans la famille ❤️',
    image: null as string | null,
    tag: 'nook-message',
    // Optionnel : si tu veux gérer le mode sombre, envoie cette info depuis ton serveur dans le payload push
    prefersDark: false
  };

  if (event.data) {
    try {
      data = { ...data, ...event.data.json() };
    } catch (e) {
      console.error('Erreur parsing push data:', e);
    }
  };

  // Utilise les icônes dark si le payload indique le mode sombre
  const useDark = !!data.prefersDark;

  const options = {
    body: data.body || 'Vous avez un nouveau message',
    icon: useDark ? ICONS.dark : ICONS.default,
    badge: useDark ? ICONS.badgeDark : ICONS.badge,
    tag: data.tag ?? 'nook-notification',
    data: { url: '/chat' },
    actions: [
      {
        action: 'open-chat',
        title: 'Voir le message',
        icon: ICONS.default
      },
      {
        action: 'reply',
        title: 'Répondre rapidement',
        icon: ICONS.reply
      }
    ]
  };

  event.waitUntil(
    swSelf.registration.showNotification(data.title || 'Nook', options)
  );
});

swSelf.addEventListener('notificationclick', (event: NotificationEvent) => {
  event.notification.close();

  const urlToOpen = (event.notification.data?.url as string | undefined) ?? '/chat';

  event.waitUntil(
    (async () => {
      // Cherche une fenêtre Nook déjà ouverte
      const windowClients = await swSelf.clients.matchAll({
        type: 'window',
        includeUncontrolled: true
      });

      let targetClient: WindowClient | undefined = windowClients.find(client =>
        client.url.includes(urlToOpen) && 'focus' in client
      );

      if (targetClient) {
        // Focus sur l'onglet existant
        await targetClient.focus();

        // Si action "reply", on peut envoyer un message au client pour focus l'input
        if (event.action === 'reply') {
          targetClient.postMessage({ action: 'focus-reply-input' });
        }
      } else {
        // Ouvre une nouvelle fenêtre
        const newClient = await swSelf.clients.openWindow(urlToOpen);
        targetClient = newClient ?? undefined;

        // Optionnel : attendre que la page charge pour focus l'input
        if (event.action === 'reply' && newClient) {
          // Petit délai pour laisser la page charger
          setTimeout(() => {
            newClient?.postMessage({ action: 'focus-reply-input' });
          }, 1000);
        }
      }
    })()
  );
});

swSelf.addEventListener('install', (event: ExtendableEvent) => {
  console.log('Service Worker installé');
  swSelf.skipWaiting();
});

swSelf.addEventListener('activate', (event: ExtendableEvent) => {
  console.log('Service Worker activé');
  event.waitUntil(swSelf.clients.claim());
});