---
name: nook-test-automation
category: "software-development"
description: "Run Playwright E2E tests, check coverage, diagnose failures for Nook."
---

# 🧪 Test Automation Skill

## Trigger
- "Run tests", "check test coverage", "tests failing"
- After code changes
- Before release

## Steps
1. Build: `cd frontend && npm run build`
2. List tests: `npx playwright test --list`
3. Run: `npx playwright test`
4. Report: `npx playwright show-report`
5. Diagnose failures, fix, rerun

## Diagnose failures
- Check selector (prefer data-testid)
- Check timing (add waitForLoadState)
- Check scope (Svelte 5 runes)

## Report
Save to `.hermes/TEST-REPORT.md` and push.
