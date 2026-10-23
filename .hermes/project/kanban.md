# 🗂️ Kanban Nook — Suivi des tâches et déploiements

Dernière maintenance : 2026-10-22
Statut : 2 tâches actives, 0 en attente.

---

## 📦 En cours

| ID | Nom | Assigné | Status | Notes |
|----|-----|---------|--------|-------|
| t_github-manager-rebuild | GITHUB-MANAGER-REBUILD | github-manager | 🔄 TODO | Le fix emoji picker (commit 0afb68c8) est pushé sur origin/develop. Déclencher le rebuild CI via workflow_dispatch: Frontend.yml + Backend.yml + test-nook.yml + Docker.yml + Release.yml. Le picker vertical flip est en code mais le homeserver (192.168.1.192:6300) n'a pas encore le nouveau build. |
| t_b086f747 | TEST-CHAT-FIX-DEPLOY | Mo Ju | ✅ DONE | Checklist validation déploiement chat (picker/scroll/saisie) — commit 74a39d7f |

---

## ✅ Terminé (récent)

| ID | Nom | Commit | Date | Résultat |
|----|-----|--------|------|----------|
| t_c6165e93 | FIX-CHAT-UI | 74a39d7f | 2026-09-06 | Picker en dessous, scroll KO, input visible |
| t_emoji-picker-flip | FIX-EMOJI-PICKER-FLIP | 0afb68c8 | 2026-09-11 | Picker flip fixé — vertical position conditionnelle |
| t_88036983 | CI-BUILD-v26 | — | 2026-09-08 | Pipeline déploiement terminé |
| t_0b0decb6 | ADR-017 (reply-to) récupéré/commité | 11f57006 | 2026-10-22 | Texte perdu retrouvé à l'identique dans le dump de session coder du 2026-08-28 (430 lignes) ; commité docs/adr/017-reply-to-feature.md, réf. ADR-018:8 corrigée, README §4.6 (provenance) + HISTORY.md |

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

- [ ] Vérifier déploiement sur 192.168.1.192:6300
- [ ] Signaler résultat validation