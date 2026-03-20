# 🌿 Nook — Guide de l'utilisateur

Nook est ta **messagerie familiale privée**, hébergée sur ton propre serveur.
Tes conversations restent chez toi. Pas de compte externe. Pas de publicité.

---

## Se connecter

Ouvre l'application à l'adresse fournie par l'admin de ta famille.

- **Identifiant** : le nom que t'a donné l'admin (ex : `marie`)
- **Mot de passe** : reçu par l'admin, à changer à la première connexion

> **Nouveau membre ?** Tu reçois un lien d'invitation par message ou email.
> Clique dessus, crée ton compte — l'admin l'approuvera avant que tu puisses te connecter.

---

## Chat

Dès la connexion, tu arrives dans le **groupe Nook** — la conversation de toute la famille.

**Envoyer un message** : tape ton texte en bas et appuie sur Entrée (ou clique →).

**Partager un fichier ou une photo** : clique sur le trombone 📎. Taille max 50 Mo.
Les fichiers sont chiffrés automatiquement sur le serveur et supprimés après 48h.

**Réagir à un message** : passe la souris (ou appuie longuement sur mobile) sur un message
pour faire apparaître le bouton 😊 — choisis ton emoji.

**Enregistrer un vocal** : clique sur le micro 🎤, parle, relâche pour envoyer.

**Nouvelle conversation** : clique sur ✏️ en haut pour créer un groupe ou une discussion privée.

---

## Calendrier

La page `/calendar` affiche les événements de toute la famille.

Clique sur **+ Ajouter** pour créer un rendez-vous : titre, date, heure (optionnelle), description.
Seul le créateur ou l'admin peut supprimer un événement.

---

## Sondages

La page `/polls` permet de créer des votes rapides.

**Créer un sondage** : clique sur **+ Nouveau sondage**, écris ta question et tes options (2 à 10).

**Voter** : clique sur ta réponse. Tu peux changer d'avis tant que le sondage est ouvert.

**Fermer un sondage** : le créateur ou l'admin peut clore le vote — les résultats restent visibles.

---

## Échecs ♟️

La page `/chess` liste les parties en cours et terminées.

**Nouvelle partie** : choisis ta couleur et l'adversaire (un membre de la famille ou l'IA en easy/medium/hard).

**Jouer** : clique sur ta pièce, puis sur la case d'arrivée. Les coups illégaux sont refusés.

Les coups de l'adversaire apparaissent en temps réel — pas besoin de rafraîchir la page.

---

## Appels audio & vidéo 📞

La page `/call` permet de lancer un appel avec un membre.

> ⚠️ Les appels fonctionnent bien sur le réseau local (wifi maison).
> Sur internet, ils peuvent parfois échouer selon la configuration réseau.
> Le serveur ne voit jamais le flux — la connexion est directe entre appareils.

---

## Paramètres ⚙️

La page `/settings` contient trois onglets :

**👤 Profil** — Modifie ton prénom affiché. L'identifiant de connexion ne peut pas changer.

**🔒 Sécurité** — Modifie ton mot de passe. Active les **notifications push** 🔔
pour recevoir une alerte sur cet appareil même quand Nook est fermé
(nécessite d'accepter la demande de permission du navigateur).

**🎨 Apparence** — Choisis ton thème :
- 🌿 **Jardin Secret** — doux et naturel
- 🚀 **Space Hub** — sombre et futuriste
- 🏠 **Maison Chaleureuse** — chaud et ambre

Le mode sombre est disponible en complément de chaque thème.

---

## Chiffrement & vie privée

Tout ce qui passe par Nook reste sur **ton serveur**.

- Les **fichiers** partagés sont chiffrés (XChaCha20-Poly1305) sur le disque.
- Les **messages** sont chiffrés de bout en bout (X25519) dès que les deux participants ont leurs clés.
  Les clés sont générées automatiquement à la connexion et stockées chiffrées sur ton appareil — jamais sur le serveur.
- Les **mots de passe** sont hachés avec Argon2id — même l'admin ne peut pas les lire.

---

## Problèmes fréquents

**Je ne reçois plus de notifications push**
→ Va dans Paramètres → Sécurité → désactive puis réactive les notifications.

**Mes messages n'apparaissent pas**
→ Recharge la page. Si le problème persiste, déconnecte-toi et reconnecte-toi.

**Je ne peux pas me connecter**
→ Vérifie que l'admin a approuvé ton compte. Si tu as oublié ton mot de passe, contacte l'admin.

**J'ai perdu mes clés de chiffrement**
→ Change ton mot de passe dans Paramètres → Sécurité. De nouvelles clés seront générées.
