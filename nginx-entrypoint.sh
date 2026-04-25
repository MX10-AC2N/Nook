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

# Copier le CA dans un dossier servi par nginx
CA_SERVE_DIR=/usr/share/nginx/html/ca
mkdir -p "$CA_SERVE_DIR"
cp "$CERT_DIR/ca.crt" "$CA_SERVE_DIR/nook-ca.crt"

# Créer la page HTML d'aide
cat > "$CA_SERVE_DIR/index.html" << 'HTMLEOF'
<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Nook - Installation du certificat CA</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; color: #333; line-height: 1.6; padding: 20px; }
    .container { max-width: 600px; margin: 0 auto; background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
    h1 { color: #2d5016; margin-bottom: 8px; }
    .subtitle { color: #666; margin-bottom: 24px; }
    .download-btn { display: block; background: #2d5016; color: white; text-align: center; padding: 14px; border-radius: 8px; text-decoration: none; font-weight: bold; margin-bottom: 24px; }
    .download-btn:hover { background: #3a6a1c; }
    .steps { margin-bottom: 24px; }
    .step { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 16px; }
    .step-num { background: #2d5016; color: white; width: 28px; height: 28px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: bold; flex-shrink: 0; }
    .step-text { flex: 1; }
    .step-text strong { display: block; margin-bottom: 4px; }
    .note { background: #fff3cd; border: 1px solid #ffc107; border-radius: 8px; padding: 12px; font-size: 14px; }
    .tabs { display: flex; gap: 8px; margin-bottom: 16px; }
    .tab { padding: 8px 16px; border-radius: 8px; background: #eee; cursor: pointer; border: none; font-size: 14px; }
    .tab.active { background: #2d5016; color: white; }
    .tab-content { display: none; }
    .tab-content.active { display: block; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔒 Certificat CA Nook</h1>
    <p class="subtitle">Pour activer les notifications push, installez ce certificat sur votre appareil.</p>
    
    <a href="/ca/nook-ca.crt" download class="download-btn">⬇️ Télécharger le certificat CA</a>
    
    <div class="tabs">
      <button class="tab active" onclick="showTab('android')">Android</button>
      <button class="tab" onclick="showTab('ios')">iOS / iPhone</button>
      <button class="tab" onclick="showTab('pc')">PC / Mac</button>
    </div>
    
    <div id="android" class="tab-content active">
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <div class="step-text"><strong>Téléchargez</strong> le certificat via le bouton ci-dessus</div>
        </div>
        <div class="step">
          <div class="step-num">2</div>
          <div class="step-text"><strong>Paramètres → Sécurité → Chiffrement et données → Installer un certificat</strong></div>
        </div>
        <div class="step">
          <div class="step-num">3</div>
          <div class="step-text">Sélectionnez <strong>"Certificat CA"</strong></div>
        </div>
        <div class="step">
          <div class="step-num">4</div>
          <div class="step-text">Naviguez vers <strong>nook-ca.crt</strong> et confirmez</div>
        </div>
        <div class="step">
          <div class="step-num">5</div>
          <div class="step-text"><strong>Redémarrez</strong> votre navigateur, puis réessayez les notifications</div>
        </div>
      </div>
      <div class="note">⚠️ Sur Samsung : Paramètres → Biométrie et sécurité → Autres paramètres de sécurité → Certificats → Installer depuis le stockage</div>
    </div>
    
    <div id="ios" class="tab-content">
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <div class="step-text"><strong>Téléchargez</strong> le certificat via le bouton ci-dessus</div>
        </div>
        <div class="step">
          <div class="step-num">2</div>
          <div class="step-text"><strong>Réglages → Général → VPN et gestion de l'appareil</strong></div>
        </div>
        <div class="step">
          <div class="step-num">3</div>
          <div class="step-text">Appuyez sur <strong>"Nook Local CA"</strong> puis <strong>"Installer"</strong></div>
        </div>
        <div class="step">
          <div class="step-num">4</div>
          <div class="step-text"><strong>Réglages → Général → À propos → Réglages de confiance des certificats</strong></div>
        </div>
        <div class="step">
          <div class="step-num">5</div>
          <div class="step-text">Activez le certificat <strong>"Nook Local CA"</strong></div>
        </div>
      </div>
      <div class="note">⚠️ iOS affichera un avertissement de sécurité — c'est normal, c'est votre propre CA</div>
    </div>
    
    <div id="pc" class="tab-content">
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <div class="step-text"><strong>Téléchargez</strong> le certificat via le bouton ci-dessus</div>
        </div>
        <div class="step">
          <div class="step-num">2</div>
          <div class="step-text"><strong>Windows :</strong> Double-cliquez → "Installer le certificat" → "Ordinateur local" → "Autorités de certification racines de confiance"</div>
        </div>
        <div class="step">
          <div class="step-num">3</div>
          <div class="step-text"><strong>Mac :</strong> Double-cliquez → Ouvrez "Accès aux clés" → Changez en "Toujours faire confiance"</div>
        </div>
      </div>
      <div class="note">⚠️ Chrome utilise les certificats système. Firefox utilise son propre magasin — importez-le via Paramètres → Vie privée → Certificats</div>
    </div>
    
    <div class="note" style="margin-top: 24px;">
      📱 <strong>Après installation :</strong> revenez sur Nook et cliquez à nouveau sur "Activer les notifications"
    </div>
  </div>
  
  <script>
    function showTab(id) {
      document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.tab').forEach(el => el.classList.remove('active'));
      document.getElementById(id).classList.add('active');
      event.target.classList.add('active');
    }
  </script>
</body>
</html>
HTMLEOF

echo "Page d'aide créée : $CA_SERVE_DIR/index.html"

echo "Démarrage nginx..."
exec nginx -g 'daemon off;'
