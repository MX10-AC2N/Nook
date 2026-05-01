#!/bin/sh
# Nook SSL entrypoint — génère le CA + certificat serveur automatiquement
# Doit tourner en root (pas de USER nginx dans Dockerfile.nginx)

CERT_DIR=/etc/nginx/ssl
mkdir -p "$CERT_DIR"

# Utiliser HOST_IP si fourni, sinon extraire depuis PUBLIC_SITE_URL, sinon fallback
if [ -n "$HOST_IP" ]; then
  IP="$HOST_IP"
else
  IP=$(echo "$PUBLIC_SITE_URL" | sed -n 's/.*\/\/\([0-9.]*\).*/\1/p')
  [ -z "$IP" ] && IP="192.168.1.192"
fi

echo "IP détectée : $IP"

# Générer le CA s'il n'existe pas (une seule fois, persisté via volume)
if [ ! -f "$CERT_DIR/ca.crt" ] || [ ! -f "$CERT_DIR/ca.key" ]; then
  echo "Génération du CA local..."
  openssl genrsa -out "$CERT_DIR/ca.key" 4096 2>/dev/null
  openssl req -new -x509 -days 3650 -key "$CERT_DIR/ca.key" -out "$CERT_DIR/ca.crt" \
    -subj "/CN=Nook Local CA/O=Nook/C=FR" 2>/dev/null
  echo "CA créé : $CERT_DIR/ca.crt"
  # Copier le CA pour téléchargement
  mkdir -p /usr/share/nginx/html/ca
  cp "$CERT_DIR/ca.crt" /usr/share/nginx/html/ca/nook-ca.crt
  echo "CA copié vers /usr/share/nginx/html/ca/nook-ca.crt"
fi

# Générer le certificat serveur s'il n'existe pas (ou si l'IP a changé)
NEED_CERT=0
if [ ! -f "$CERT_DIR/nook.crt" ] || [ ! -f "$CERT_DIR/nook.key" ]; then
  NEED_CERT=1
fi

# Vérifier si l'IP dans le SAN correspond à HOST_IP actuel
if [ -f "$CERT_DIR/nook.crt" ]; then
  CURRENT_SAN=$(openssl x509 -in "$CERT_DIR/nook.crt" -text -noout 2>/dev/null | grep -A1 "Subject Alternative Name" | tail -1 | sed 's/.*IP://;s/[[:space:]]//g')
  if [ "$CURRENT_SAN" != "$IP" ]; then
    echo "IP changée : était $CURRENT_SAN, maintenant $IP"
    NEED_CERT=1
  fi
fi

if [ $NEED_CERT -eq 1 ]; then
  echo "Génération du certificat serveur pour $IP..."
  openssl genrsa -out "$CERT_DIR/nook.key" 2048 2>/dev/null
  openssl req -new -key "$CERT_DIR/nook.key" -out "$CERT_DIR/nook.csr" \
    -subj "/CN=$IP/O=Nook/C=FR" 2>/dev/null
  
  # Créer un fichier de config SAN
  echo "subjectAltName=IP:$IP" > /tmp/extfile
  openssl x509 -req -in "$CERT_DIR/nook.csr" -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" \
    -CAcreateserial -out "$CERT_DIR/nook.crt" -days 3650 -extfile /tmp/extfile 2>/dev/null
  echo "Certificat serveur créé pour $IP"
fi

# S'assurer que nginx peut lire les fichiers (on est root, on chmod + chown)
chmod 644 "$CERT_DIR/nook.crt" "$CERT_DIR/nook.key" 2>/dev/null
chown nginx-user:nginx-user "$CERT_DIR/nook.crt" "$CERT_DIR/nook.key" 2>/dev/null

echo "Démarrage nginx..."
# nginx -g "daemon off;"  # Nginx gère le passage root→nginx tout seul
# En fait, on lance juste nginx en avant-plan
exec nginx -g "daemon off;"
