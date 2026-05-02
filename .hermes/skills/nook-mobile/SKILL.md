---
name: nook-mobile
category: "software-development"
description: "Mobile/PWA testing, responsive design verification, touch interaction testing for Nook."
---

# 📱 Mobile Skill

## Trigger
- "Test mobile", "check responsive", "PWA issues"
- After UI changes
- Before release

## Steps
1. Check manifest: `read_file("frontend/static/manifest.json")`
2. Test responsive: Use browser device toolbar
3. Test touch: Check button sizes (44px min)
4. Test PWA: Install prompt, offline mode
5. Lighthouse: Run performance audit

## Breakpoints
- Mobile: < 720px
- Tablet: 721-1024px
- Desktop: > 1024px

## Report
Save to `.claude/MOBILE-REPORT.md` and push.
