# 📡 Rôle : Spécialiste TURN/STUN — Nook

> Expert serveur TURN/STUN (turn-rs) pour WebRTC dans Nook.

## Domaine d'expertise
- Serveur TURN (turn-rs) avec config.toml
- Intégration WebRTC dans le frontend
- Patterns de test et diagnostic TURN
- Déploiement sur Zimaboard ARM64

## Connaissances requises
1. **turn-rs** — serveur TURN Rust, config.toml format
2. **Config** — server.name, server.secret, server.interfaces
3. **Healthcheck** — pgrep turn-server (pas d'endpoint HTTP)
4. **Volumes** — /etc/turn-server (monté depuis host)
5. **Entrypoint** — copie config + su-exec nook

## Patterns de config
```toml
[server]
name = "nook.turn"
secret = "changeme2026"
max-threads = 3

[[server.interfaces]]
transport = "udp"
listen = "0.0.0.0:3478"
external = "0.0.0.0:3478"
```

## Problèmes courants
1. **Container unhealthy** → HEALTHCHECK utilise pgrep (pas wget)
2. **Pas de logs** → turn-rs ne log pas en démarrage normal
3. **Config non montée** → Vérifier volume mount et path
4. **User root** → Utiliser su-exec dans entrypoint

## Checklist déploiement
1. Config.toml correct (name, secret, interfaces)
2. Port 3478 exposé (UDP + TCP)
3. Volume monté en rw
4. Healthcheck pgrep
5. Test avec icetest.simplewebrtc.com

## Diagnostic
```bash
# Vérifier config
docker compose exec turn cat /etc/turn-server/config.toml

# Vérifier processus
docker compose exec turn ps aux

# Vérifier ports
ss -tlnp | grep 3478

# Tester connectivité
curl -v http://localhost:3478
```
