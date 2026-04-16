#!/bin/sh
set -e
CERT_DIR=/etc/nginx/ssl
mkdir -p "$CERT_DIR"
if [ ! -f "$CERT_DIR/nook.crt" ] || [ ! -f "$CERT_DIR/nook.key" ]; then
  HOSTNAME=$(hostname -i 2>/dev/null || echo "192.168.1.192")
  echo "Génération du certificat auto-signé pour $HOSTNAME..."
  openssl req -x509 -nodes -days 3650 \
    -newkey rsa:2048 \
    -keyout "$CERT_DIR/nook.key" \
    -out "$CERT_DIR/nook.crt" \
    -subj "/CN=$HOSTNAME" \
    -addext "subjectAltName=IP:$HOSTNAME,IP:127.0.0.1,DNS:nook.local" \
    2>/dev/null
  echo "Certificat créé."
fi
exec nginx -g 'daemon off;'
