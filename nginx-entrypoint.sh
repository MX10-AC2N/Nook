#!/bin/sh
set -e
CERT_DIR=/etc/nginx/ssl
mkdir -p "$CERT_DIR"
if [ ! -f "$CERT_DIR/nook.crt" ] || [ ! -f "$CERT_DIR/nook.key" ]; then
  # Utiliser HOST_IP si fourni, sinon extraire depuis PUBLIC_SITE_URL, sinon fallback
  if [ -n "$HOST_IP" ]; then
    IP="$HOST_IP"
  else
    IP=$(echo "$PUBLIC_SITE_URL" | sed -n 's/.*\/\/\([0-9.]*\).*/\1/p')
    [ -z "$IP" ] && IP="192.168.1.192"
  fi
  echo "Génération du certificat auto-signé pour $IP..."
  openssl req -x509 -nodes -days 3650 \
    -newkey rsa:2048 \
    -keyout "$CERT_DIR/nook.key" \
    -out "$CERT_DIR/nook.crt" \
    -subj "/CN=$IP" \
    -addext "subjectAltName=IP:$IP,IP:127.0.0.1,DNS:nook.local" \
    2>/dev/null
  echo "Certificat créé."
fi
exec nginx -g 'daemon off;'
