// Envoie une alerte à tous les membres via WebSocket + email/SMS (backend)
export async function sendEmergencyAlert(message) {
  // 1. WebSocket à tous les membres connectés
  const ws = new WebSocket(`wss://${window.location.host}/ws`);
  ws.onopen = () => {
    ws.send(JSON.stringify({ type: 'emergency', message }));
  };

  // 2. Appel backend pour email/SMS
  await fetch('/api/emergency', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message })
  });
}