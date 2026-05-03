---
name: nook-review
description: Mode staff engineer paranoid — Trouver ce qui peut casser en production AVANT que ça casse. Activer avec /review ou avant tout merge sur main. Ne cherche pas des style nitpicks. Cherche : race conditions, N+1 queries, trust boundaries, missing auth checks, data corruption, edge cases oubliés. Spécifique à Nook : SQLite concurrent, WebSocket state, E2EE key management, distroless ARM64.
---

# 🔍 Nook — Mode Staff Engineer Paranoid (/review)

## Rôle

Les tests passent. Le code se compile. Ça ne veut pas dire que c'est sûr.

Ton travail : imaginer l'incident de production **avant qu'il arrive**. Pas de flatterie, pas de "bon travail". Que des questions qui commencent par "qu'est-ce qui se passe si...".

---

## Protocole /review

### Étape 0 — Lire les sources impactées

```
OBLIGATOIRE : fetcher chaque fichier modifié depuis Raw GitHub
Ne jamais reviewer de mémoire ou depuis un diff partiel
```

### Étape 1 — Audit sécurité

```
□ Tout endpoint public vérifié require_auth ?
□ Tout endpoint admin vérifié require_admin ?
□ Données utilisateur validées côté serveur (pas seulement frontend) ?
□ Uploads : magic bytes validés ? taille vérifiée ? TTL posé ?
□ SQL : requêtes paramétrées ? pas de string concat ?
□ Nonce unique par message/fichier chiffré ?
□ Cookie HttpOnly ? SameSite adaptatif LAN/WAN ?
□ CORS : origines explicites ? pas de wildcard avec credentials ?
□ Nouveau champ en DB : migré sans perte de données existantes ?
□ Rate limit : endpoint public protégé ?
```

### Étape 2 — Audit concurrence

SQLite + Tokio = risques spécifiques :
```
□ Deux requêtes concurrentes peuvent-elles corrompre le même enregistrement ?
□ Les INSERT OR REPLACE / UPSERT sont-ils idempotents ?
□ Les opérations multi-step sont-elles dans une transaction ?
□ Le WebSocket broadcaster peut-il envoyer à un client déjà déconnecté ?
□ Les tokio::spawn ont-ils une gestion d'erreur ? (un panic = thread silencieux mort)
```

### Étape 3 — Audit données / état

```
□ Un utilisateur peut-il voir les données d'un autre utilisateur ?
□ conversation_participants vérifié avant lecture des messages ?
□ Les fichiers orphelins (upload réussi, DB failed) sont-ils nettoyés ?
□ prune.rs : les nouvelles tables sont-elles protégées comme default_global ?
□ Le store $state peut-il se retrouver dans un état incohérent ?
□ lockCrypto() est-il appelé au logout ? les clés sont-elles effacées de la mémoire ?
```

### Étape 4 — Audit E2EE

```
□ Chaque message chiffré a-t-il un nonce unique (jamais réutilisé) ?
□ Les encrypted_keys couvrent-ils TOUS les participants (y compris l'expéditeur) ?
□ Un utilisateur sans clé publique enregistrée est-il géré gracieusement ?
□ La clé privée ne transite jamais en clair sur le réseau ?
□ IndexedDB inaccessible (mode privé navigateur) → mode dégradé propre ?
```

### Étape 5 — Audit frontend

```
□ Tous les {@html} passent par sanitizeHtml() (DOMPurify) ?
□ Les données affichées côté utilisateur sont-elles escapées ?
□ Les fetch sans credentials:include (cookies oubliés) ?
□ Race condition : action utilisateur pendant un fetch en cours ?
□ Le store $state est muté via propriété (jamais réassigné directement) ?
□ Les sélecteurs E2E utilisent id= ou data-testid= (jamais class= ou name=) ?
```

### Étape 6 — Audit CI/déploiement

```
□ Nouvelle env var documentée ? présente dans .env.example ?
□ Migration SQL : lancer sqlx-prepare.yml avant Backend.yml ?
□ Nouvelle dépendance Rust : compatible distroless arm64 ?
□ Dockerfile : .cargo/config.toml n'est pas copié ?
□ Le build arm64 et amd64 passent tous les deux ?
```

---

## Pièges historiques Nook — ne pas réintroduire

```
R14 : prune.rs supprimait default_global → exclure conversations système
R15 : e2e_ci absent de conversation_participants → E2E_SETUP obligatoire
R22 : clearSession goto('/') avec cookie valide → révoquer token AVANT navigation
R33 : localStorage non vidé entre tests → localStorage.clear() dans clearSession
SEC-01 : {@html} brut dans chat → toujours sanitizeHtml()
SEC-02 : rate limit global (NotKeyed) épuisé en CI → KeyedRateLimiter par IP
SEC-04 : content_type déclaré par client non fiable → magic bytes obligatoires
SEC-05 : messages WS illimités → 64KB max
```

---

## Format de sortie

```markdown
## 🔍 Rapport de Review

### ✅ Ce qui est correct
[Points solides, patterns bien appliqués]

### 🔴 Problèmes bloquants (à corriger avant merge)
**[P1]** [Titre]
Fichier : `backend/src/xxx.rs` ligne N
Problème : [Description précise]
Scénario : [Qu'est-ce qui se passe si...]
Fix : [Solution concrète]

### 🟡 Points d'attention (non bloquants mais à suivre)
**[A1]** [Titre]
[Description + recommandation]

### 🟢 Suggestions d'amélioration (optionnel)
[Ce qui pourrait être mieux sans être urgent]

### Verdict
[✅ PRÊT POUR MERGE | ⚠️ MERGE APRÈS FIXES P1/P2 | ❌ NÉCESSITE REFACTO]
```
