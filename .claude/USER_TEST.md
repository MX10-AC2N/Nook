# Retour d'expérience utilisateur

# test manuel du 26 Février 2026

Je vais listé ici mon retour d'expérience faite après déploiement sur mon homeserver fonctionnant sur Zimaboard 832

  - Déploiement sans problème.
  - 1er connexion en 'admin' => OK
  - Remplacement du mot de passe admin obligatoire => OK
  - Connexion au Chat => OK
  - Message de Test sur Groupe Global => Échec, aucun message envoyé, rien de visible.
  - Test d'upload d'un fichier => Échec, info visible dans les logs du docker Nook => [Upload Chat] Erreur DB: error returned from database: (code: 787) FOREIGN KEY constraint failed
  - Test d'administration :
      - génération du invitation =>  Non fonctionnel lorsque l'on clic pour généré une invitation le retour est 'Dernier lien généré : http://192.168.1.192:6300/invite?token=undefined' donc inutilisable et même sans lien pour le copié-collé. Mais l'invitation est quand même créé est apparaît dans le tableau des invitations en cours marqué :
        Créée le	Expire le	Statut	Lien	Action
26/02/2026 09:43	28/02/2026 09:43	Valide	af18c61a-eae… Copier	Supprimer
26/02/2026 08:19	28/02/2026 08:19	Valide	14413c8a-f2f…
Mais le bouton pour 'copier' le lien ne fonctionne pas.
      - Test d'approbation du demande 'en attente' => OK
      - Test de modification du Prénom 'Administrateur Initial' dans paramètre profil => Échec avec retour => Route API introuvable
      - Test de modification du thème => Échec, rien ne change..
  - Test de création d'un nouvel utilisateur => OK
  - Test de Chess :
      - Création d'une partie => OK mais il manque la possibilité d'invités les autres joueurs
      - Connexion entre 2 joueurs => OK, j'ai testé une partie, cela fonctionne le seul problème était le rafraîchissement de plateau de jeux a chaque coup. Si je joue les blancs, il fallait que les noirs actualise la page web pour voir le déplacement malgré que la page indique déjà bien 'a vous de joué'

  - Autre retour d'info plus général.
Le menu n'est pas complet, il manque plein d'accès on y trouve seulement :
```
Menu Nook
💬 Chat
📅 Calendrier
👑 Administration
⚙️ Paramètres
❓ Aide
```
Le Chat ne permet pas de voir les utilisateurs connectés, => impossible de lancé une conversation 🤦
la page Aide n'est pas a jour du tout..!

Certains mises en pages sur mobile dépasse de l'écran.

