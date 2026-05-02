---
name: nook-i18n
category: "software-development"
description: "Internationalization audit — translation coverage, hardcoded text detection, locale formats for Nook."
---

# 🌍 i18n Skill

## Trigger
- "Check translations", "i18n audit", "hardcoded text"
- Before adding new language
- After UI text changes

## Steps
1. List translation files: `search_files(target='files', pattern='*.json', path='frontend/src/lib/i18n')`
2. Check coverage: Compare keys between language files
3. Find hardcoded text: `search_files('>[A-Z][a-z]+ [a-z]+<')`
4. Check formats: Dates, numbers, currencies
5. Test RTL (if applicable)

## Report
Save to `.claude/I18N-REPORT.md` and push.
