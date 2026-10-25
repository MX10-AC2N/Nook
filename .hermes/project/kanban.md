# 🗂️ Kanban Nook — Suivi des tâches et déploiements

Dernière maintenance : 2026-10-25
Statut : 4 tâches non archivées (1 en cours, 2 prêtes, 1 bloquée).

---

## 📦 En cours

| ID | Nom | Assigné | Status | Notes |
|----|-----|---------|--------|-------|
| t_cc0ea93d | NETTOYAGE working tree : retirer la copie obsolète docs/workspace-protocol.md (v5.3) | orchestrator | 🔄 RUNNING | Copie non suivie v5.3 retirée du working tree ; la v8 vit dans PR#48 (branche w/t_f8e650c1-workspace-protocol-doc, head e1bf4df5, 27857 o). Aucun commit sur develop ne peut plus embarquer la version obsolète. |
| t_de452c48 | Recréer le conteneur Nook de 192.168.1.192 (bundle périmé) | deployer | ⛔ BLOCKED | Bundle périmé `0.CrOWh7b4.css` — reconstruire le conteneur. |

---

## 📥 Prêtes

| ID | Nom | Assigné | Notes |
|----|-----|---------|-------|
| t_586756e1 | Ajouter backend/nook.db + nook-data/ au .gitignore | coder | Base SQLite embarquée par git — à ignorer. |
| t_e90110f9 | Aligner la skill nook-worker-supervision (references/workspace-protocol.md) | orchestrator | Copie skill à resynchroniser sur la v8 du protocole (PR#48). |

---

## ✅ Terminé (récent)

| ID | Nom | Commit | Date | Résultat |
|----|-----|--------|------|----------|
| t_35ea07c7 | Corriger docs/workspace-protocol.md (PR#48) | e1bf4df5 | 2026-10-25 | Revue @architect round 9 : APPROUVÉ (4/4 critères, vérificateur indépendant 70 contrôles). v8 = 27857 o, md5 4bfbc4e1…. PR#48 ouverte, non mergée, +383/-0, 1 fichier. |
| t_0b0decb6 | ADR-017 (reply-to) récupéré/commité | 11f57006 | 2026-10-22 | Texte perdu retrouvé à l'identique dans le dump de session coder du 2026-08-28 (430 lignes) ; commité docs/adr/017-reply-to-feature.md, réf. ADR-018:8 corrigée, README §4.6 (provenance) + HISTORY.md |
| t_c6165e93 | FIX-CHAT-UI | 74a39d7f | 2026-09-06 | Picker en dessous, scroll KO, input visible |

---

## 🗑️ Nettoyage

Toutes les anciennes cartes (150+) ont été purgées. Seules les tâches actives + 3 dernières terminées sont conservées.
L'historique complet est archivé dans `.hermes/archive/`.

---

## 🔄 Règle de maintenance

- Conserver uniquement : tâches actives + 3 dernières terminées
- Cartes > 7 jours terminées → archivées/purgées
- Le kanban = outil de suivi en direct, pas un historique

---

## 📝 Prochaines actions

- [ ] Merger PR#48 (docs/workspace-protocol.md v8) sur develop
- [ ] Vérifier déploiement sur 192.168.1.192:6300
