---
name: nook-uiux-test
category: "software-development"
description: "Test UI/UX across pages — visual, ergonomic, responsive, accessible. Produces structured report in .claude/UIUX-REPORT.md"
---

# 🎨 UI/UX Test Skill

## Trigger
Run when:
- "Test UI", "audit UX", "check design"
- After major UI changes
- Before release
- User reports UI issues

## Steps

### 1. Navigate and screenshot each page
For each page (chat, chess, polls, calendar, admin, webrtc):
```bash
# Use browser tools
browser_navigate("http://localhost:6300/[page]")
browser_vision("What issues do you see with layout, spacing, readability?")
browser_snapshot()  # accessibility tree
browser_console()   # JS errors
```

### 2. Check responsive
```bash
# Desktop (>1024px)
browser_navigate("http://localhost:6300/[page]")
# Check layout

# Tablet (768px)  
# Use browser_viewport if available
# Or check CSS media queries

# Mobile (375px)
# Use browser_viewport if available
```

### 3. Check accessibility
- Tab navigation works
- Focus visible
- Labels present
- Contrast OK

### 4. Produce report
Save to `.claude/UIUX-REPORT.md` and push to GitHub.

## Report template
```markdown
# 🎨 Rapport UI/UX — Nook [Date]

## Score global : [X/10]

## Page: Chat
- [✅/❌] Emojis 4rem
- [✅/❌] GIFs 600px
- [✅/❌] Input sticky
- [✅/❌] Scroll auto

## Page: Chess
- [✅/❌] Pieces 3.8rem
- [✅/❌] Board centered
- [✅/❌] Responsive 3 breakpoints

## Problèmes
| Page | Issue | Priority | Fix |
|------|-------|----------|-----|
| [p]  | [desc]| [🔴🟡🟢]| [fix]|
```
