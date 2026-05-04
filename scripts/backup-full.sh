#!/bin/bash
# Full Nook backup: DB + config + .hermes
# Usage: ./backup-full.sh [backup_dir]

BACKUP_DIR="${1:-./backups}"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/nook_full_$DATE.tar.gz"

mkdir -p "$BACKUP_DIR"

tar -czf "$BACKUP_FILE" \
  "${NOOK_DB_PATH:-/data/nook.db}" \
  .env \
  docker-compose.yml \
  turn-config/ \
  .hermes/ \
  frontend/static/ \
  backend/migrations/

echo "Full backup created: $BACKUP_FILE"
ls -lh "$BACKUP_FILE"
