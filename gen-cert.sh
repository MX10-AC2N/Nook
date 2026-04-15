#!/bin/sh
# Génère un certificat auto-signé pour Nook en HTTPS local
# Usage: ./gen-cert.sh [IP]

IP="${1:-192.168.1.192}"
DIR="$(dirname "$0")/nginx-ssl"
mkdir -p "$DIR"

openssl req -x509 -nodes -days 3650 \
  -newkey rsa:2048 \
  -keyout "$DIR/nook.key" \
  -out "$DIR/nook.crt" \
  -subj "/CN=$IP" \
  -addext "subjectAltName=IP:$IP,IP:127.0.0.1,DNS:nook.local"

echo "Certificat créé dans $DIR/"
echo "  nook.crt  (certificat)"
echo "  nook.key  (clé privée)"
echo ""
echo "Accédez à https://$IP:6443 depuis votre LAN"
