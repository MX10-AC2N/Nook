self.addEventListener('push', (event) => {
  const data = event.data?.json() || { title: 'Nook', body: 'Nouveau message' };
  const options = {
    body: data.body,
    icon: '/icon-192.png',
    badge: '/icon-72.png',
    vibrate: [200, 100, 200],
    actions: [
      { action: 'reply', title: 'Répondre', icon: '/reply.png' },
      { action: 'dismiss', title: 'Ignorer' }
    ]
  };
  event.waitUntil(self.registration.showNotification(data.title, options));
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  if (event.action === 'reply') {
    clients.openWindow('/chat?reply=1');
  } else {
    clients.openWindow('/chat');
  }
});