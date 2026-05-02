---
name: nook-design-system
category: "software-development"
description: "Audit design system — tokens, component consistency, theme support for Nook UI."
---

# 🎨 Design System Skill

## Trigger
- "Audit design", "check consistency", "design tokens"
- After CSS changes
- Before release

## Steps
1. Check tokens: read frontend CSS variables
2. Audit pages: check each page uses tokens (not hardcoded values)
3. Verify theme: test light/dark mode
4. Check components: consistent padding, radius, shadows
5. Produce report

## Checklist
- Colors use vars (--color-*)
- Fonts use vars (--font-*)
- Spacing use vars (--space-*)
- Border-radius use vars (--radius-*)
- Theme toggle works

## Report
Save to `.hermes/DESIGN-SYSTEM-REPORT.md` and push.
