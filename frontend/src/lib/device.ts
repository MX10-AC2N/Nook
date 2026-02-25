/**
 * Device registration utilities.
 * Compatible HTTP/LAN — n'utilise pas crypto.randomUUID() (secure context uniquement).
 */

import { browser } from '$app/environment';

/**
 * Génère un UUID v4 compatible HTTP/LAN.
 * crypto.randomUUID() n'est disponible qu'en secure context (HTTPS).
 * Cette implémentation fonctionne sur HTTP (LAN) et HTTPS (WAN).
 */
function generateId(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = Math.random() * 16 | 0;
    return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
  });
}

/**
 * Génère un lien d'ajout d'appareil.
 *
 * - Crée un token UUID stocké dans le `localStorage` sous la clé `device-token`.
 * - Retourne une URL absolue du type `<origin>/join-device?token=<uuid>`.
 *
 * @returns URL à partager (ou chaîne vide si exécuté côté serveur).
 */
export function generateDeviceLink(): string {
  if (!browser) return '';

  const token = generateId();
  localStorage.setItem('device-token', token);
  return `${window.location.origin}/join-device?token=${token}`;
}

/**
 * Enregistre un nouvel appareil via WebSocket.
 *
 * @param name  Nom descriptif de l'appareil (ex. « iPhone », « Laptop »).
 * @returns     Promise qui se résout lorsque le serveur a reçu le message.
 * @throws      Erreur si la connexion WebSocket échoue ou se ferme prématurément.
 */
export async function registerDevice(name: string): Promise<void> {
  if (!browser) return;

  return new Promise<void>((resolve, reject) => {
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const ws = new WebSocket(`${protocol}://${window.location.host}/ws`);

    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error('Timeout lors de l\'enregistrement de l\'appareil'));
    }, 10_000);

    ws.onopen = () => {
      const token = localStorage.getItem('device-token');
      ws.send(JSON.stringify({ type: 'register-device', name, token }));
      clearTimeout(timeout);
      resolve();
    };

    ws.onerror = () => {
      clearTimeout(timeout);
      reject(new Error('Échec de la connexion WebSocket'));
    };

    ws.onclose = (event) => {
      if (event.code !== 1000 && event.wasClean === false) {
        clearTimeout(timeout);
        reject(new Error('WebSocket fermé de manière inattendue'));
      }
    };
  });
}
