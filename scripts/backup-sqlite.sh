#!/bin/bash
# Backup Nook SQLite database
# Usage: ./backup-sqlite.sh [backup_dir]

BACKUP_DIR="${1:-./backups}"
DATE=$(date +%Y%m%d_%H%M%S)
DB_PATH="${NOOK_DB_PATH:-/data/nook.db}"
BACKUP_FILE="$BACKUP_DIR/nook_db_$DATE.sqlite"

mkdir -p "$BACKUP_DIR"

if [ ! -f "$DB_PATH" ]; then
  echo "Error: Database not found at $DB_PATH"
  exit 1
fi

sqlite3 "$DB_PATH" ".backup $BACKUP_FILE"
gzip "$BACKUP_FILE"

echo "SQLite backup created: ${BACKUP_FILE}.gz"
ls -lh "${BACKUP_FILE}.gz"
