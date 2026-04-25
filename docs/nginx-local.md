# HTTPS local pour Nook — Configuration

## Pourquoi ?

L'enregistrement audio/vidéo dans le navigateur (`getUserMedia`) nécessite un **contexte sécurisé** :
- HTTPS avec certificat valide
- ou localhost

Sur HTTP LAN (ex: `http://192.168.1.192:6300`), le navigateur bloque l'accès au microphone.

## Solution

Nook inclut un conteneur **nginx** qui sert de reverse proxy HTTPS local sur le LAN.

## Installation

### 1. Créer le dossier des certificats

```bash
mkdir -p nginx-ssl
```

### 2. Démarrer

```bash
docker compose up -d
```

Le certificat auto-signé est **généré automatiquement** au premier lancement.

Ou générez-le manuellement :

```bash
./gen-cert.sh 192.168.1.192
```

### 3. Accéder à Nook en HTTPS

```
https://192.168.1.192:6443
```

Acceptez l'avertissement du certificat auto-signé (le certificat est valide pendant 10 ans).

### 4. Configurer le port (optionnel)

Dans `.env` :

```env
NGINX_HTTPS_PORT=6443
```

## Architecture

```
LAN (HTTPS)                    WAN (HTTPS)
https://192.168.1.192:6443      https://ton-domaine.com
       │                              │
   ┌───┴───┐                   ┌──────┴──────┐
   │ nginx │                   │ nginx proxy │
   │ local │                   │  manager    │
   └───┬───┘                   └──────┬──────┘
       │                              │
       └──────────┬───────────────────┘
                  │
             ┌────┴────┐
             │  Nook   │ :3000
             └─────────┘
```

## Certificat auto-signé

Le certificat est généré avec :
- Validité : 10 ans
- CN : IP de la machine
- SAN : IP + localhost + nook.local

Le dossier `nginx-ssl/` est **persistant** (volume Docker) et **ignoré par git** (`.gitignore`).

## Dépannage

### Le port 6443 est déjà utilisé

```env
NGINX_HTTPS_PORT=8443
```

### Recréer le certificat

```bash
rm -rf nginx-ssl/
./gen-cert.sh 192.168.1.192
docker compose restart nginx-local
```

### Le navigateur refuse le certificat

C'est normal avec un certificat auto-signé. Cliquez "Avancé" → "Continuer vers le site".

Sur mobile, le certificat est accepté après le premier avertissement.
