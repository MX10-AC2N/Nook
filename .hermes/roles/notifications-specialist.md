# 🔔 Rôle : Spécialiste Notifications — Nook

> Spécialiste du système de notifications in-app, Web Push, et toast pour Nook.

## Domaine d'expertise
- Notifications in-app via `notificationStore.svelte.ts`
- Composant `NotificationToast.svelte` (toast + historique)
- Intégration AudioContext pour HTTP/LAN
- Web Push (VAPID) pour HTTPS
- Patterns d'intégration dans chaque module (chat, chess, polls, calendar, admin, calls)

## Connaissances requises
1. Store `notificationStore.svelte.ts` — types NotificationType, fonctions notify/notifyMessage/notifyChess/etc.
2. Composant `NotificationToast.svelte` — affichage toast, historique, dismiss
3. AudioContext — initialisation nécessaire après interaction utilisateur
4. Web Push — VAPID keys, service worker, registration
5. Intégration dans modules — imports manquants (polls, calendar, admin)

## Problèmes courants
1. **notifyXxx is not defined** → Import manquant dans le composant
2. **Pas de son** → AudioContext non initialisé (nécessite interaction)
3. **Web Push non fonctionnel** → HTTPS requis (pas dispo en HTTP/LAN)
4. **Toast pas visible** → Vérifier CSS `.toast` et z-index

## Patterns de code
```typescript
// Import dans un module
import { notifyMessage } from '$lib/notificationStore.svelte';

// Appel dans une fonction
notifyMessage('Nouveau message', content.slice(0, 50));
```

## Checklist d'intégration
1. Importer `notifyXxx` depuis `notificationStore.svelte`
2. Appeler dans le bon contexte (envoi message, création sondage, etc.)
3. Tester sur HTTP/LAN (AudioContext) et HTTPS (Web Push)
4. Vérifier CSS (z-index, position)
