# 🚀 Superviseur Nook - Point d'Entrée

## Pour lancer le superviseur, utilise l'une de ces commandes:

### 1. Status Général
/supervisor status


### 2. Classifier une Tâche
/supervisor classify "ta tâche ici"


### 3. Rapport d'Optimisation
/supervisor optimize


### 4. Aide Complète
/supervisor help


---

## Comment ça marche?

1. **Analyse automatique** : Quand tu commences une conversation, le superviseur analyse ta demande
2. **Classification** : Il détermine la complexité (SIMPLE/MODERATE/COMPLEX/CRITICAL)
3. **Recommandation** : Il suggère Claude Free ou Plus selon le cas
4. **Journalisation** : Toutes les décisions sont documentées pour optimisation future

---

## Exemple d'Utilisation

**Toi:** `/supervisor classify "Refactoriser le module d'authentification"`

**Superviseur:**
📋 Classification

Tâche: "Refactoriser le module d'authentification"

Niveau: COMPLEX Modèle recommandé: Claude Plus

Justification:

Mots-clés: "refactoriser", "authentification" (sécurité)
Impact: Élevé (module critique)
Risque: Modification de code sensible
Recommandations: ⚠️ Cette tâche bénéficie de Claude Plus pour:

Meilleure compréhension du contexte
Moins d'itérations nécessaires
Qualité de code supérieure
Continuer avec Claude Free? (oui/non)


---

## Intégration avec tes Rôles

Le superviseur lit automatiquement:
- `.claude/roles/*.md` → pour comprendre les spécialités
- `.claude/CLAUDE.md` → pour le contexte projet
- `.claude/BUGS.md` → pour prioriser les tâches critiques

---

# Notes Importantes

- ✅ Aucune API requise
- ✅ Fonctionne avec Claude Free
- ✅ Compatible Claude Android
- ✅ Zéro configuration supplémentaire
- ✅ Persistance via fichiers `.claude/`

---

**Prêt à optimiser?** Tape `/supervisor status` pour commencer !



