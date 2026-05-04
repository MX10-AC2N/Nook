# Nook Disaster Recovery Plan

## Overview
This document outlines procedures to recover Nook from:
- Database corruption/loss
- Disk failure on Zimaboard
- Accidental deletion of config/data
- Security breach requiring restore

---

## 1. Backup Strategy (3-2-1 Rule)
- **3 copies** of data: Live DB + local backup + remote backup
- **2 different media**: SQLite DB + compressed tar backup
- **1 offsite**: Push backups to GitHub Releases or cloud storage

### Automated Backups
Run daily via cron:
```bash
# Add to crontab (crontab -e)
0 2 * * * /opt/data/home/.hermes/Nook/scripts/backup-full.sh /backups
```

---

## 2. Recovery Procedures

### Scenario A: SQLite DB Corrupted/Lost
1. Stop all containers:
   ```bash
   cd /opt/data/home/.hermes/Nook
   docker compose down
   ```

2. Restore latest backup:
   ```bash
   LATEST=$(ls -t /backups/nook_db_*.sqlite.gz | head -1)
   gunzip -c "$LATEST" > /data/nook.db
   ```

3. Verify DB integrity:
   ```bash
   sqlite3 /data/nook.db "PRAGMA integrity_check;"
   ```

4. Restart containers:
   ```bash
   docker compose up -d
   ```

### Scenario B: Disk Failure (Zimaboard)
1. Replace disk, reinstall OS (Debian 13)
2. Install Docker:
   ```bash
   apt-get update && apt-get install -y docker.io docker-compose
   ```

3. Clone Nook repo:
   ```bash
   git clone https://github.com/MX10-AC2N/Nook.git /opt/data/home/.hermes/Nook
   cd /opt/data/home/.hermes/Nook
   git checkout develop
   ```

4. Restore latest full backup:
   ```bash
   LATEST=$(ls -t /backups/nook_full_*.tar.gz | head -1)
   tar -xzf "$LATEST" -C /
   ```

5. Start services:
   ```bash
   cp .env.example .env
   # Edit .env with TURN_SECRET, ADMIN_INITIAL_PASSWORD
   docker compose up -d
   ```

### Scenario C: Accidental Config Deletion
1. Restore from Git:
   ```bash
   cd /opt/data/home/.hermes/Nook
   git checkout -- .env docker-compose.yml turn-config/
   ```

2. Restart services:
   ```bash
   docker compose up -d
   ```

---

## 3. Verification Steps
After any recovery:
1. Check container health: `docker ps`
2. Test API: `curl http://localhost:3000/api/health`
3. Test WebRTC: Check TURN server logs `docker logs nook-turn`
4. Verify frontend: Open `https://localhost:6443`

---

## 4. Emergency Contacts
- Admin: admin@nook.app
- Repo: https://github.com/MX10-AC2N/Nook/issues
