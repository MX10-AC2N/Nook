// /frontend/src/service-worker.ts

// Icônes (assure-toi qu'elles existent dans /public ou /static)
const ICONS = {
  default: '/icon-192.png',
  badge: '/icon-72.png',
  reply: '/reply.png',
  dark: '/icon-192-dark.png',     // Optionnel : icône pour mode sombre
  badgeDark: '/icon-72-dark.png'
} as const;

// Vibration subtile et chaleureuse (comme un petit cœur qui bat)
const VIBRATION_PATTERN = [100, 50, 100, 50, 200];

self.addEventListener('push', (event) => {
  const data = event.data?.json() || {
    title: 'Nook',
    body: 'Nouveau message dans la famille ❤️',
    image: null,
    tag: 'nook-message'
  };

  // Détection du mode sombre (si supporté par le navigateur)
  const isDark = matchMedia('(prefers-color-scheme: dark)').matches;

  const options: NotificationOptions = {
    body: data.body || 'Vous avez un nouveau message',
    icon: isDark && ICONS.dark ? ICONS.dark : ICONS.default,
    badge: isDark && ICONS.badgeDark ? ICONS.badgeDark : ICONS.badge,
    image: data.image || undefined, // Affiche l'image (GIF, photo) si envoyée
    tag: data.tag || 'nook-notification', // Groupe les notifs
    renotify: true,
    vibrate: VIBRATION_PATTERN,
    timestamp: Date.now(),
    data: { url: '/chat' }, // URL à ouvrir
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
    self.registration.showNotification(data.title || 'Nook', options)
  );
});

self.addEventListener('notificationclick', async (event) => {
  const notification = event.notification;
  notification.close();

  const urlToOpen = notification.data?.url || '/chat';

  // Cherche une fenêtre Nook déjà ouverte
  const windowClients = await clients.matchAll({
    type: 'window',
    includeUncontrolled: true
  });

  let targetClient = windowClients.find(client =>
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
    targetClient = await clients.openWindow(urlToOpen);

    // Optionnel : attendre que la page charge pour focus l'input
    if (event.action === 'reply' && targetClient) {
      // Petit délai pour laisser la page charger
      setTimeout(() => {
        targetClient.postMessage({ action: 'focus-reply-input' });
      }, 1000);
    }
  }
});