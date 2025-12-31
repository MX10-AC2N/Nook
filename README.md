# ⚠️ WORK IN PROGRESS ⚠️

# Nook - La messagerie qui protège ta famille 🏠

Salut ! Bienvenue sur Nook, un projet construit pour que ta famille puisse discuter en toute tranquillité, sans que personne ne vienne fouiner dans tes conversations.

![Nook](https://img.shields.io/badge/Nook_pour-ta_famille-blue)
![Message](https://img.shields.io/badge/build%20with%20🫶-8A2BE2)
![License](https://img.shields.io/badge/License-MIT-green)

## Pourquoi Nook ?

Tu en as marre que les grandes firmes vendent tes données ? Que tes messages servent à alimenter leurs pubs ciblées ? Nous aussi. 😤

Nook, c'est simple :
- C'est **toi** qui héberges l'application (sur ton serveur, ton NAS, ou même un Raspberry Pi)
- C'est **toi** qui controlles tes données (elles ne vont nulle part ailleurs)
- C'est **chiffré** de bout en bout (même nous on ne peut pas lire tes messages)

## Ce que Nook sait faire

### 💬 Discuter en toute liberté
Des conversations textuelles simples, rapides, et surtout privées.

### 📞 S'entendre et se voir
Appels audio et vidéo pour garder le contact avec ceux qu'on aime, même quand ils sont loin.

### 📁 Partager sans complexe
Photos de famille, vidéos des kids, documents importants... Tout ce que tu partages reste entre vous.

### 🔐 Sécurité de ninja
On utilise libsodium pour chiffrer tes messages. Tes clés sont générées chez toi, stockées chez toi. C'est un peu comme un coffre-fort dont seul toi as la clé.

## Démarrer en 3 étapes

### 1. Récupérer le projet

```
git clone https://github.com/MX10-AC2N/Nook.git
cd Nook
```

### 2. Lancer avec Docker

```
docker-compose up --build -d
```

### 3. Ouvrir ton navigateur

Va sur `http://localhost:3000` et c'est parti ! 🎉

## Ce qu'il faut

- Docker et Docker Compose (c'est tout !)
- Un navigateur récent (Chrome, Firefox, Safari, Edge... ça marche partout)

## Comment ça marche ?

```
Nook/
├── backend/          # Le cerveau en Rust (rapide et léger)
├── frontend/         # La tête en SvelteKit (jolie et fluide)
└── Dockerfile        # Tout打包 pour déployer facilement
```

## FAQ

**"C'est compliqué à installer ?"**
Pas du tout ! Docker s'occupe de tout. Une fois que c'est lancé, tu n'as plus rien à faire.

**"Mes données sont où ?"**
Sur ta machine ! Aucune donnée ne part sur un serveur tiers. C'est le principe de l'auto-hébergement.

**"Je peux l'utiliser sur mon NAS ?"**
Absolument ! Tant que tu peux y faire tourner Docker, Nook fonctionnera.

**"Les appels vidéo ça consomme beaucoup ?"**
On utilise WebRTC pour des connexions directes entre les appareils. C'est efficace et respectueux de ta bande passante.

## Besoin d'aide ?

Tu bloques sur quelque chose ? Ouvre une issue sur GitHub, on sera ravis de t'aider !

## Le mot de la fin

Nook, c'est un projet qu'on a build avec l'envie de proposer une alternative aux géants du web. Pas de pub, pas de tracking, pas de revente de données. Juste un outil pour que ta famille puisse communiquer en toute sérénité.

Amuse-toi bien ! 🎈
