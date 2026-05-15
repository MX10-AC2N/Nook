/**
 * Envoie une alerte d’urgence à tous les membres.
 *
 * 1️⃣ Notifie les clients connectés via WebSocket (`type: 'emergency'`).
 * 2️⃣ Invoque le endpoint backend `/api/emergency` pour déclencher les
 *    notifications par email / SMS.
 *
 * La fonction renvoie une `Promise<void>` qui se résout lorsque les deux
 * actions sont terminées (ou rejette en cas d’erreur).  
 * Elle ne s’exécute que côté client (`browser`).
 */

import { browser } from '$app/environment';

/**
 * Envoie une alerte d’urgence.
 *
 * @param message - Texte de l’alerte (ex. « Incendie », « Urgence médicale », …).
 * @returns       Promise qui se résout quand le WebSocket a transmis le
 *                message ET que le backend a répondu.
 * @throws        Erreur si la connexion WebSocket échoue ou si le POST
 *                vers le backend renvoie un statut non‑2xx.
 */
export async function sendEmergencyAlert(message: string): Promise<void> {
  if (!browser) {
    // En SSR on ne fait rien.
    return;
  }

  // -----------------------------------------------------------------
  // 1️⃣ Notification via WebSocket (clients connectés)
  // -----------------------------------------------------------------
  await new Promise<void>((resolve, reject) => {
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const ws = new WebSocket(`${protocol}://${window.location.host}/webrtc/ws`);

    // Timeout de secours (5 s) au cas où le serveur ne répondrait pas.
    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error('Timeout lors de l’envoi de l’alerte d’urgence via WebSocket'));
    }, 5_000);

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'emergency', message }));
      clearTimeout(timeout);
      ws.close(); // on n’a plus besoin de garder la connexion ouverte
      resolve();
    };

    ws.onerror = (ev) => {
      clearTimeout(timeout);
      reject(new Error('Erreur de connexion WebSocket pour l’alerte d’urgence'));
    };

    ws.onclose = (ev) => {
      // Si la connexion se ferme avant `onopen`, on considère que c’est une erreur.
      if (ev.code !== 1000 && ev.wasClean === false) {
        clearTimeout(timeout);
        reject(new Error('WebSocket fermé de manière inattendue pendant l’envoi de l’alerte'));
      }
    };
  });

  // -----------------------------------------------------------------
  // 2️⃣ Notification via le backend (email / SMS)
  // -----------------------------------------------------------------
  const resp = await fetch('/api/emergency', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
  });

  if (!resp.ok) {
    const errText = await resp.text().catch(() => '');
    throw new Error(
      `Erreur du serveur d’urgence : ${resp.status} ${resp.statusText}${
        errText ? ` – ${errText}` : ''
      }`
    );
  }
}