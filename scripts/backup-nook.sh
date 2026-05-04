#!/bin/bash
# backup-nook.sh — Script de sauvegarde pour Nook
# Sauvegarde la DB SQLite, les configs, et les fichiers utilisateurs

set -euo pipefail

# Configuration
NOOK_DIR="/opt/data/home/.hermes/Nook"
BACKUP_DIR="/opt/data/backups/nook"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="$BACKUP_DIR/nook-backup-$DATE.tar.gz"

# Variables d'env (pour les secrets)
source "$NOOK_DIR/.env" 2>/dev/null || true

mkdir -p "$BACKUP_DIR"

echo "📦 Début de la sauvegarde Nook : $DATE"

# 1. Sauvegarde de la base SQLite (backend)
SQLITE_DB="$NOOK_DIR/backend/nook.db"
if [ -f "$SQLITE_DB" ]; then
  echo "  → Sauvegarde SQLite DB : $SQLITE_DB"
  sqlite3 "$SQLITE_DB" ".backup $BACKUP_DIR/nook-$DATE.db"
else
  echo "  ⚠️ SQLite DB non trouvée, skipping"
fi

# 2. Sauvegarde des fichiers de config
echo "  → Sauvegarde des configs"
tar -czf "$BACKUP_FILE" \
  -C "$NOOK_DIR" \
  turn-config/ \
  backend/.env.example \
  frontend/.env.example \
  docker-compose.yml \
  nginx-ssl/ \
  2>/dev/null || true

# 3. Sauvegarde des fichiers uploadés (si existants)
UPLOAD_DIR="$NOOK_DIR/backend/uploads"
if [ -d "$UPLOAD_DIR" ]; then
  echo "  → Sauvegarde des uploads"
  tar -czf "$BACKUP_DIR/nook-uploads-$DATE.tar.gz" -C "$UPLOAD_DIR" .
fi

# 4. Nettoyage des vieux backups (garde les 7 derniers)
echo "  → Nettoyage des vieux backups"
ls -t "$BACKUP_DIR"/nook-backup-*.tar.gz | tail -n +8 | xargs -d '\n' rm -f 2>/dev/null || true
ls -t "$BACKUP_DIR"/nook-*.db | tail -n +8 | xargs -d '\n' rm -f 2>/dev/null || true

echo "✅ Sauvegarde terminée : $BACKUP_FILE"
ls -lh "$BACKUP_FILE" "$BACKUP_DIR/nook-$DATE.db" 2>/dev/null