# Changelog

Tous les changements notables du projet Nook sont documentés ici.

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).

---

## [0.5.0] — 2026-04-12

### Ajouté
- **Avatar** — Composant réutilisable avec système de fallback (initiales + couleur hashée)
- **Calendrier — Glisser-déposer** — Déplace les événements entre les jours en mode mois
- **Échecs — Notation PGN** — Export PGN complet avec bouton copier dans l'historique des coups
- **Échecs — Animations** — Animations de déplacement des pièces et coups légaux
- **Sondages — Animations** — Animations de vote, barres de progression, badge gagnant
- **Chart.js — Lazy loading** — Chargement à la demande pour optimiser le bundle
- **Tests E2E** — 163 tests couvrant toutes les fonctionnalités

### Modifié
- **Bundle** — Optimisation manualChunks (chart.js, webrtc, chess)
- **CSS** — Nettoyé les règles dupliquées, layout flex optimisé
- **CI** — Workflow corrigé (git clean avant commit, .gitignore récursif)
- **Config** — Routing intelligent avec modèles gratuits OpenRouter

### Corrigé
- Réactions — Chargement au changement de conversation
- Chat — sendMessage avec mise à jour optimiste
- Tests — Flaky tests remplacés par vérifications API stables
- Vite — CVE corrigé, build sécurisé

---

## [0.5.0-beta.1] — 2026-04-11

### Ajouté
- **Réactions emoji** — Système complet avec picker, UPSERT, notifications temps réel
- **Calendrier — Vues** — Mois, semaine, jour avec navigation fluide
- **Échecs — Coups spéciaux** — Roque, promotion, en passant, règle des 50 coups
- **Échecs — Timer** — Minuteur avec formats 5/10/15/30 minutes
- **Échecs — IA** — 5 niveaux de difficulté (easy → godlike)
- **Échecs — Invitations** — Système d'invitation entre joueurs
- **Notifications** — 6 types via WebSocket (message, réaction, poll, calendar, chess, admin)
- **E2EE** — Chiffrement de bout en bout X25519 + XChaCha20
- **Push** — Notifications push activables (VAPID)
- **Upload** — Fichiers jusqu'à 50 Mo chiffrés sur disque
- **Settings** — 3 onglets (Profil, Sécurité, Apparence)

---

## [0.4.x] — Versions précédentes

Voir [commits GitHub](https://github.com/MX10-AC2N/Nook/commits/main) pour l'historique complet.
