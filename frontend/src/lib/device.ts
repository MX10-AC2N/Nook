/**
 * Device registration utilities.
 * Compatible HTTP/LAN — n'utilise pas crypto.randomUUID() (secure context uniquement).
 */

import { browser } from '$app/environment';

/**
 * Génère un UUID v4 cryptographiquement sûr compatible HTTP/LAN.
 * NOTE: `crypto.getRandomValues()` est disponible en HTTP et HTTPS.
 * Seul `crypto.randomUUID()` est restreint au secure context (HTTPS).
 * Cette implémentation fonctionne partout en utilisant crypto.getRandomValues().
 */
function generateId(): string {
  // FIX L3: utiliser crypto.getRandomValues au lieu de Math.random()
  // crypto.getRandomValues est disponible en HTTP (secure context non requis)
  const buf = new Uint8Array(16);
  crypto.getRandomValues(buf);
  // Forcer version 4, variant 2
  buf[6] = (buf[6] & 0x0f) | 0x40;
  buf[8] = (buf[8] & 0x3f) | 0x80;
  return [
    buf.slice(0, 4).reduce((s, b) => s + b.toString(16).padStart(2, '0'), ''),
    buf.slice(4, 6).reduce((s, b) => s + b.toString(16).padStart(2, '0'), ''),
    buf.slice(6, 8).reduce((s, b) => s + b.toString(16).padStart(2, '0'), ''),
    buf.slice(8, 10).reduce((s, b) => s + b.toString(16).padStart(2, '0'), ''),
    buf.slice(10).reduce((s, b) => s + b.toString(16).padStart(2, '0'), ''),
  ].join('-');
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
    const ws = new WebSocket(`${protocol}://${window.location.host}/api/webrtc/ws`);

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
