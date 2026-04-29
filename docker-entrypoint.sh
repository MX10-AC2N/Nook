#!/bin/sh
set -e

# Fix permissions on ALL writable directories
for dir in "$DATA_DIR" "$LOG_DIR" "$STATIC_DIR"; do
  if [ -d "$dir" ]; then
    chown -R nook:nook "$dir" 2>/dev/null || true
    chmod -R 755 "$dir" 2>/dev/null || true
  fi
done

mkdir -p "$DATA_DIR/uploads" 2>/dev/null || true
chown -R nook:nook "$DATA_DIR/uploads" 2>/dev/null || true

echo "[entrypoint] Permissions fixees, demarrage de nook..."

exec /app/nook-backend
