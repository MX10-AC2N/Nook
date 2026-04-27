# 🛠️ Session Start — Outils Hermes + Lecture .claude OBLIGATOIRE

> **CE HOOK EST EXÉCUTÉ AUTOMATIQUEMENT AU DÉMARRAGE**
> **NE JAMAIS IGNORER LES ÉTAPES CI-DESSOUS**

## 🔴 ÉTAPE 0 — LECTURE .claude/hermes/ (OBLIGATOIRE)

**AVANT TOUTE CHOSE, lire dans l'ordre :**

```bash
# 1. Vérifier si on est dans le repo Nook
if [ -f .claude/hermes/active-session.md ]; then
  echo "=== LECTURE ESPACE HERMES ==="
  cat .claude/hermes/active-session.md
fi
```

**Action manuelle équivalente :**
1. ✅ Lire `.claude/hermes/active-session.md` → Qu'est-ce que je faisais ?
2. ✅ Lire `.claude/hermes/known-issues.md` → Quels pièges éviter ?
3. ✅ Lire `.claude/hermes/hermes-memory.md` → Infos critiques (tokens, comptes)
4. ✅ Vérifier `.claude/project/project-state.md` → Où en est Nook ?

**⚠️ RÈGLE ABSOLUE :**
> SI `.claude/hermes/` EXISTE, le lire SYSTÉMATIQUEMENT avant de répondre à l'utilisateur.
> C'est MON espace de travail. Pas de perte de mémoire.

---

## 🔧 ÉTAPE 1 — Installation outils (si nécessaire)

```bash
bash .claude/hooks/scripts/hermes-tools-setup.sh
```

Cela installe :
- `curl` — requêtes HTTP
- `playwright` — screenshots navigateur
- `chromium` — headless browser
- `lightpanda` — fast headless browser (111MB)

## Raccourci
Si le script échoue, installer manuellement :
```bash
apt-get install -y curl
pip install --break-system-packages playwright
playwright install chromium
curl -L -o /tmp/lightpanda https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux && chmod +x /tmp/lightpanda
```

## Utilisation
- `curl http://192.168.1.192:6300/api/health` — test API
- `python3 -c "from playwright.sync_api import sync_playwright"` — vérifier playwright
- `/tmp/lightpanda fetch --dump html http://...` — fetch rapide

---

## 📋 CHECKLIST DÉMARRAGE (à cocher mentalement)

- [ ] J'ai lu `.claude/hermes/active-session.md`
- [ ] J'ai lu `.claude/hermes/known-issues.md`
- [ ] Je connais l'état actuel (project-state.md)
- [ ] Je sais où en est la CI (Backend.yml, Frontend.yml)
- [ ] Je n'oublierai pas : rand 0.9 = `rng()`, Axum 0.8 = `{param}`
- [ ] Je ne toucherai pas aux versions des dépendances dans un commit de fix

---
*Ce hook garantit que je n'oublierai plus le contexte au redémarrage.*
