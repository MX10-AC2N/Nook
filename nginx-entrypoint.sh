#!/bin/sh
CERT_DIR=/etc/nginx/ssl
mkdir -p "$CERT_DIR"

# Utiliser HOST_IP si fourni, sinon extraire depuis PUBLIC_SITE_URL, sinon fallback
if [ -n "$HOST_IP" ]; then
  IP="$HOST_IP"
else
  IP=$(echo "$PUBLIC_SITE_URL" | sed -n 's/.*\/\/\([0-9.]*\).*/\1/p')
  [ -z "$IP" ] && IP="192.168.1.192"
fi

# Générer le CA s'il n'existe pas (une seule fois, persisté via volume)
if [ ! -f "$CERT_DIR/ca.crt" ] || [ ! -f "$CERT_DIR/ca.key" ]; then
  echo "Génération du CA local..."
  openssl genrsa -out "$CERT_DIR/ca.key" 4096 2>/dev/null
  openssl req -new -x509 -days 3650 -key "$CERT_DIR/ca.key" -out "$CERT_DIR/ca.crt" \
    -subj "/CN=Nook Local CA/O=Nook/C=FR" 2>/dev/null
  echo "CA créé : $CERT_DIR/ca.crt"
fi

# Générer le certificat serveur s'il n'existe pas (ou si l'IP a changé)
NEED_CERT=0
if [ ! -f "$CERT_DIR/nook.crt" ] || [ ! -f "$CERT_DIR/nook.key" ]; then
  NEED_CERT=1
fi

# Vérifier si l'IP dans le SAN correspond à HOST_IP actuel
if [ -f "$CERT_DIR/nook.crt" ]; then
  CURRENT_SAN=$(openssl x509 -in "$CERT_DIR/nook.crt" -noout -text 2>/dev/null | grep -o "IP Address:$IP" || true)
  if [ -z "$CURRENT_SAN" ]; then
    echo "IP changée ($IP), régénération du certificat..."
    NEED_CERT=1
  fi
fi

if [ "$NEED_CERT" = "1" ]; then
  echo "Génération du certificat serveur pour $IP (signé par le CA local)..."
  
  # Clé privée du serveur
  openssl genrsa -out "$CERT_DIR/nook.key" 2048 2>/dev/null
  
  # CSR
  openssl req -new -key "$CERT_DIR/nook.key" -out "$CERT_DIR/nook.csr" \
    -subj "/CN=$IP" 2>/dev/null
  
  # Signer avec le CA
  openssl x509 -req -days 365 -in "$CERT_DIR/nook.csr" \
    -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial \
    -out "$CERT_DIR/nook.crt" \
    -extfile <(printf "subjectAltName=IP:$IP,IP:127.0.0.1,DNS:localhost,DNS:nook.local") \
    2>/dev/null || {
    # Fallback si -extfile ne marche pas (busybox)
    openssl x509 -req -days 365 -in "$CERT_DIR/nook.csr" \
      -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial \
      -out "$CERT_DIR/nook.crt" 2>/dev/null
  }
  
  rm -f "$CERT_DIR/nook.csr"
  echo "Certificat créé : $CERT_DIR/nook.crt (signé par CA local)"
fi

# Vérifier que les certificats existent
if [ ! -f "$CERT_DIR/nook.crt" ] || [ ! -f "$CERT_DIR/ca.crt" ]; then
  echo "ERREUR: certificats introuvables dans $CERT_DIR"
  ls -la "$CERT_DIR"
  exit 1
fi

# Copier le CA dans un dossier servi par nginx pour téléchargement
CA_SERVE_DIR=/usr/share/nginx/html/ca
mkdir -p "$CA_SERVE_DIR"
cp "$CERT_DIR/ca.crt" "$CA_SERVE_DIR/nook-ca.crt"

echo "Démarrage nginx..."
exec nginx -g 'daemon off;'
