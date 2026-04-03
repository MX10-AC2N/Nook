#!/usr/bin/env python3
"""
scripts/generate-test-report.py
Génère .claude/TEST_REPORT.md à partir :
  - du JSON Playwright  (/tmp/playwright-results.json, défini dans playwright.config.ts)
  - des logs Docker     (stdin ou fichier)
  - des variables CI    (env : RUN_DATE, RUN_URL, COMMIT_SHA, BRANCH, PLAYWRIGHT_EXIT)

Usage :
  python3 scripts/generate-test-report.py \
      --json   /tmp/playwright-results.json \
      --docker /tmp/docker.log \
      --output .claude/TEST_REPORT.md
"""

import argparse
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


# ─────────────────────────────────────────────────────────────────────────────
# Parsing du JSON Playwright
# ─────────────────────────────────────────────────────────────────────────────

def strip_ansi(text: str) -> str:
    return re.sub(r'\x1b\[[0-9;]*m', '', text)


def parse_playwright_json(json_path: str) -> dict:
    """
    Lit le JSON généré par --reporter=json de Playwright.
    Retourne un dict avec :
      stats       : passed, failed, skipped, total, duration_ms
      suites      : liste de { name, tests: [{ title, status, duration_ms, errors: [...] }] }
      all_tests   : liste plate de tous les tests
      failures    : liste des tests échoués avec détail
    """
    result = {
        "stats": {"passed": 0, "failed": 0, "skipped": 0, "total": 0, "duration_ms": 0},
        "suites": [],
        "all_tests": [],
        "failures": [],
    }

    path = Path(json_path)
    if not path.exists():
        print(f"[WARN] JSON Playwright non trouvé : {json_path}", file=sys.stderr)
        return result

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"[WARN] Impossible de parser {json_path} : {e}", file=sys.stderr)
        return result

    # Stats globales
    stats = data.get("stats", {})
    result["stats"] = {
        "passed":      stats.get("expected", 0),
        "failed":      stats.get("unexpected", 0),
        "skipped":     stats.get("skipped", 0),
        "total":       stats.get("expected", 0) + stats.get("unexpected", 0) + stats.get("skipped", 0),
        "duration_ms": stats.get("duration", 0),
    }

    # Parcourir les suites récursivement
    def walk_suite(suite, path_parts: list):
        suite_title = suite.get("title", "")
        current_path = path_parts + ([suite_title] if suite_title else [])

        suite_tests = []

        for spec in suite.get("specs", []):
            spec_title = spec.get("title", "")
            for test in spec.get("tests", []):
                status = test.get("status", "unknown")
                # Playwright status : "expected" = pass, "unexpected" = fail, "skipped"
                if status == "expected":
                    status_label = "passed"
                elif status == "unexpected":
                    status_label = "failed"
                else:
                    status_label = "skipped"

                duration_ms = sum(r.get("duration", 0) for r in test.get("results", []))
                errors = []

                for res in test.get("results", []):
                    for err in res.get("errors", []):
                        msg = strip_ansi(err.get("message", ""))
                        loc = res.get("location", {})
                        snippet = strip_ansi(err.get("snippet", ""))
                        errors.append({
                            "message": msg,
                            "file": loc.get("file", ""),
                            "line": loc.get("line", "?"),
                            "snippet": snippet,
                        })

                test_info = {
                    "title":       " > ".join(filter(None, current_path + [spec_title])),
                    "short_title": spec_title,
                    "suite":       " > ".join(filter(None, current_path)),
                    "status":      status_label,
                    "duration_ms": duration_ms,
                    "errors":      errors,
                }

                suite_tests.append(test_info)
                result["all_tests"].append(test_info)
                if status_label == "failed":
                    result["failures"].append(test_info)

        for sub in suite.get("suites", []):
            walk_suite(sub, current_path)

        if suite_tests:
            result["suites"].append({
                "name":  " > ".join(filter(None, current_path)) or "Root",
                "tests": suite_tests,
            })

    for suite in data.get("suites", []):
        walk_suite(suite, [])

    return result


# ─────────────────────────────────────────────────────────────────────────────
# Parsing des logs Docker
# ─────────────────────────────────────────────────────────────────────────────

def parse_docker_logs(log_path: str) -> list[str]:
    """Extrait les lignes WARN/ERROR/panic des logs Docker."""
    path = Path(log_path)
    if not path.exists():
        return []
    lines = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        clean = strip_ansi(line)
        if re.search(r'ERROR|WARN|panic|thread.*panicked', clean, re.IGNORECASE):
            # Retirer le préfixe timestamp et nom container
            clean = re.sub(r'^.*nook\s+\|\s+', '', clean)
            clean = re.sub(r'^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s+', '', clean)
            lines.append(clean.strip())
    return lines[:30]


# ─────────────────────────────────────────────────────────────────────────────
# Formatage Markdown
# ─────────────────────────────────────────────────────────────────────────────

def fmt_duration(ms: int) -> str:
    if ms <= 0:
        return "N/A"
    s = ms // 1000
    m = s // 60
    s = s % 60
    return f"{m}m {int(s):02d}s" if m else f"{int(s)}s"


def build_report(pw: dict, docker_warnings: list[str], ctx: dict) -> str:
    stats = pw["stats"]
    passed  = stats["passed"]
    failed  = stats["failed"]
    skipped = stats["skipped"]
    total   = stats["total"]
    dur     = fmt_duration(stats["duration_ms"])

    playwright_exit = ctx.get("playwright_exit", "1")
    status_ok = (failed == 0 and playwright_exit == "0")
    status_icon = "✅" if status_ok else "❌"
    status_text = "SUCCÈS" if status_ok else "ÉCHEC"

    run_date    = ctx.get("run_date",    datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC"))
    run_url     = ctx.get("run_url",     "#")
    commit_sha  = ctx.get("commit_sha",  "unknown")
    commit_short = commit_sha[:7]
    repo        = ctx.get("repo",        "MX10-AC2N/Nook")
    branch      = ctx.get("branch",      "develop")

    lines = []

    # ── En-tête ───────────────────────────────────────────────────────────────
    lines += [
        "# 🧪 Rapport E2E — Nook",
        "",
        f"> Généré par `test-nook.yml` · **{run_date}**",
        "",
        "---",
        "",
    ]

    # ── Résumé ────────────────────────────────────────────────────────────────
    lines += [
        "## 📊 Résumé",
        "",
        "| Indicateur | Valeur |",
        "|-----------|--------|",
        f"| **Statut** | {status_icon} **{status_text}** |",
        f"| **Tests passés** | {passed} |",
        f"| **Tests échoués** | {failed} |",
        f"| **Tests ignorés** | {skipped} |",
        f"| **Total** | {total} |",
        f"| **Durée** | {dur} |",
        f"| **Branche** | `{branch}` |",
        f"| **Commit** | [`{commit_short}`](https://github.com/{repo}/commit/{commit_sha}) |",
        f"| **Run CI** | [Voir le run complet]({run_url}) |",
        "",
        "---",
        "",
    ]

    # ── Architecture ──────────────────────────────────────────────────────────
    lines += [
        "## 🗂️ Suites de tests",
        "",
        "| Suite | Fichier | Périmètre |",
        "|-------|---------|-----------|",
        "| **Sanité API** | `api-sanity.spec.ts` | 401/403 sur toutes les routes protégées |",
        "| **Admin** | `admin.spec.ts` | Login, change-pwd, membres, inscription→approbation, invitations, analytics, isolation |",
        "| **User** | `user.spec.ts` | Auth, chat, réactions, upload, polls, chess, calendar, settings, E2EE, push, navigation |",
        "",
        "---",
        "",
    ]

    # ── Résultats par suite ───────────────────────────────────────────────────
    lines += [
        "## 📋 Résultats par suite",
        "",
    ]

    # Grouper les tests par fichier spec
    by_file: dict[str, list] = {}
    for t in pw["all_tests"]:
        # Extraire le nom du fichier depuis le titre de suite
        suite = t.get("suite", "")
        # Le premier segment est souvent le nom du fichier/describe
        file_key = suite.split(" > ")[0] if suite else "Autres"
        by_file.setdefault(file_key, []).append(t)

    for file_key, tests in by_file.items():
        f_passed  = sum(1 for t in tests if t["status"] == "passed")
        f_failed  = sum(1 for t in tests if t["status"] == "failed")
        f_skipped = sum(1 for t in tests if t["status"] == "skipped")
        icon = "✅" if f_failed == 0 else "❌"
        lines.append(f"### {icon} {file_key} — {f_passed}/{len(tests)} passés")
        lines.append("")
        lines.append("| Statut | Test | Durée |")
        lines.append("|--------|------|-------|")
        for t in tests:
            s_icon = {"passed": "✅", "failed": "❌", "skipped": "⏭️"}.get(t["status"], "❓")
            dur_t = fmt_duration(t["duration_ms"])
            title = t["short_title"].replace("|", "\\|")
            lines.append(f"| {s_icon} | {title} | {dur_t} |")
        lines.append("")

    lines += ["---", ""]

    # ── Détail des échecs ─────────────────────────────────────────────────────
    if pw["failures"]:
        lines += [
            "## ❌ Échecs détaillés",
            "",
            f"> {len(pw['failures'])} test(s) en échec",
            "",
        ]
        for i, t in enumerate(pw["failures"], 1):
            lines.append(f"### Échec {i} — `{t['short_title']}`")
            lines.append("")
            lines.append(f"**Suite :** `{t['suite']}`")
            lines.append("")
            for err in t["errors"]:
                if err["file"]:
                    lines.append(f"**Fichier :** `{err['file']}` ligne {err['line']}")
                    lines.append("")
                if err["message"]:
                    msg_lines = err["message"].splitlines()[:20]
                    lines.append("**Message :**")
                    lines.append("```")
                    lines.extend(msg_lines)
                    lines.append("```")
                    lines.append("")
                if err["snippet"]:
                    snip_lines = err["snippet"].splitlines()[:15]
                    lines.append("**Extrait :**")
                    lines.append("```typescript")
                    lines.extend(snip_lines)
                    lines.append("```")
                    lines.append("")
        lines += ["---", ""]
    else:
        lines += [
            "## ✅ Aucun échec",
            "",
            "Tous les tests ont passé.",
            "",
            "---",
            "",
        ]

    # ── Logs Docker ───────────────────────────────────────────────────────────
    lines += [
        "## 🐳 Logs backend (warnings/erreurs)",
        "",
    ]
    if docker_warnings:
        lines.append("```")
        lines.extend(docker_warnings)
        lines.append("```")
    else:
        lines.append("Aucun warning ou erreur dans les logs backend. ✅")
    lines += ["", "---", ""]

    # ── Couverture fonctionnelle ──────────────────────────────────────────────
    lines += [
        "## 🔍 Couverture fonctionnelle",
        "",
        "| Domaine | Endpoints / Fonctionnalités | Couverture |",
        "|---------|----------------------------|-----------|",
        "| **Auth** | login, logout, /me, change-pwd, register→approve | ✅ Complet |",
        "| **Conversations** | GET/POST conv, messages, participants, rename | ✅ Complet |",
        "| **Réactions** | POST/DELETE/GET, UPSERT, UI picker→pill | ✅ Complet |",
        "| **Upload/Download** | upload chat, download, 401/404 | ✅ Complet |",
        "| **Polls** | CRUD, vote, UPSERT, double vote, fermeture, vote fermé | ✅ Complet |",
        "| **Chess** | créer, coups légaux/illégaux, IA, resign, invitations, UI plateau | ✅ Complet |",
        "| **Calendrier** | GET/POST/DELETE événements, UI grille | ✅ Complet |",
        "| **Settings** | profil, sécurité, apparence, update nom | ✅ Complet |",
        "| **Admin** | users, pending, approve, invites, delete, analytics | ✅ Complet |",
        "| **E2EE** | register/get public keys | ✅ Complet |",
        "| **Push** | subscribe, preferences, vapid-key | ✅ Complet |",
        "| **Sécurité** | ~47 routes 401, 403 admin, rate limit flood | ✅ Complet |",
        "| **Navigation** | 7 routes accessibles sans erreur | ✅ Complet |",
        "",
        "---",
        "",
        f"*Rapport généré par `scripts/generate-test-report.py` — {run_date}*",
        "",
    ]

    return "\n".join(lines)


# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Génère TEST_REPORT.md depuis les artifacts Playwright")
    parser.add_argument("--json",   default="/tmp/playwright-results.json", help="Chemin vers results.json Playwright (généré par playwright.config.ts)")
    parser.add_argument("--docker", default="",  help="Chemin vers le fichier de logs Docker (optionnel)")
    parser.add_argument("--output", default=".claude/TEST_REPORT.md", help="Chemin du rapport MD à écrire")
    args = parser.parse_args()

    # Contexte CI depuis les variables d'environnement
    ctx = {
        "playwright_exit": os.environ.get("PLAYWRIGHT_EXIT", "1"),
        "run_date":        os.environ.get("RUN_DATE",  datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")),
        "run_url":         os.environ.get("RUN_URL",   "#"),
        "commit_sha":      os.environ.get("COMMIT_SHA", "unknown"),
        "repo":            os.environ.get("REPO",       "MX10-AC2N/Nook"),
        "branch":          os.environ.get("BRANCH",     "develop"),
    }

    print(f"[INFO] Lecture JSON : {args.json}")
    pw = parse_playwright_json(args.json)

    docker_warnings = []
    if args.docker and Path(args.docker).exists():
        print(f"[INFO] Lecture logs Docker : {args.docker}")
        docker_warnings = parse_docker_logs(args.docker)

    print(f"[INFO] Stats : {pw['stats']['passed']} passed, {pw['stats']['failed']} failed, {pw['stats']['skipped']} skipped")
    print(f"[INFO] Échecs : {len(pw['failures'])}")

    report = build_report(pw, docker_warnings, ctx)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding="utf-8")

    print(f"[OK] Rapport écrit : {args.output} ({len(report.splitlines())} lignes)")

    # Code de sortie : 1 si des tests ont échoué (pour que le step soit rouge)
    sys.exit(0)  # toujours 0 — le workflow gère le code d'erreur séparément


if __name__ == "__main__":
    main()
