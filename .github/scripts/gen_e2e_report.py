#!/usr/bin/env python3
"""
Génère .hermes/E2E-TARGETED-REPORT.md
Appelé par e2e-targeted.yml — toutes les variables passées via env.
"""

import os

# ── Variables d'environnement injectées par le workflow ──────────────────────
run_date   = os.environ.get("RUN_DATE", "?")
run_id     = os.environ.get("RUN_ID", "?")
repo       = os.environ.get("GITHUB_REPOSITORY", "")
commit_sha = os.environ.get("COMMIT_SHA", "")
commit_short = commit_sha[:7] if commit_sha else "?"
branch     = os.environ.get("BRANCH", "?")
label      = os.environ.get("LABEL", "?")
grep       = os.environ.get("GREP", "?")
passed     = os.environ.get("PASSED", "?")
failed     = os.environ.get("FAILED", "?")
flaky      = os.environ.get("FLAKY", "0")
traces     = os.environ.get("TRACES_LABEL", "false")

run_url    = f"https://github.com/{repo}/actions/runs/{run_id}"
commit_url = f"https://github.com/{repo}/commit/{commit_sha}"

failed_int = int(failed) if failed and failed.isdigit() else 0
status_icon = "✅" if failed_int == 0 else "❌"
status_text = "SUCCÈS" if failed_int == 0 else "ÉCHEC"


def read_stripped(path: str, max_lines: int = 200) -> str:
    """Lit un fichier, retire les codes ANSI, limite aux N dernières lignes."""
    import re
    ansi = re.compile(r"\x1b\[[0-9;]*m")
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
        lines = [ansi.sub("", l) for l in lines[-max_lines:]]
        return "".join(lines).strip()
    except FileNotFoundError:
        return "(fichier non disponible)"


test_lines    = read_stripped("/tmp/test_lines.txt",    100)
error_section = read_stripped("/tmp/error_section.txt",  60)
raw_output    = read_stripped("/tmp/e2e-targeted-output.txt", 200)

report = f"""\
# 🧪 Rapport E2E ciblé — Nook

Généré automatiquement par `e2e-targeted.yml`
**Dernière mise à jour : {run_date}**

---

## Résumé

| Champ | Valeur |
|-------|--------|
| **Statut** | {status_icon} {status_text} |
| **Suite lancée** | {label} |
| **Filtre Playwright** | `{grep}` |
| **Tests passés** | {passed} |
| **Tests échoués** | {failed} |
| **Tests flaky** | {flaky} |
| **Traces activées** | {traces} |
| **Branche** | `{branch}` |
| **Commit** | [`{commit_short}`]({commit_url}) |
| **Run CI** | [Voir le run complet]({run_url}) |

---

## Résultats par test

```
{test_lines or "(résultats détaillés non disponibles)"}
```

---

## Erreurs détectées

```
{error_section or "Aucune erreur détectée"}
```

---

## Output brut (200 dernières lignes)

```
{raw_output}
```

---

*Rapport généré par `.github/workflows/e2e-targeted.yml`*
"""

out = ".hermes/E2E-TARGETED-REPORT.md"
os.makedirs(".hermes", exist_ok=True)
with open(out, "w", encoding="utf-8") as f:
    f.write(report)

print(f"Rapport écrit : {out}")
print(f"  Statut  : {status_icon} {status_text}")
print(f"  Passés  : {passed}  |  Échoués : {failed}  |  Flaky : {flaky}")
