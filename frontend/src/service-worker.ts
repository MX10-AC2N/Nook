// Enregistrement du Service Worker
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/service-worker.js')
      .then(registration => console.log('SW registered'))
      .catch(err => console.log('SW failed:', err));
  });
}

// Service Worker (à compiler dans static/)
self.addEventListener('push', (event) => {
  const data = event.data?.json() || { title: 'Nook', body: 'Nouveau message' };
  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: '/icon-192.png'
    })
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(clients.openWindow('/chat'));
});