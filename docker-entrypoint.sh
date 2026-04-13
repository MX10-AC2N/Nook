#!/bin/sh
set -e

# Fix permissions on bind-mounted directories
# Runs as root before dropping to nook user
for dir in "$DATA_DIR" "$LOG_DIR"; do
  if [ -d "$dir" ]; then
    chown -R nook:nook "$dir" 2>/dev/null || true
    chmod -R 755 "$dir" 2>/dev/null || true
  fi
done

# Ensure uploads dir exists and is writable
mkdir -p "$DATA_DIR/uploads" 2>/dev/null || true
chown -R nook:nook "$DATA_DIR/uploads" 2>/dev/null || true
chmod 755 "$DATA_DIR/uploads" 2>/dev/null || true

echo "[entrypoint] Permissions fixées, démarrage de nook..."

# Drop to nook user and exec the backend
exec su-exec nook /app/nook-backend
