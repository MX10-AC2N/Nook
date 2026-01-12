/**
 * Device registration utilities.
 *
 * - `generateDeviceLink` crée un token unique stocké dans le `localStorage`
 *   et renvoie une URL que l’utilisateur peut partager (ex. pour associer
 *   un nouvel appareil à son compte).
 * - `registerDevice` ouvre un WebSocket vers le serveur et envoie le nom
 *   de l’appareil ainsi que le token précédemment stocké.
 *
 * Toutes les fonctions sont typées, les erreurs sont gérées et le code
 * ne s’exécute que côté client (`browser`).  
 */

import { browser } from '$app/environment';

/**
 * Génère un lien d’ajout d’appareil.
 *
 * - Crée un UUID (token) stocké dans le `localStorage` sous la clé `device-token`.
 * - Retourne une URL absolue du type `<origin>/join-device?token=<uuid>`.
 *
 * @returns URL à partager (ou chaîne vide si exécuté côté serveur).
 */
export function generateDeviceLink(): string {
  if (!browser) return '';

  const token = crypto.randomUUID();
  localStorage.setItem('device-token', token);
  return `${window.location.origin}/join-device?token=${token}`;
}

/**
 * Enregistre un nouvel appareil via WebSocket.
 *
 * @param name  Nom descriptif de l’appareil (ex. « iPhone », « Laptop »).
 * @returns     Promise qui se résout lorsque le serveur a reçu le message.
 * @throws      Erreur si la connexion WebSocket échoue ou se ferme prématurément.
 */
export async function registerDevice(name: string): Promise<void> {
  if (!browser) {
    // En SSR on ne fait rien.
    return;
  }

  return new Promise<void>((resolve, reject) => {
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const ws = new WebSocket(`${protocol}://${window.location.host}/ws`);

    // Timeout de secours (10 s) au cas où le serveur ne répondrait pas.
    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error('Timeout lors de l’enregistrement de l’appareil'));
    }, 10_000);

    ws.onopen = () => {
      // Récupérer le token stocké précédemment (ou undefined)
      const token = localStorage.getItem('device-token');

      ws.send(
        JSON.stringify({
          type: 'register-device',
          name,
          token,
        })
      );

      clearTimeout(timeout);
      resolve();
    };

    ws.onerror = () => {
      clearTimeout(timeout);
      reject(new Error('Échec de la connexion WebSocket'));
    };

    ws.onclose = (event) => {
      // Si la connexion se ferme avant `onopen`, on considère que c’est une erreur.
      if (event.code !== 1000 && event.wasClean === false) {
        clearTimeout(timeout);
        reject(new Error('WebSocket fermé de manière inattendue'));
      }
    };
  });
}