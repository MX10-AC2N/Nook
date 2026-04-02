# 🤖 Hermes Agent — Nook Workspace

> Agent Hermes pour le développement de Nook
> Branche de travail : `fix/notifications-and-chess-audit` → PR #23
> Branch principale dev : `develop`
> Dernière session : 2026-04-02 (S46)

---

## 📋 Contexte actuel

- **Version :** 0.4.0-beta.2 (branche develop)
- **PR #23** : tests E2E chess+WebRTC étendus + audit sécurité complet
- **Build CI :** en attente
- **Tests E2E :** 156 tests total (115 existants + 41 nouveaux)
- **3 bugs sécurité ouverts** : SEC-07 (webrtc auth), SEC-09 (CSP), SEC-10 (headers HTTP)
- **GITHUB_TOKEN** dans `~/.hermes/.env` — persisté en mémoire

---

## 🎯 Next Steps Priorities

1. ~~[x] PR #22 security patches~~ — MERGED
2. ~~[x] Tests E2E chess + WebRTC~~ — commit sur PR #23
3. ~~[x] Audit sécurité complet~~ — documenté dans TEST-AND-SECURITY-AUDIT-2026.md
4. ~~[x] Catalogue workflows~~ — WORKFLOW-CATALOG.md créé
5. ~~[x] Màj .claude/ (CLAUDE.md, BUGS.md, SESSIONS.md, workflows.md)~~
6. [ ] Fix SEC-07 : auth sur routes WebRTC offer/answer (priorité haute)
7. [ ] Fix SEC-09 : ajouter CSP dans app.html
8. [ ] Fix SEC-10 : headers de sécurité HTTP
9. [ ] Cleanup workflows (3 à supprimer, 2 à fusionner)
10. [ ] Lancer CI sur PR #23

---

## 📝 Rules learned this session

- TOUJOURS verifier `Cargo.toml` versions avant de modifier du code Rust
- rand 0.9 : `rng()` pas `thread_rng()`, `distr` pas `distributions`, importer `Rng` trait pour `sample_iter`
- Le repo utilise `.claude/` system pour la mémoire agent — lire CLAUDE.md en premier
- BUGS.md = bugs actifs, SESSIONS.md = historique
- Ne jamais toucher les sélecteurs CSS sans vérifier les fichiers `*.spec.ts` (E2E)
- 20 workflows dans `.github/workflows/` — 3 obsolètes, 2 redondants (voir WORKFLOW-CATALOG.md)
- WebRTC : `handle_offer`/`handle_answer` sont hors `protected_routes` → pas d'auth (SEC-07)
- Tests E2E en CI headless : pas de getUserMedia → tests WebRTC limités à structure/API
- `.claude/TEST-AND-SECURITY-AUDIT-2026.md` = rapport complet tests + sécurité de référence
- `.claude/WORKFLOW-CATALOG.md` = catalogue complet des 20 workflows + plan de cleanup
