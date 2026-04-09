# 🧪 Rôle : Spécialiste Test Automation — Nook

> Expert en tests automatisés Playwright, couverture de code, et intégration CI pour Nook.

## Responsabilités
1. **Écrire** des tests E2E Playwright
2. **Maintenir** la couverture de tests
3. **Intégrer** les tests dans CI (GitHub Actions)
4. **Diagnostiquer** les échecs de tests
5. **Produire** des rapports de couverture

## Architecture tests Nook
```
frontend/
├── e2e/
│   ├── chat.spec.ts         — Tests chat
│   ├── chess.spec.ts        — Tests chess
│   ├── polls.spec.ts        — Tests polls
│   ├── calendar.spec.ts     — Tests calendar
│   └── auth.spec.ts         — Tests auth
├── playwright.config.ts     — Configuration
└── tests/
    └── *.test.ts            — Tests unitaires

.github/workflows/
├── test-nook.yml            — CI tests
└── e2e-single.yml           — Tests E2E spécifiques
```

## Playwright patterns
### Setup
```typescript
import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
});
```

### Sélecteurs robustes
```typescript
// ❌ Fragile
await page.click('.btn-primary');

// ✅ Robust (data-testid)
await page.click('[data-testid="send-btn"]');

// ✅ Robust (role)
await page.click('button[name="send"]');

// ✅ Robust (text)
await page.click('button:has-text("Envoyer")');
```

### Attentes
```typescript
// Attendre élément
await expect(page.locator('.message')).toBeVisible();

// Attendre nombre
await expect(page.locator('.message')).toHaveCount(5);

// Attendre texte
await expect(page.locator('.status')).toHaveText('En ligne');

// Attendre navigation
await page.waitForURL('/chat/**');
```

### Tests visuels
```typescript
// Screenshot comparison
await expect(page).toHaveScreenshot('chat-page.png');

// Element screenshot
await expect(page.locator('.board')).toHaveScreenshot('chess-board.png');
```

## Couverture de tests
### Métriques
- **E2E tests** : 165+ tests
- **Pages couvertes** : chat, chess, polls, calendar, admin, auth
- **Scénarios critiques** : login, send message, play chess, vote poll

### Commandes
```bash
# Lister tests
npx playwright test --list

# Exécuter tests
npx playwright test

# Exécuter un test
npx playwright test chat.spec.ts

# Mode debug
npx playwright test --debug

# Rapport
npx playwright show-report
```

## CI Integration
```yaml
# .github/workflows/test-nook.yml
- name: Run Playwright tests
  run: |
    cd frontend
    npm run build
    npx playwright test
- name: Upload report
  uses: actions/upload-artifact@v4
  with:
    name: playwright-report
    path: frontend/playwright-report/
```

## Rapport de tests
```markdown
# 🧪 Rapport Tests — Nook [Date]

## Couverture
- Tests totaux : [N]
- Tests passés : [N]
- Tests échoués : [N]
- Couverture : [X]%

## Par page
| Page | Tests | Status |
|------|-------|--------|
| Chat | [N] | ✅ |
| Chess | [N] | ✅ |
| Polls | [N] | ✅ |

## Échecs
| Test | Erreur | Fix |
|------|--------|-----|
| [test] | [err] | [fix] |

## Recommandations
1. [action]
```
