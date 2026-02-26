# Retour d'expérience utilisateur — Template de test

> **Instructions** : Après chaque test, remplis la colonne Résultat avec :
> ✅ OK | ❌ Échec | ⚠️ Partiel | 🔄 Non testé
> Et note tes observations dans la colonne Détails (message d'erreur, comportement observé, etc.)

---

## 🖥️ Environnement de test

| Champ | Valeur |
|---|---|
| Date du test | |
| Version du commit / tag | |
| Serveur | Zimaboard 832 |
| Navigateur(s) testé(s) | |
| Réseau (LAN / WAN / les deux) | |
| OS mobile (si testé) | |

---

## 🚀 Déploiement

| # | Test | Résultat | Détails |
|---|---|---|---|
| D1 | `docker compose up -d` sans erreur | | |
| D2 | Backend répond sur `:6300/health` | | |
| D3 | Logs Docker sans erreur critique | | |

---

## 🔐 Authentification

| # | Test | Résultat | Détails |
|---|---|---|---|
| A1 | Connexion admin (admin / nouveau mdp) | | |
| A2 | Redirection forcée vers /change-password à la 1ère connexion | | |
| A3 | Changement de mot de passe admin | | |
| A4 | Déconnexion (bouton 🔌) | | |
| A5 | Redirection vers /login après déconnexion | | |
| A6 | Connexion avec mauvais mot de passe → message d'erreur clair | | |

---

## 👑 Administration

| # | Test | Résultat | Détails |
|---|---|---|---|
| ADM1 | Page /admin accessible | | |
| ADM2 | Génération d'un lien d'invitation | | |
| ADM3 | Lien généré correct (format `http(s)://host/invite?token=UUID`) | | |
| ADM4 | Bouton "Copier" du lien dans le tableau fonctionne | | |
| ADM5 | Invitation apparaît dans le tableau avec statut "Valide" | | |
| ADM6 | Suppression d'une invitation | | |
| ADM7 | Liste des utilisateurs en attente d'approbation | | |
| ADM8 | Approbation d'un utilisateur en attente | | |
| ADM9 | Liste de tous les utilisateurs | | |

---

## 💬 Chat

| # | Test | Résultat | Détails |
|---|---|---|---|
| C1 | Page /chat accessible après connexion | | |
| C2 | "Groupe Global" visible dans la sidebar | | |
| C3 | Envoi d'un message texte dans Groupe Global | | |
| C4 | Message apparaît dans la conversation | | |
| C5 | Rechargement de page → messages précédents visibles | | |
| C6 | Envoi d'un GIF (bouton 🎬) | | |
| C7 | Upload d'un fichier image (bouton 📎) | | |
| C8 | Upload d'un fichier non-image | | |
| C9 | Message d'erreur si envoi échoue | | |

---

## ♟️ Échecs (Chess)

| # | Test | Résultat | Détails |
|---|---|---|---|
| CH1 | Page /chess accessible | | |
| CH2 | Création d'une nouvelle partie | | |
| CH3 | Connexion de 2 joueurs à la même partie | | |
| CH4 | Coup joué par joueur 1 visible chez joueur 2 sans refresh | | |
| CH5 | Indicateur "À vous de jouer" correct | | |
| CH6 | Fin de partie (échec et mat) détectée | | |

---

## 📅 Calendrier

| # | Test | Résultat | Détails |
|---|---|---|---|
| CAL1 | Page /calendar accessible | | |
| CAL2 | Création d'un événement (titre + date) | | |
| CAL3 | Événement apparaît dans la liste | | |
| CAL4 | Suppression d'un événement | | |
| CAL5 | Rechargement page → événements persistés | | |

---

## 📊 Sondages (Polls)

| # | Test | Résultat | Détails |
|---|---|---|---|
| P1 | Page /polls accessible depuis le menu | | |
| P2 | Création d'un sondage | | |
| P3 | Vote sur un sondage | | |
| P4 | Résultats visibles | | |

---

## ⚙️ Paramètres

| # | Test | Résultat | Détails |
|---|---|---|---|
| S1 | Page /settings accessible | | |
| S2 | Modification du prénom (Paramètres → Profil) | | |
| S3 | Prénom mis à jour dans le header après sauvegarde | | |
| S4 | Changement de thème (Jardin Secret / Space Hub / Maison Chaleureuse) | | |
| S5 | Thème appliqué immédiatement sans rechargement | | |
| S6 | Thème persisté après rechargement de la page | | |
| S7 | Mode sombre toggle | | |
| S8 | Changement de mot de passe depuis les paramètres | | |

---

## 🧭 Navigation & Menu

| # | Test | Résultat | Détails |
|---|---|---|---|
| N1 | Menu hamburger s'ouvre | | |
| N2 | Chat visible dans le menu | | |
| N3 | Échecs visible dans le menu | | |
| N4 | Calendrier visible dans le menu | | |
| N5 | Sondages visible dans le menu | | |
| N6 | Administration visible (admin seulement) | | |
| N7 | Paramètres visible dans le menu | | |
| N8 | Menu se ferme après navigation | | |

---

## 👤 Inscription par invitation

| # | Test | Résultat | Détails |
|---|---|---|---|
| I1 | Lien d'invitation redirige vers /invite | | |
| I2 | Formulaire d'inscription pré-rempli avec le token | | |
| I3 | Inscription réussie avec un lien valide | | |
| I4 | Connexion immédiate après inscription (sans attente d'approbation) | | |
| I5 | Lien invalide → message d'erreur clair | | |
| I6 | Lien expiré → message d'erreur clair | | |

---

## 📱 Mobile / Responsive

| # | Test | Résultat | Détails |
|---|---|---|---|
| M1 | Page /login lisible sur mobile | | |
| M2 | Menu hamburger utilisable sur mobile | | |
| M3 | Chat utilisable sur mobile (pas de débordement) | | |
| M4 | Sidebar conversations visible sur mobile | | |
| M5 | Calendrier lisible sur mobile | | |
| M6 | Administration lisible sur mobile | | |

---

## 🐛 Bugs libres

> Note ici tout comportement inattendu non couvert par les tests ci-dessus.

| # | Description | Page/Fonctionnalité | Reproductible | Gravité (🔴/🟡/🟢) |
|---|---|---|---|---|
| B1 | | | | |
| B2 | | | | |
| B3 | | | | |
| B4 | | | | |
| B5 | | | | |

---

## 💡 Suggestions UX

> Idées d'amélioration de l'expérience utilisateur.

| # | Suggestion | Priorité (haute/moyenne/basse) |
|---|---|---|
| U1 | | |
| U2 | | |
| U3 | | |

---

## 📊 Résumé

| Catégorie | Total | ✅ OK | ❌ Échec | ⚠️ Partiel | 🔄 Non testé |
|---|---|---|---|---|---|
| Déploiement | 3 | | | | |
| Auth | 6 | | | | |
| Admin | 9 | | | | |
| Chat | 9 | | | | |
| Chess | 6 | | | | |
| Calendrier | 5 | | | | |
| Sondages | 4 | | | | |
| Paramètres | 8 | | | | |
| Navigation | 8 | | | | |
| Invitation | 6 | | | | |
| Mobile | 6 | | | | |
| **TOTAL** | **70** | | | | |
