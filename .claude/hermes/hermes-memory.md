# 🤖 Hermes Agent — Nook Workspace

> Agent Hermes pour le développement de Nook
> Branche de travail : `fix/security-patches-2025` → PR #22
> Branch principale dev : `develop`
> Dernière session : 2026-04-01

---

## 📋 Contexte actuel

- **Version :** 0.4.0-beta.2 (branche develop)
- **PR #22 :** 6 fix de sécurité (C1, C2, C5, M1, M2, M3, L3)
- **Build CI :** ✅ Passe après correction API rand 0.9
- **Zéro bug actif** dans BUGS.md

---

## 🎯 Next Steps Priorities

1. ~~[x] PR #22 security patches~~ — BUILD PASSE
2. ~~[ ]~~ Fix C4: WebSocket broadcast global filtering by conversation (architectural)
3. ~~[ ]~~ Fix C3: Session token hashing in DB (needs migration)
4. ~~[ ]~~ Fix C6: Encrypt pending E2E keys in localStorage
5. ~~[ ]~~ ~~Update BUGS.md/SESSIONS.md with session 45~~
6. ~~[ ]~~ ~~Verify all .claude/ docs consistency~~

---

## 📝 Rules learned this session

- TOUJOURS verifier `Cargo.toml` versions avant de modifier du code Rust
- rand 0.9 : `rng()` pas `thread_rng()`, `distr` pas `distributions`, importer `Rng` trait pour `sample_iter`
- Le repo utilise `.claude/` system pour la mémoire agent — lire CLAUDE.md en premier
- BUGS.md = bugs actifs (vide maintenant), SESSIONS.md = historique
- Ne jamais toucher les sélecteurs CSS sans vérifier `user.spec.ts` (E2E)
