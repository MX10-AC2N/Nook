#!/bin/bash
set -e

cd /tmp/Nook/nginx-local/certs

# 1. Générer la clé privée du CA
openssl genrsa -out ca.key 4096

# 2. Générer le certificat du CA (valide 10 ans)
openssl req -new -x509 -days 3650 -key ca.key -out ca.crt \
  -subj "/CN=Nook Local CA/O=Nook/C=FR"

# 3. Générer la clé privée du serveur
openssl genrsa -out server.key 2048

# 4. Créer un fichier de configuration pour le certificat
cat > server.cnf << 'EOF'
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = 192.168.1.192

[v3_req]
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
IP.1 = 192.168.1.192
DNS.1 = localhost
EOF

# 5. Générer la CSR (Certificate Signing Request)
openssl req -new -key server.key -out server.csr -config server.cnf

# 6. Signer le certificat avec le CA
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out server.crt -extensions v3_req -extfile server.cnf

# 7. Créer un fichier PEM combiné pour nginx
cat server.crt server.key > server.pem

# 8. Vérifier les certificats
echo "=== CA Certificate ==="
openssl x509 -in ca.crt -noout -subject -issuer -dates
echo ""
echo "=== Server Certificate ==="
openssl x509 -in server.crt -noout -subject -issuer -dates
echo ""
echo "=== Subject Alternative Names ==="
openssl x509 -in server.crt -noout -text | grep -A1 "Subject Alternative Name"

echo ""
echo "✅ Certificats générés avec succès !"
