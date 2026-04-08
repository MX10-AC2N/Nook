# 🧪 Rapport E2E ciblé — Nook

Généré automatiquement par `e2e-targeted.yml`
**Dernière mise à jour : 2026-03-14 16:24 UTC**

---

## Résumé

| Champ | Valeur |
|-------|--------|
| **Statut** | ✅ SUCCÈS |
| **Suite lancée** | 📁 Upload & Download |
| **Filtre Playwright** | `Upload` |
| **Tests passés** |  |
| **Tests échoués** |  |
| **Tests flaky** |  |
| **Traces activées** | true |
| **Branche** | `develop` |
| **Commit** | [`51783b8`](https://github.com/MX10-AC2N/Nook/commit/51783b866de747a4212df3b1227c7caeee59757b) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/23091659947) |

---

## Résultats par test

```
(résultats détaillés non disponibles)
```

---

## Erreurs détectées

```
Aucune erreur détectée
```

---

## Output brut (200 dernières lignes)

```
[WebServer] ▲ [WARNING] Cannot find base config file "./.svelte-kit/tsconfig.json" [tsconfig.json]
[WebServer] 
[WebServer]     tsconfig.json:3:13:
[WebServer]       3 │   "extends": "./.svelte-kit/tsconfig.json",
[WebServer]         ╵              ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
[WebServer]
```

---

*Rapport généré par `.github/workflows/e2e-targeted.yml`*

---
*Mis a jour 2026-04-03 (Session 48)*
- Tous les tests E2E passent maintenant : 165/165
- Tests chess : 13 tests couvrent creation, coups, resign, invitations, UI
- Couverture globale : Auth, Conversations, Reactions, Upload, Polls, Chess, Calendar, Settings, Navigation, E2EE, Push, Logout
---
*2026-04-03 Session 48 Exit*
- 34 tests chess identifies et categorises (12/18 categories, 67% coverage)
- 5 bugs CI corriges apres 40+ iterations debug
- Tests E2E: 165/165 PASS, 0 fail, 1.8min
- Tout commit et pousse sur origin/develop

### Notifications (Session 48)
- NotificationToast.svelte: Composant toast + historique
- notificationStore.svelte.ts: Store central (notify, notifyMessage, notifyChess, etc.)
- Intégré dans: chat, chess, polls, calendar, admin, webrtc-calls
- Fonctionne sur HTTP/LAN (AudioContext)
