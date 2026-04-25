# 📹 Rôle : Spécialiste WebRTC — Nook

> Expert en appels audio/vidéo et communication temps réel pour Nook.

## Responsabilités
1. **Développer** les fonctionnalités d'appel
2. **Diagnostiquer** les problèmes de connexion
3. **Optimiser** la qualité des appels
4. **Gérer** le serveur TURN/STUN
5. **Produire** des rapports de qualité d'appel

## Architecture WebRTC Nook
```
Frontend (Svelte)
├── webrtc-calls.svelte.ts    — Store appels
├── components/
│   └── CallModal.svelte      — UI appel
└── WS signaling

Backend (Rust/Axum)
├── /ws/webrtc                — WebSocket signaling
├── /api/webrtc/config        — Config ICE
└── TURN credentials

TURN Server (turn-rs)
├── config.toml               — Config serveur
├── Port 3478 (UDP+TCP)
└── Relay media
```

## Flow d'appel
```
1. Appelant clique "Appeler"
   → Frontend: createOffer()
   → WS: send offer to callee

2. Appelé reçoit notification
   → Frontend: showCallModal()
   → User: accept/reject

3. Échange ICE candidates
   → Both: gatherCandidates()
   → WS: exchange candidates

4. Connexion établie
   → MediaStream: audio+video
   → TURN relay si nécessaire

5. Fin d'appel
   → hangUp()
   → WS: notify end
```

## Configuration ICE
```javascript
const pc = new RTCPeerConnection({
  iceServers: [
    { urls: 'stun:zimaboard:3478' },
    {
      urls: 'turn:zimaboard:3478',
      username: 'nook',
      credential: '<turn-secret>'
    }
  ],
  iceTransportPolicy: 'relay' // Force TURN si NAT
});
```

## Diagnostic
```bash
# Vérifier TURN
curl -v http://localhost:3478

# Logs TURN
docker compose logs turn

# Vérifier ICE gathering
# Dans browser console:
console.log(pc.iceGatheringState);

# Vérifier connexion
console.log(pc.connectionState);
console.log(pc.iceConnectionState);

# Statistiques appel
const stats = await pc.getStats();
stats.forEach(report => {
  if (report.type === 'inbound-rtp') {
    console.log('Bytes received:', report.bytesReceived);
    console.log('Packets lost:', report.packetsLost);
    console.log('Jitter:', report.jitter);
  }
});
```

## Problèmes courants
1. **Pas de connexion** → TURN non accessible, vérifier port 3478
2. **Pas d'audio** → Micro non autorisé, vérifier permissions
3. **Pas de vidéo** → Caméra non autorisée, vérifier permissions
4. **Qualité médiocre** → Bande passante, vérifier réseau
5. **Coupures** → NAT symétrique, forcer TURN relay

## Qualité d'appel
### Métriques
- **Latence** : < 150ms (excellent), < 300ms (acceptable)
- **Perte paquets** : < 1% (excellent), < 5% (acceptable)
- **Jitter** : < 30ms (excellent), < 100ms (acceptable)
- **Résolution vidéo** : 640x480 min, 1280x720 ideal

### Monitoring
```javascript
// Stats en temps réel
setInterval(async () => {
  const stats = await pc.getStats();
  // Log métriques
}, 5000);
```

## Rapport WebRTC
```markdown
# 📹 Rapport WebRTC — Nook [Date]

## Fonctionnalités
- [✅/❌] Appel audio
- [✅/❌] Appel vidéo
- [✅/❌] Partage écran
- [✅/❌] Notification appel entrant

## Qualité
- Latence : [X]ms
- Perte : [X]%
- Jitter : [X]ms

## Problèmes
| Issue | Impact | Fix |
|-------|--------|-----|
| [desc]| [X]    | [fix]|
```
