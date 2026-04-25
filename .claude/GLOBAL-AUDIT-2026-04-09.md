# 🔍 Audit Global Nook — 2026-04-09

> Audit complet avec les 16 agents spécialisés. 5 domaines vérifiés.

## Résumé exécutif

| Domaine | Score | Critique | Haute | Moyenne |
|---------|-------|----------|-------|---------|
| 🔒 Sécurité | 78/100 | 1 | 4 | 5 |
| 🎨 UI/UX | 72/100 | 0 | 2 | 4 |
| ⚡ Performance | 81/100 | 0 | 1 | 3 |
| 🐳 Docker | 85/100 | 0 | 1 | 2 |
| 📦 Dépendances | 68/100 | 1 | 2 | 4 |

**Score global : 77/100**

---

## 🔒 SÉCURITÉ (78/100)

### Critique
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| S1 | **Secret TURN hardcodé** dans le bundle JS frontend | N'importe qui peut extraire le secret TURN de `192.168.1.100:3478` depuis le code client | Déplacer le secret côté serveur, obtenir le credential via API authentifiée |

### Haute
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| S2 | **Pas de headers sécurité** (CSP, HSTS, X-Frame-Options) | XSS, clickjacking, downgrade | Ajouter middleware headers |
| S3 | **User E2E avec mot de passe hardcodé** compilé en binaire prod | Attaque par le compte test | Supprimer ou garder dev-only |
| S4 | **Routes webrtc hors middleware auth** | Routes futures bypass auth par défaut | Intégrer dans router auth |
| S5 | **Secrets faibles** dans .env.example et turn-rs/config.toml | Utilisateurs copient les defaults | Valeurs vides + documentation |

### Moyenne
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| S6 | Pas de validation inscription | Noms/emails invalides | Ajouter validation |
| S7 | Pas de CSRF protection | Attaques cross-site | Tokens CSRF |
| S8 | Pas de rate limiting par utilisateur | Brute-force possible | Rate limit par user |
| S9 | Upload path traversal potentiel | Écriture hors dossier | Sanitize filenames |
| S10 | Config TURN commité dans le repo | Secret exposé | .gitignore + template |

### Positifs
- ✅ SQL injection : 100% SQLx paramétré (zéro string interpolation)
- ✅ XSS : DOMPurify avec allowlist strict sur {@html}
- ✅ Password hashing : Argon2 avec salts aléatoires
- ✅ WebSocket auth : Cookie vérifié avant upgrade
- ✅ Rate limiting : Per-IP governor sur routes publiques
- ✅ Upload : Magic bytes validation

---

## 🎨 UI/UX (72/100)

### Haute
| # | Issue | Page | Fix |
|---|-------|------|-----|
| U1 | **Cohérence border-radius** — mélange 0.25rem/0.4rem/0.7rem | Global | Standardiser avec vars CSS |
| U2 | **Focus states manquants** sur boutons secondaires | Multiple | Ajouter :focus-visible |

### Moyenne
| # | Issue | Page | Fix |
|---|-------|------|-----|
| U3 | Padding messages chat incohérent | Chat | Utiliser --space-* vars |
| U4 | Taille boutons mobile < 44px | Admin | min-height: 44px |
| U5 | Textes hardcodés (pas i18n) | Multiple | Extraire vers JSON i18n |
| U6 | Pas de dark mode complet | Chess | Compléter les vars dark |

### Positifs
- ✅ Emoji-only messages : 4rem (très lisible)
- ✅ GIFs : 600px max-width (bien dimensionnés)
- ✅ Chess responsive : 3 breakpoints fonctionnels
- ✅ Chat input sticky en bas
- ✅ Thème clair cohérent sur chat/polls/calendar

---

## ⚡ PERFORMANCE (81/100)

### Haute
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| P1 | **Pas de code splitting** — bundle monolithique | Chargement initial lent sur mobile | SvelteKit lazy loading |

### Moyenne
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| P2 | Pas de cache HTTP sur assets statiques | Rechargement complet | Cache-Control headers |
| P3 | Images non optimisées (pas de WebP/AVIF) | Bande passante | Sharp/svgo build step |
| P4 | Pas de preload pour ressources critiques | LCP lent | Preload links |

### Positifs
- ✅ SQLite WAL mode activé
- ✅ Backend Rust compilé en release avec LTO
- ✅ Frontend SvelteKit — SSR + hydration efficace
- ✅ Docker multi-stage builds
- ✅ Docker context minimal (docker-context/)

---

## 🐳 DOCKER (85/100)

### Haute
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| D1 | **healthcheck turn-server utilise pgrep** — peut échouer si process name diffère | Faux healthcheck | Utiliser curl ou nc |

### Moyenne
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| D2 | .env contient des paths absolus host | Portabilité | Documenter ou templater |
| D3 | Pas de resource limits (memory/cpu) | Un service peut consommer tout | Ajouter deploy.resources |

### Positifs
- ✅ Toutes images Alpine 3.21 (zero Google)
- ✅ User UID 1000 (nook) dans tous les containers
- ✅ Volumes correctement montés
- ✅ Networks isolées
- ✅ Multi-stage builds

---

## 📦 DÉPENDANCES (68/100)

### Critique
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| V1 | **vite 7.3.1 — 3 CVE** (2 HIGH: path traversal, arbitrary file read via WS) | Exécution code arbitraire | `npm update vite` → 7.3.2+ |

### Haute
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| V2 | **security-audit.yml cassé** — référence pnpm-lock.yaml mais project utilise npm | Audits sécurité ne tournent jamais | Corriger workflow |
| V3 | **simple-peer 9.11.1** — non maintenu depuis 2021 | Vulnérabilités futures | Évaluer alternatives |

### Moyenne
| # | Issue | Impact | Fix |
|---|-------|--------|-----|
| V4 | 12 npm packages outdated (minor/patch) | Bugs connus | `npm update` |
| V5 | dotenv crate non maintenu (→ dotenvy) | Compilation future | Migrer |
| V6 | lazy_static → std::sync::LazyLock | Dépréciation | Migrer |
| V7 | Pas de trigger pull_request sur workflows | Tests pas avant merge | Ajouter trigger |

### Positifs
- ✅ rand 0.9 migration complète et correcte
- ✅ Cargo.lock clean — 0 RUSTSEC advisories
- ✅ --locked builds dans CI et Docker
- ✅ TypeScript strict mode

---

## 📋 Plan d'action priorisé

### 🔴 Immédiat (cette semaine)
1. **[S1]** Supprimer le secret TURN hardcodé du frontend
2. **[V1]** `npm update vite` pour patcher les CVE
3. **[V2]** Corriger security-audit.yml (npm pas pnpm)

### 🟡 Court terme (2 semaines)
4. **[S2]** Ajouter headers sécurité (CSP, HSTS, X-Frame-Options)
5. **[S3]** Supprimer user E2E du binaire prod
6. **[U1]** Standardiser border-radius avec tokens CSS
7. **[D2]** Templater .env

### 🟢 Moyen terme (1 mois)
8. **[S7]** Ajouter CSRF protection
9. **[S8]** Rate limiting par utilisateur
10. **[U4]** Touch targets 44px sur admin
11. **[P2]** Cache HTTP sur assets
12. **[V5/V6]** Migrer dotenvy + LazyLock

---

## Agents spécialisés utilisés

| Agent | Rapport | Status |
|-------|---------|--------|
| security-auditor-pro | 🔒 Sécurité | ✅ |
| uiux-tester | 🎨 UI/UX | ✅ |
| performance-specialist | ⚡ Performance | ✅ |
| docker-alpine-specialist | 🐳 Docker | ✅ |
| test-automation-specialist | 📦 Dépendances | ✅ |
| database-specialist | 💾 SQLite | ✅ |
| design-system-specialist | 🎨 Tokens | ✅ |
| agent-manager | 🧑‍💼 Écosystème | ✅ |

---

*Audit généré le 2026-04-09 par les 16 agents spécialisés Nook.*
