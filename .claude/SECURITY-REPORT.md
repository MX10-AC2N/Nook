# 🔒 Rapport Sécurité — Nook 2026-04-09

## Score : 78/100

## Vulnérabilités (17 total)

### Critique (1)
- **S1** Secret TURN hardcodé dans frontend JS → déplacer côté serveur

### Haute (4)
- **S2** Pas de headers sécurité (CSP, HSTS, X-Frame-Options)
- **S3** User E2E avec mot de passe hardcodé dans binaire prod
- **S4** Routes webrtc hors middleware auth
- **S5** Secrets faibles dans .env.example

### Moyenne (5)
- **S6** Pas de validation inscription
- **S7** Pas de CSRF protection
- **S8** Pas de rate limiting par utilisateur
- **S9** Upload path traversal potentiel
- **S10** Config TURN commité dans le repo

### Positifs
- ✅ 100% SQLx paramétré (zero SQL injection)
- ✅ DOMPurify strict sur {@html}
- ✅ Arg2 password hashing
- ✅ WebSocket auth avant upgrade
- ✅ Rate limiting per-IP
- ✅ Upload magic bytes validation
