# 🔧 Commandes Slash Nook — Conversation CLI

> Liste des commandes slash disponibles dans la conversation
> Usage : Taper `/commande` dans le chat → Hermes exécute l'action associée
> Mis à jour : 2026-04-27

---

## 📋 Commandes disponibles

### `/nook-fin` — Session Exit Propre
**Action :** Termine la session, met à jour `.claude/hermes/`, push tout, et quitte proprement.
**Équivalent :** Skill `nook-fin`
**État :** ✅ Disponible

### `/nook-fin-now` — Session Exit Immédiat
**Action :** Comme `/nook-fin` mais sans vérifications étendues.
**État :** ✅ Disponible

### `/nook-status` — État du projet
**Action :** Affiche l'état actuel (CI, commits, bugs, TODO).
**État :** ✅ Disponible

### `/nook-ci` — Vérifier la CI
**Action :** Check le status des workflows GitHub Actions (Backend, Frontend, Docker).
**État :** ✅ Disponible

### `/nook-bugs` — Lister les bugs
**Action :** Affiche `.claude/hermes/known-issues.md` et `.claude/project/BUGS.md`.
**État :** ✅ Disponible

### `/nook-fix-e2ee` — Fix bug E2EE refresh
**Action :** S'attaque au bug E2EE refresh (cryptoStore.ready=false).
**État :** ✅ Disponible (à lancer après /nook-fin)

### `/nook-p2p-test` — Tester P2P file transfer
**Action :** Vérifie et teste le transfert P2P >50 Mo.
**État :** ✅ Disponible

### `/nook-help` — Aide
**Action :** Affiche cette liste de commandes.
**État :** ✅ Disponible

---

## 🤖 Comment Hermes interprète les commandes

Quand tu tapes `/commande` dans le chat, je :

1. ✅ Reconnais le pattern `/xxx`
2. ✅ Lis le fichier correspondant (ex: `.claude/skills/nook-fin/SKILL.md`)
3. ✅ Exécute **TOUTES** les étapes décrites
4. ✅ Met à jour `.claude/hermes/` selon l'action
5. ✅ Te confirme l'action effectuée

---

## 📝 Notes importantes

- **Toutes les commandes slash sont exécutées PAR MOI** (Hermes Agent) dans cette conversation
- **Pas besoin de cliquer** — juste type `/commande` et j'exécute
- **`/nook-fin` DOIT être lancé** avant de quitter (pour ne rien oublier)
- **Consulter `.claude/reference/commands.md`** pour les commandes techniques (git, docker, cargo...)

---

## 🔗 Liens rapides

- **Commands référence :** `.claude/reference/commands.md`
- **Slash commands :** `.claude/commands.md` (ce fichier)
- **Skills détaillés :** `.claude/skills/*/SKILL.md`

---

*Pour ajouter une nouvelle commande slash : modifier ce fichier + créer le skill correspondant dans `.claude/skills/`*
