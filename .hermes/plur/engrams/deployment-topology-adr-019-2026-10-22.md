---
type: architectural
scope: global
domain: nook.architecture.deployment-topology
tags: [nook, architect, 2026-10-22, adr-019, adr-001, adr-007, deployment-topology, plur-convention]
statement: "Topologie de déploiement Nook (ADR-019, 2026-10-22, carte t_107d5ece) : un seul hôte, 3 services conteneurisés — nook, nginx-local, turn (docker-compose.yml:2,58,88 ; docker-compose.ci.yml:2,58,86) — et la contrainte de simplicité porte sur le budget RAM/CPU (MEMORY_LIMIT:-1028M, CPUS_LIMIT:-1.5, docker-compose.yml:39-46) et non sur le nombre de conteneurs ; TLS LAN = nginx-local (nginx-local.conf, Dockerfile.nginx, gen-cert.sh), TLS WAN = reverse proxy externe hors périmètre (NPM/Caddy/Traefik au choix) que Nook honore via X-Forwarded-Proto (backend/src/auth.rs:184-216) ; ADR-001 (« single container ») et ADR-007 (« NPM for LAN/WAN TLS ») sont Superseded — la garde-fou est REFORMULÉE sur une contrainte mesurable, pas supprimée, et la formulation « aucun artefact NPM » est imprécise (NPM est cité comme recommandation WAN dans README.md:108,188,199, AGENTS.md:47, docs/nginx-local.md:57-69)."
---
