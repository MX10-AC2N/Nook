# Certificats SSL pour Nook

## Structure

```
nginx-local/certs/
├── ca.crt          # Certificat CA (à installer sur les appareils)
├── ca.key          # Clé privée du CA (NE PAS PARTAGER)
├── server.crt      # Certificat du serveur
├── server.key      # Clé privée du serveur
├── server.pem      # Certificat + clé (pour nginx)
├── server.csr      # Certificate Signing Request
└── generate-certs.sh  # Script pour régénérer les certificats
```

## Installation du CA sur les appareils

Pour que les Service Workers et notifications push fonctionnent, vous devez installer le certificat CA (`ca.crt`) sur chaque appareil.

### Android

1. Copiez `ca.crt` sur votre téléphone
2. Allez dans **Paramètres > Sécurité > Chiffrement et données > Installer un certificat**
3. Sélectionnez **"Certificat CA"**
4. Naviguez vers `ca.crt` et installez-le
5. Redémarrez le navigateur

### iOS

1. Copiez `ca.crt` sur votre iPhone
2. Ouvrez le fichier et allez dans **Réglages > Général > Profil**
3. Installez le profil
4. Allez dans **Réglages > Général > À propos > Réglages de confiance des certificats**
5. Activez le certificat **"Nook Local CA"**

### Windows

1. Double-cliquez sur `ca.crt`
2. Cliquez sur **"Installer le certificat"**
3. Sélectionnez **"Ordinateur local"**
4. Placez-le dans **"Autorités de certification racines de confiance"**

### macOS

1. Double-cliquez sur `ca.crt`
2. Ouvrez **"Accès aux clés"**
3. Recherchez **"Nook Local CA"**
4. Double-cliquez et changez la confiance en **"Toujours faire confiance"**

## Configuration nginx

La configuration nginx utilise les certificats suivants :

```nginx
ssl_certificate /etc/nginx/ssl/server.crt;
ssl_certificate_key /etc/nginx/ssl/server.key;
```

## Vérification

Pour vérifier que le certificat fonctionne :

```bash
curl --cacert nginx-local/certs/ca.crt https://192.168.1.192:6443/
```

## Régénération des certificats

Pour régénérer les certificats (par exemple, si l'IP change) :

```bash
cd nginx-local/certs
./generate-certs.sh
```

Puis redémarrez nginx :

```bash
docker compose restart nginx-local
```

## Sécurité

- **Ne partagez jamais `ca.key`** — c'est la clé privée du CA
- Les certificats sont valides :
  - CA : 10 ans (jusqu'en 2036)
  - Serveur : 1 an (à régénérer annuellement)
