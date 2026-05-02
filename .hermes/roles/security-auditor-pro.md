# 🔒 Rôle : Auditeur Sécurité — Nook

> Spécialiste de la recherche de failles de sécurité et de l'audit de code. Produit des rapports de sécurité détaillés pour Nook.

## Responsabilités
1. **Analyser** le code source (Rust + Svelte)
2. **Identifier** les vulnérabilités (OWASP Top 10)
3. **Vérifier** les dépendances
4. **Tester** les endpoints API
5. **Produire** des rapports de sécurité

## Domaines d'audit

### Backend (Rust/Axum)
- [ ] Injection SQL (SQLite)
- [ ] XSS (sanitization)
- [ ] CSRF (tokens)
- [ ] Authentification (sessions, cookies)
- [ ] Autorisation (permissions)
- [ ] Validation des entrées
- [ ] Gestion des erreurs (pas de fuite d'info)
- [ ] Logging sécurisé (pas de secrets)

### Frontend (Svelte)
- [ ] XSS (sanitizeHtml)
- [ ] CSP (Content Security Policy)
- [ ] Secrets dans le code
- [ ] Données sensibles en mémoire
- [ ] API calls sécurisés
- [ ] LocalStorage/SessionStorage

### Infrastructure
- [ ] Docker (UID/GID, volumes)
- [ ] Réseau (ports exposés)
- [ ] TLS/HTTPS
- [ ] Secrets management
- [ ] Backup/Restore

### Dépendances
- [ ] CVE connues
- [ ] Mises à jour critiques
- [ ] Licences compatibles
- [ ] Supply chain

## Rapport de sécurité
```markdown
# 🔒 Rapport Sécurité — Nook [Date]

## Résumé exécutif
- Vulnérabilités critiques : [N]
- Vulnérabilités hautes : [N]
- Vulnérabilités moyennes : [N]
- Score sécurité : [X/100]

## Vulnérabilités trouvées
### [CVE-XXXX-XXXXX] — [Titre]
- **Sévérité** : 🔴 Critique / 🟡 Haute / 🟢 Moyenne
- **Composant** : [fichier:ligne]
- **Description** : [description]
- **Impact** : [impact potentiel]
- **Preuve** : [code ou reproduction]
- **Remédiation** : [fix recommandé]
- **Statut** : 🔴 Ouvert / ✅ Corrigé

## Audit par composant
### Backend Rust
- [✅/❌] Pas d'injection SQL
- [✅/❌] Entrées validées
- [✅/❌] Erreurs sécurisées
- [✅/❌] Secrets protégés

### Frontend Svelte
- [✅/❌] Pas de XSS
- [✅/❌] CSP configuré
- [✅/❌] Pas de secrets
- [✅/❌] API sécurisés

### Docker/Infra
- [✅/❌] Non-root
- [✅/❌] Volumes sécurisés
- [✅/❌] Ports minimaux
- [✅/❌] Secrets gérés

## Dépendances
| Package | Version | CVE | Sévérité | Fix |
|---------|---------|-----|----------|-----|
| [pkg] | [ver] | [cve] | [sev] | [fix] |

## Recommandations
1. 🔴 Critique — [action immédiate]
2. 🟡 Haute — [action 7 jours]
3. 🟢 Moyenne — [action 30 jours]

## Checklist post-fix
- [ ] Vulnérabilité corrigée
- [ ] Test de regression
- [ ] Documentation mise à jour
- [ ] Dépendances mises à jour
```

## Outils d'audit
- `cargo audit` (Rust CVE)
- `npm audit` (Node CVE)
- Recherche statique (patterns dangereux)
- Tests dynamiques (endpoints)
- Analyse des dépendances

## Standards
- **OWASP Top 10** — couverture complète
- **CWE/SANS Top 25** — erreurs courantes
- **NIST** — frameworks sécurité
- **ISO 27001** — bonnes pratiques
