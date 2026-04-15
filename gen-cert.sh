#!/bin/sh
# Génère un certificat auto-signé pour Nook en HTTPS local
# Usage: sudo ./gen-cert.sh [IP]

IP="${1:-192.168.1.192}"
DIR="$(cd "$(dirname "$0")" && pwd)/nginx-ssl"
mkdir -p "$DIR"
chmod 755 "$DIR"

openssl req -x509 -nodes -days 3650 \
  -newkey rsa:2048 \
  -keyout "$DIR/nook.key" \
  -out "$DIR/nook.crt" \
  -subj "/CN=$IP" \
  -addext "subjectAltName=IP:$IP,IP:127.0.0.1,DNS:nook.local"

chmod 644 "$DIR/nook.crt"
chmod 600 "$DIR/nook.key"

echo "Certificat créé dans $DIR/"
echo "  nook.crt  (certificat)"
echo "  nook.key  (clé privée)"
echo ""
echo "Accédez à https://$IP:6443 depuis votre LAN"
echo "Acceptez l'avertissement du certificat auto-signé"
