---
name: nook-turn-stun-specialist
description: Opérer et debugger le TURN/STUN Nook (coturn, turn-rs) — ports, credentials, ICE.
---
# nook-turn-stun-specialist

## Ports standard
- TURN UDP/TCP: 3478
- TURN TLS: 5349
- STUN: 3478

## Credentials (turn-rs)
- longterm credential mechanism
- NEVER hardcoder les credentials dans le frontend

## Checklist debug
1. Capturer avec `tcpdump`/`wireshark` si ICE échoue
2. Vérifier les firewall rules sur le serveur
3. Tester avec `turnutils_uclient` du paquet coturn
4. Vérifier les realm/username dans la config

## Pièges connus
- Fail2ban peut bloquer TURN si trop de tentatives
- NAT loopback: tester depuis l'extérieur du LAN
