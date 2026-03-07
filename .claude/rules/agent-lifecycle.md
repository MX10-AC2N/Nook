# 🧬 Cycle de vie des agents — Nook

> Comment les agents naissent, apprennent, évoluent et disparaissent.
> Lire avant de créer un nouvel agent ou de modifier la structure existante.

---

## 🌱 Créer un nouvel agent

### Critères de création

Un nouvel agent est justifié si **au moins 3** de ces conditions sont remplies :
- [ ] Le domaine représente ≥ 5 fichiers cohérents dans le repo
- [ ] Des règles spécifiques s'appliquent (pièges, patterns, contraintes propres)
- [ ] Ce domaine est apparu dans ≥ 2 sessions consécutives
- [ ] Les agents existants débordent sur ce périmètre de façon répétée
- [ ] Des bugs récurrents sont propres à ce domaine

### Template de création

```markdown
# [Emoji] Agent [NOM] — Nook

> Description courte — une phrase.
> Activer pour : [déclencheurs explicites].

---

## 🎯 Périmètre
[Fichiers exacts du repo, pas de domaines flous]

---

## ⚠️ Points critiques
[Pièges connus, anti-patterns, règles spécifiques]

---

## 🤝 Interface inter-agents

### Ce que [AGENT] produit pour les autres
[Sorties concrètes : types, endpoints, sélecteurs, variables...]

### Ce que [AGENT] attend des précédents
[Entrées nécessaires]

---

## 📚 Apprentissages
> *Section vide à la création — se remplit avec l'expérience.*
```

### Enregistrement

Après création :
1. Ajouter à la table **AGENTS DISPONIBLES** dans `CLAUDE.md`
2. Ajouter à la grille de **DISPATCH** dans `CLAUDE.md`
3. Ajouter aux exemples de dispatch si pertinent
4. Créer un ADR dans `memory-decisions.md` (D-series)

---

## 📈 Faire évoluer un agent

### Section `## 📚 Apprentissages` — Règles d'écriture

```markdown
### [APP-{AGENT}-{N}] Titre court — Session X [→ Statut]

Description du problème ou du pattern découvert.
Contexte : pourquoi c'est arrivé, dans quelle configuration.
Fix ou règle : ce qu'il faut faire à la place.
Status : Découverte / Confirmé / Promu / Archivé
```

### Cycle de vie d'un apprentissage

```
Découverte (session N)
    ↓
Noté dans SESSIONS.md + section Apprentissages de l'agent
    ↓ (si revu dans une 2ème session)
Confirmé → marqué "Confirmé" dans Apprentissages
    ↓ (si applicable à toutes les futures interventions)
Promu → intégré dans la section principale du rôle
        marqué "Promu" dans Apprentissages (gardé pour traçabilité)
    ↓ (alternatif : si décision d'architecture)
Archivé → déplacé dans memory-decisions.md (D-series)
```

### Quand promouvoir

Un apprentissage est promu vers la section principale quand :
- Il s'applique à **chaque** intervention dans ce domaine (pas seulement les cas rares)
- Il a été **validé** au moins une fois en session réelle
- Il évite un bug ou une perte de temps significative

---

## 🔀 Fusionner deux agents

Si deux agents couvrent des domaines qui se chevauchent de plus en plus :
1. Identifier le périmètre commun
2. Décider lequel absorbe l'autre (ou créer un troisième)
3. Migrer les apprentissages
4. Mettre à jour `CLAUDE.md` et les interfaces inter-agents
5. Garder l'ancien fichier avec un header `# ⚠️ OBSOLÈTE — Voir [nouveau agent]`

---

## ❌ Retirer un agent

Un agent est retiré si :
- Son domaine est absorbé par un autre agent
- La feature couverte est supprimée du projet
- Il n'a jamais été activé après 10 sessions

Procédure :
1. Vider le fichier, ajouter `# OBSOLÈTE — [Raison] — Session N`
2. Le laisser 5 sessions pour référence historique
3. Supprimer + commit avec message explicite

---

## 📊 État actuel des agents (Session 24)

| Agent | Sessions d'activation | Maturité | Apprentissages |
|-------|-----------------------|----------|----------------|
| 🦀 RUST | ~15 sessions | 🟢 Mature | 5 |
| 🎨 SVELTE | ~12 sessions | 🟢 Mature | 5 |
| 🚀 DEVOPS | ~10 sessions | 🟢 Mature | 6 |
| 🧪 E2E | ~8 sessions | 🟢 Mature | 6 |
| 🔐 CRYPTO | ~5 sessions | 🟡 Stabilisation | 5 |
| ♟️ CHESS | ~3 sessions | 🟡 Stabilisation | 3 |
| 📊 DATA | ~2 sessions | 🔴 Nouveau | 2 |
| 📐 ARCHITECT | ~2 sessions | 🔴 Nouveau | 4 |
| 🤖 DELEGATE  | ~0 sessions | 🔴 Nouveau | 0 (à alimenter) |

---

## 🔮 Agents candidats à créer (si le projet évolue)

| Domaine | Déclencheur | Fichiers concernés |
|---------|-------------|-------------------|
| 🎙️ **MEDIA** | Appels audio/video, enregistrements | `webrtc-calls.svelte.ts`, `mediaStore.svelte.js`, `call/+page.svelte` |
| 🔔 **NOTIF** | Push notifications, alertes temps réel | Nouveau backend + SW frontend |
| 📱 **PWA** | Service worker, offline, install prompt | `manifest.json`, SW à créer |
| 🌍 **I18N** | Internationalisation si multi-langue | Tous les composants |
| 📦 **MIGRATION** | Migrations DB majeures, breaking changes | Toutes les migrations SQL |
