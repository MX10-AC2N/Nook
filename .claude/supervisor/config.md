# 🧠 Superviseur Nook - Configuration

## Objectif
Optimiser l'utilisation de Claude Free en allouant intelligemment les ressources selon la complexité des tâches.

---

## Modèles Disponibles (via Claude)

| Modèle | Type | Usage Recommandé |
|--------|------|------------------|
| Claude Free | Gratuit | Tâches simples, itérations rapides |
| Claude Plus | Payant | Tâches complexes, décisions critiques |

---

## Classification des Tâches

### Niveau 1 - SIMPLE (Claude Free OK)
- Questions factuelles
- Recherche d'information
- Formatage de texte
- Vérifications basiques
- Commandes shell simples

**Mots-clés :** "bonjour", "version", "liste", "aide", "comment faire", "qu'est-ce que"

### Niveau 2 - MODERATE (Claude Free acceptable)
- Analyse de code
- Génération de documentation
- Résumés
- Debugging simple
- Refactoring mineur

**Mots-clés :** "analyser", "expliquer", "améliorer", "documenter", "tester"

### Niveau 3 - COMPLEX (Recommander Claude Plus)
- Architecture système
- Design patterns complexes
- Optimisation performance
- Sécurité
- Intégrations multiples

**Mots-clés :** "architecture", "design", "optimisation", "sécurité", "production"

### Niveau 4 - CRITICAL (Claude Plus requis)
- Décisions métier importantes
- Données sensibles
- Paiements/authentification
- Audit de sécurité

**Mots-clés :** "sécurité", "audit", "production", "données sensibles", "paiement"

---

## Règles d'Allocation

IF complexity == SIMPLE THEN → Utiliser Claude Free → Limiter à 3 itérations max → Demander confirmation avant extension

IF complexity == MODERATE THEN → Utiliser Claude Free → Limiter à 5 itérations max → Proposer upgrade si blocage

IF complexity == COMPLEX THEN → Avertir: "Cette tâche pourrait bénéficier de Claude Plus" → Continuer avec Free si utilisateur confirme → Documenter les limitations

IF complexity == CRITICAL THEN → Recommander fortement Claude Plus → Ne pas procéder sans confirmation explicite → Documenter les risques

---

## Commandes Disponibles

| Commande | Description |
|----------|-------------|
| `/supervisor status` | État actuel et statistiques |
| `/supervisor classify [tâche]` | Classer une tâche |
| `/supervisor optimize` | Rapport d'optimisation |
| `/supervisor budget` | Suivi du budget estimé |
| `/supervisor help` | Aide complète |

---

## Journal des Décisions

| Date | Tâche | Complexité | Modèle | Justification |
|------|-------|------------|--------|---------------|
| - | - | - | - | - |

---

## Limitations Connues

- Claude Free a des limites de contexte
- Pas d'accès API pour automatisation
- Sessions limitées dans l'app Android
- Pas de persistance entre sessions
