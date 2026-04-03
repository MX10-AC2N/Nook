#!/usr/bin/env python3
"""
scripts/generate-test-report.py
Génère .claude/TEST_REPORT.md à partir :
  - du JSON Playwright  (généré par --reporter=json)
  - des logs Docker     (fichier)
  - des variables CI    (env : RUN_DATE, RUN_URL, COMMIT_SHA, BRANCH, REPO, PLAYWRIGHT_EXIT)

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
      stats       : passed, failed, skipped, flaky, total, duration_ms
      suites      : liste de { name, tests: [...] }
      all_tests   : liste plate de tous les tests (avec timing, status, errors, file)
      failures    : liste des tests échoués avec détail complet
      slowest     : liste des tests les plus lents (top 10)
      by_file     : dict { filename: [tests] }
      by_category : dict { category: {passed, failed, skipped, tests} }
    """
    result = {
        "stats": {"passed": 0, "failed": 0, "skipped": 0, "flaky": 0, "total": 0, "duration_ms": 0},
        "suites": [],
        "all_tests": [],
        "failures": [],
        "slowest": [],
        "by_file": {},
        "by_category": {},
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

    stats = data.get("stats", {})
    result["stats"] = {
        "passed":      stats.get("expected", 0),
        "failed":      stats.get("unexpected", 0),
        "skipped":     stats.get("skipped", 0),
        "flaky":       stats.get("flaky", 0),
        "total":       stats.get("expected", 0) + stats.get("unexpected", 0) + stats.get("skipped", 0) + stats.get("flaky", 0),
        "duration_ms": stats.get("duration", 0),
    }

    def walk_suite(suite, path_parts: list):
        suite_title = suite.get("title", "")
        current_path = path_parts + ([suite_title] if suite_title else [])

        suite_tests = []

        for spec in suite.get("specs", []):
            spec_title = spec.get("title", "")
            spec_location = spec.get("location", {})
            spec_file = spec_location.get("file", "")

            for test in spec.get("tests", []):
                status = test.get("status", "unknown")
                if status == "expected":
                    status_label = "passed"
                elif status == "unexpected":
                    status_label = "failed"
                elif status == "flaky":
                    status_label = "flaky"
                else:
                    status_label = "skipped"

                # Per-run timing (multiple results = retries)
                run_results = test.get("results", [])
                duration_ms = sum(r.get("duration", 0) for r in run_results)
                retry_count = len(run_results) - 1 if len(run_results) > 1 else 0

                # Determine if flaky: passed on retry but failed first
                is_flaky = False
                if len(run_results) > 1:
                    statuses = [r.get("status", "") for r in run_results]
                    if "unexpected" in statuses and "expected" in statuses:
                        is_flaky = True
                        result["stats"]["flaky"] += 1
                        if status_label != "flaky":
                            status_label = "flaky"

                errors = []
                for res_idx, res in enumerate(run_results):
                    for err in res.get("errors", []):
                        msg = strip_ansi(err.get("message", ""))
                        loc = res.get("location", {})
                        snippet = strip_ansi(err.get("snippet", ""))
                        stack = strip_ansi(err.get("stack", ""))
                        errors.append({
                            "message": msg,
                            "file": loc.get("file", ""),
                            "line": loc.get("line", "?"),
                            "snippet": snippet,
                            "stack": stack,
                            "retry": res_idx,
                        })

                test_info = {
                    "title":       " > ".join(filter(None, current_path + [spec_title])),
                    "short_title": spec_title,
                    "suite":       " > ".join(filter(None, current_path)),
                    "status":      status_label,
                    "duration_ms": duration_ms,
                    "errors":      errors,
                    "file":        spec_file,
                    "retries":     retry_count,
                    "is_flaky":    is_flaky,
                }

                suite_tests.append(test_info)
                result["all_tests"].append(test_info)

                # Group by file
                fname = Path(spec_file).name if spec_file else "unknown"
                result["by_file"].setdefault(fname, []).append(test_info)

                # Group by category (first describe block = category)
                category = current_path[0] if current_path else "Autres"
                cat_data = result["by_category"].setdefault(category, {"passed": 0, "failed": 0, "skipped": 0, "flaky": 0, "tests": []})
                cat_data["tests"].append(test_info)
                if status_label == "passed":
                    cat_data["passed"] += 1
                elif status_label == "failed":
                    cat_data["failed"] += 1
                elif status_label == "flaky":
                    cat_data["flaky"] += 1
                else:
                    cat_data["skipped"] += 1

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

    # Compute top 10 slowest tests
    result["slowest"] = sorted(result["all_tests"], key=lambda t: t["duration_ms"], reverse=True)[:10]

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
        if re.search(r'ERROR|WARN|panic|thread.*panicked|FATAL|CRITICAL', clean, re.IGNORECASE):
            clean = re.sub(r'^.*nook\s+\|\s+', '', clean)
            clean = re.sub(r'^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s+', '', clean)
            lines.append(clean.strip())
    return lines[:50]


# ─────────────────────────────────────────────────────────────────────────────
# Formatage Markdown
# ─────────────────────────────────────────────────────────────────────────────

def fmt_duration(ms: int) -> str:
    if ms <= 0:
        return "N/A"
    s = ms / 1000.0
    if s >= 60:
        m = int(s // 60)
        sec = s % 60
        return f"{m}m {sec:.1f}s"
    return f"{s:.1f}s"


def build_report(pw: dict, docker_warnings: list[str], ctx: dict) -> str:
    stats = pw["stats"]
    passed  = stats["passed"]
    failed  = stats["failed"]
    skipped = stats["skipped"]
    flaky   = stats["flaky"]
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
    prev_report = ctx.get("prev_report", "")

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
    flaky_badge = f" · ⚠️ **{flaky} flaky**" if flaky > 0 else ""
    lines += [
        "## 📊 Résumé",
        "",
        "| Indicateur | Valeur |",
        "|-----------|--------|",
        f"| **Statut** | {status_icon} **{status_text}**{flaky_badge} |",
        f"| **Tests passés** | {passed} |",
        f"| **Tests échoués** | {failed} |",
        f"| **Tests flaky** | {flaky} |",
        f"| **Tests ignorés** | {skipped} |",
        f"| **Total** | {total} |",
        f"| **Durée totale** | {dur} |",
        f"| **Branche** | `{branch}` |",
        f"| **Commit** | [`{commit_short}`](https://github.com/{repo}/commit/{commit_sha}) |",
        f"| **Run CI** | [Voir le run complet]({run_url}) |",
        "",
        "---",
        "",
    ]

    # ── Architecture / Suites ─────────────────────────────────────────────────
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

    # ── ⏱️ Top 10 Slowest Tests (Hotspot Detection) ─────────────────────────
    if pw["slowest"]:
        lines += [
            "## ⏱️ Top 10 Tests les plus lents (Performance Hotspots)",
            "",
            "| Rang | Test | Durée | Fichier |",
            "|------|------|-------|---------|",
        ]
        for i, t in enumerate(pw["slowest"], 1):
            fname = Path(t["file"]).name if t["file"] else "?"
            title = t["short_title"].replace("|", "\\|")
            lines.append(f"| {i} | {title} | {fmt_duration(t['duration_ms'])} | `{fname}` |")
        lines += ["", "---", ""]

    # ── Résultats par catégorie ───────────────────────────────────────────────
    if pw["by_category"]:
        lines += [
            "## 📋 Résultats par catégorie",
            "",
        ]
        # Sort by failed first, then alphabetically
        sorted_cats = sorted(pw["by_category"].items(),
                            key=lambda x: (-x[1]["failed"], -x[1]["flaky"], x[0]))

        for cat_name, cat_data in sorted_cats:
            c_passed = cat_data["passed"]
            c_failed = cat_data["failed"]
            c_flaky  = cat_data["flaky"]
            c_skipped = cat_data["skipped"]
            c_total = len(cat_data["tests"])
            icon = "✅" if c_failed == 0 and c_flaky == 0 else "❌"

            lines.append(f"### {icon} **{cat_name}** — {c_passed}/{c_total} passés" +
                        (f" · ⚠️ {c_flaky} flaky" if c_flaky > 0 else "") +
                        (f" · ❌ {c_failed} failed" if c_failed > 0 else ""))
            lines.append("")
            lines.append("| Statut | Test | Durée | Retries |")
            lines.append("|--------|------|-------|---------|")
            # Sort: failed first, then flaky, then by duration desc
            sorted_tests = sorted(cat_data["tests"],
                                 key=lambda t: (0 if t["status"] == "failed" else 1 if t["status"] == "flaky" else 2,
                                               -t["duration_ms"]))
            for t in sorted_tests:
                s_icon = {"passed": "✅", "failed": "❌", "flaky": "⚠️", "skipped": "⏭️"}.get(t["status"], "❓")
                dur_t = fmt_duration(t["duration_ms"])
                retries = f"  +{t['retries']}" if t["retries"] > 0 else ""
                title = t["short_title"].replace("|", "\\|")
                lines.append(f"| {s_icon} | {title} | {dur_t}{retries} | |")
            lines.append("")

        lines += ["---", ""]

    # ── Résultats par fichier ─────────────────────────────────────────────────
    if pw["by_file"]:
        lines += [
            "## 📁 Résultats par fichier de test",
            "",
            "| Fichier | ✅ Passés | ❌ Échoués | ⚠️ Flaky | Total |",
            "|---------|-----------|-------------|-----------|-------|",
        ]
        for fname in sorted(pw["by_file"].keys()):
            tests = pw["by_file"][fname]
            f_p = sum(1 for t in tests if t["status"] == "passed")
            f_f = sum(1 for t in tests if t["status"] == "failed")
            f_fl = sum(1 for t in tests if t["status"] == "flaky")
            f_s = sum(1 for t in tests if t["status"] == "skipped")
            icon = "✅" if f_f == 0 and f_fl == 0 else "❌"
            lines.append(f"| {icon} `{fname}` | {f_p} | {f_f} | {f_fl} | {len(tests)} |")
        lines += ["", "---", ""]

    # ── Flaky Test Detection ──────────────────────────────────────────────────
    flaky_tests = [t for t in pw["all_tests"] if t["is_flaky"]]
    if flaky_tests:
        lines += [
            "## ⚠️ Tests Flaky Détectés (Instables)",
            "",
            "> Ces tests ont échoué puis réussi lors d'un retry. Ils indiquent une non-déterminisme.",
            "",
        ]
        for t in flaky_tests:
            lines.append(f"- ❌ **`{t['short_title']}`** dans `{t['suite']}`")
            lines.append(f"  - Fichier: `{t['file']}`")
            lines.append(f"  - Durée: {fmt_duration(t['duration_ms'])}")
            if t["errors"]:
                err = t["errors"][0]
                lines.append(f"  - Erreur: `{err['message'][:120]}`")
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
            if t["file"]:
                lines.append(f"**Fichier :** `{t['file']}`")
            lines.append(f"**Durée :** {fmt_duration(t['duration_ms'])}")
            lines.append("")
            for err in t["errors"]:
                if err["file"]:
                    lines.append(f"**Fichier d'erreur :** `{err['file']}` ligne {err['line']}")
                    lines.append("")
                if err["message"]:
                    msg_lines = err["message"].splitlines()[:30]
                    lines.append("**Message :**")
                    lines.append("```")
                    lines.extend(msg_lines)
                    lines.append("```")
                    lines.append("")
                if err["stack"]:
                    stack_lines = err["stack"].splitlines()[:15]
                    lines.append("**Stack Trace :**")
                    lines.append("```")
                    lines.extend(stack_lines)
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

    # ── Comparaison avec run précédent ────────────────────────────────────────
    if prev_report and os.path.exists(prev_report):
        try:
            prev_text = Path(prev_report).read_text(encoding="utf-8")
            # Extract previous stats
            prev_pass = re.search(r'passés\*\* \| (\d+)', prev_text)
            prev_fail = re.search(r'échoués\*\* \| (\d+)', prev_text)
            if prev_pass and prev_fail:
                pp, pf = int(prev_pass.group(1)), int(prev_fail.group(1))
                diff_p = passed - pp
                diff_f = failed - pf
                lines += [
                    "## 📈 Comparaison avec le run précédent",
                    "",
                    "| Métrique | Précédent | Actuel | Diff |",
                    "|----------|-----------|--------|------|",
                    f"| **Passés** | {pp} | {passed} | {'+' if diff_p >= 0 else ''}{diff_p} |",
                    f"| **Échoués** | {pf} | {failed} | {'+' if diff_f >= 0 else ''}{diff_f} |",
                    "",
                ]
                if diff_f > 0:
                    lines.append(f"> ⚠️ **{diff_f} nouveaux échecs** détectés !")
                elif diff_f < 0:
                    lines.append(f"> ✅ **{-diff_f} échecs résolus** depuis le dernier run.")
                lines += ["", "---", ""]
        except Exception as e:
            print(f"[WARN] Erreur comparaison: {e}", file=sys.stderr)

    # ── HTML Reporter Artifact Reference ──────────────────────────────────────
    lines += [
        "## 🖼️ Rapport HTML Playwright",
        "",
        f"> Le rapport HTML complet est disponible en artifact GitHub Actions.",
        f">",
        f"> - **Nom de l'artifact :** `playwright-report`",
        f"> - **URL du run :** [{run_url}]({run_url})",
        f"> - **Chemin local (CI) :** `frontend/playwright-report/`",
        "",
        "Pour examiner visuellement les échecs :",
        f"1. Télécharger l'artifact `playwright-report` depuis le [run CI]({run_url})",
        f"2. Ouvrir `index.html` dans un navigateur",
        f"3. Utiliser l'interface pour explorer les traces et screenshots",
        "",
        "---",
        "",
    ]

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

    # Read the playwright JSON from --json, the playwright output, or stdout
    parser.add_argument("--json",   default="/tmp/playwright-results.json",
                        help="Chemin vers results.json Playwright")
    parser.add_argument("--docker", default="/tmp/docker.log",
                        help="Chemin vers le fichier de logs Docker")
    parser.add_argument("--output", default=".claude/TEST_REPORT.md",
                        help="Chemin du rapport MD à écrire")
    args = parser.parse_args()

    # Contexte CI depuis les variables d'environnement
    ctx = {
        "playwright_exit": os.environ.get("PLAYWRIGHT_EXIT", "1"),
        "run_date":        os.environ.get("RUN_DATE",  datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")),
        "run_url":         os.environ.get("RUN_URL",   "#"),
        "commit_sha":      os.environ.get("COMMIT_SHA", "unknown"),
        "repo":            os.environ.get("REPO",       "MX10-AC2N/Nook"),
        "branch":          os.environ.get("BRANCH",     "develop"),
        "prev_report":     os.environ.get("PREV_REPORT", ""),
    }

    print(f"[INFO] Lecture JSON : {args.json}")
    pw = parse_playwright_json(args.json)

    docker_warnings = []
    if args.docker and Path(args.docker).exists():
        print(f"[INFO] Lecture logs Docker : {args.docker}")
        docker_warnings = parse_docker_logs(args.docker)

    print(f"[INFO] Stats : {pw['stats']['passed']} passed, {pw['stats']['failed']} failed, {pw['stats']['skipped']} skipped, {pw['stats']['flaky']} flaky")
    print(f"[INFO] Échecs : {len(pw['failures'])}")
    print(f"[INFO] Slowest: {len(pw['slowest'])} tests logged")
    print(f"[INFO] Catégories: {list(pw['by_category'].keys())}")

    report = build_report(pw, docker_warnings, ctx)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding="utf-8")

    print(f"[OK] Rapport écrit : {args.output} ({len(report.splitlines())} lignes)")
    sys.exit(0)


if __name__ == "__main__":
    main()
