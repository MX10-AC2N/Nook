# 🌿 Nook — Guide de l’utilisateur

Nook est une **messagerie instantanée privée, sécurisée et familiale**, **auto-hébergée** sur ton NAS (CasaOS).  
Aucune donnée ne quitte ton réseau. Aucun compte n’est nécessaire. Tout est chiffré de bout en bout.

---

## 🔐 1. Accès initial

1. Ouvre **`https://nook.votre-domaine.com`** (ou `http://votre-nas:3000`)
2. Si tu es **l’administrateur** :
   - Le **token admin** se trouve dans `/casaos/appdata/nook/data/admin.token`
   - Accède à **`/admin`** pour configurer ton espace
3. Si tu es **invité** :
   - Clique sur le lien d’invitation reçu (ex: `https://nook.../join?token=abc123`)
   - Saisis ton **prénom**
   - Attends l’**approbation de l’administrateur**

---

## 👥 2. Gestion des membres (Admin uniquement)

### ✅ Inviter un membre
- Clique sur **“Inviter un membre”**
- Copie le lien généré → envoie-le par SMS, email, WhatsApp…
- Le destinataire pourra **demander à rejoindre**

### ✅ Approuver un membre
- Dans **`/admin`**, trouve la demande en attente
- Clique sur **“Approuver”**
- Le membre reçoit une notification et peut discuter

> 🔒 **Aucun accès n’est possible sans validation admin**

---

## 💬 3. Utilisation du chat

### Envoyer un message
- Tape ton message → **Enter** ou **“Envoyer”**
- Les messages sont **chiffrés automatiquement**

### Réagir
- Clique sur un message → ajoute **👍** ou **❤️**

### Envoyer un GIF
- Clique sur **“GIF”**
- Recherche (ex: “chat”, “danse”)
- Clique sur un GIF → il est **chiffré et envoyé**

### Envoyer un fichier
- Fichiers ≤ **50 Mo** : upload direct (chiffré)
- Fichiers > **50 Mo** : envoi **P2P direct** (pas de stockage serveur)

---

## 📞 4. Appels audio/vidéo

1. Clique sur **“Appeler”** dans une conversation
2. Autorise **micro & caméra** si demandé
3. L’appel commence **directement entre appareils** (P2P)
4. **Aucune donnée ne transite par ton serveur** → maximum de sécurité

> 📱 Disponible sur **mobile et PC**

---

## 🗓️ 5. Calendrier & Rendez-vous

- Clique sur **“Calendrier”**
- Ajoute un événement (titre, date, heure)
- Les événements sont **chiffrés et synchronisés**

---

## 🎨 6. Personnalisation

Nook propose **3 thèmes** :

- **🌿 Jardin Secret** : doux, naturel, aquarelle
- **🚀 Space Hub** : futuriste, néon, épuré
- **🏠 Maison Chaleureuse** : feutre, crayon, bois

Pour changer de thème :
1. Clique sur l’icône en bas à droite
2. Choisis ton préféré → il est sauvegardé automatiquement

---

## 🔒 7. Sécurité & Confidentialité

- ✅ **Chiffrement de bout en bout** (libsodium)
- ✅ **Validation admin obligatoire**
- ✅ **Aucun compte, aucun mot de passe**
- ✅ **Aucun tracking, aucun analytics**
- ✅ **Code open-source (MIT License)**
- ✅ **Hébergé chez toi (CasaOS)**

---

## 📲 8. Installer Nook comme une app (PWA)

Sur **Android/Chrome** :
1. Ouvre Nook dans le navigateur
2. Clique sur **⋮ → “Installer l’application”**
3. Une icône **Nook** apparaît sur ton écran d’accueil

Sur **iOS/Safari** :
1. Ouvre Nook
2. Clique sur **Partager → “Ajouter à l’écran d’accueil”**

> 📱 Tu as maintenant une **app native, sécurisée, hors ligne**

---

## ❓ Besoin d’aide ?

- Consulte le **code source** : [https://github.com/MX10-AC2N/Nook](https://github.com/MX10-AC2N/Nook)
- Signale un bug via **Issues GitHub**
- Nook est **open-source** → tu peux contribuer !

---

🎉 **Bienvenue dans ton espace familial numérique, libre et sécurisé.**  
🌿 **Protège ta tribu.**