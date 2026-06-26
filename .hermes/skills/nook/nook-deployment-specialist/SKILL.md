---
name: nook-deployment-specialist
description: Skill for the Deployment Specialist agent - Production deployment, Docker, reverse proxy, SSL, Zimaboard self-hosting, monitoring
---

# Nook Deployment Specialist Skill

Use this skill when deploying Nook to production, setting up self-hosting on Zimaboard, configuring reverse proxies, or troubleshooting deployment issues.

## Responsibilities
- Production deployment to Zimaboard (Docker Compose)
- Configure Nginx reverse proxy with HTTPS (Let's Encrypt)
- Set up SSL/TLS certificates, auto-renewal
- Configure environment variables, secrets management
- Monitor deployment health (healthchecks, logs)
- Troubleshoot deployment failures, container crashes
- Document deployment steps for users

## Tools Required
- Docker, Docker Compose
- Nginx (or Nginx Proxy Manager)
- Certbot (Let's Encrypt)
- `gh` CLI (for pulling Docker images from GHCR / triggering workflows)
  - Installation: `apt install gh` (Debian/Ubuntu) 或 `apk add github-cli` (Alpine)
  - Auth: `gh auth login` → PAT classic avec scopes `repo` + `workflow` OU fine-grained avec `repo` contents/actions/workflows
- `curl`, `jq` (for healthchecks)

## Deployment Steps (Zimaboard Self-Hosting)
1. **Prerequisites**:
   - Zimaboard running Debian/Ubuntu
   - Docker + Docker Compose installed (host)
   - Domain name pointing to Zimaboard IP

2. **Pull Images**:
   ```bash
   docker pull ghcr.io/mx10-ac2n/nook:latest
   docker pull ghcr.io/mx10-ac2n/turn-server:latest
   docker pull nginx:alpine3.21
   ```

3. **Configure Environment**:
   - Copy `example.env` to `.env`
   - Set `PUBLIC_SITE_URL=https://your-domain.com`
   - Set `TURN_SECRET` (random string)
   - Set `ADMIN_INITIAL_PASSWORD`
   - Create `turn-config/` directory with `turnserver.conf`

4. **Start Services**:
   ```bash
   docker compose up -d
   ```

5. **Configure Nginx**:
   - Follow `docs/nginx-local.md` for reverse proxy
   - Run Certbot for Let's Encrypt SSL

6. **Verify Health**:
   ```bash
   curl https://your-domain.com/health
   ```

## Docker Compose Configuration
- Ensure `docker-compose.yml` has:
  - Healthchecks for all services
  - Non-root users (USER nook)
  - Persistent volumes for SQLite, uploads, backups
  - Correct TURN server config mount

## Pitfalls
- Zimaboard is ARM64, ensure all images support `linux/arm64`
- TURN server requires UDP ports 3478 open on firewall
- SQLite database needs persistent volume to avoid data loss
- Let's Encrypt requires port 80 open for challenge
- **HSTS on HTTP behind reverse proxy**: Backend must check `x-forwarded-proto` before sending HSTS header. See `references/hsts-reverse-proxy.md`.

## Verification
- [ ] All containers running (`docker compose ps`)
- [ ] HTTPS working with valid SSL certificate
- [ ] WebRTC calls work for remote users
- [ ] SQLite database persists after container restart
- [ ] Health endpoint returns 200 OK
