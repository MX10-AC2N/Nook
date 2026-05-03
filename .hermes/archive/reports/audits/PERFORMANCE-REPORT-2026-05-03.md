# ⚡ Rapport Performance — Nook 2026-05-03

## Score : 80/100

> **Évolution** : 81/100 (2026-04-09) → 82/100 (Session 50) → **80/100 (actuel)**
> 
> Légère baisse due à l'augmentation de la taille du bundle principal et aux nouveaux warnings CSS/a11y.

---

## Résumé de l'audit

| Catégorie | Score | Max | % | Évolution |
|-----------|-------|-----|---|-----------|
| **Bundle Size** | 18 | 25 | 72% | ⚠️ Main chunk 920kB |
| **Lazy Loading** | 15 | 15 | 100% | ✅ Modulepreload OK |
| **Images** | 8 | 15 | 53% | ⚠️ PNG non optimisés |
| **Backend SQL** | 20 | 20 | 100% | ✅ Indexes complets |
| **WebSocket** | 10 | 10 | 100% | ✅ Tout configuré |
| **Cache** | 7 | 10 | 70% | ✅ Headers présents |
| **Code Quality** | 2 | 5 | 40% | ⚠️ 50+ warnings CSS/a11y |
| **TOTAL** | **80** | **100** | **80%** | |

---

## Analyse détaillée

### 1. 📦 Bundle Size (18/25) — ⚠️ Régressé

#### Tailles des chunks (build production)

| Fichier | Taille (kB) | Gzip (kB) | Brotli (kB) | Commentaire |
|---------|-------------|------------|--------------|-------------|
| `HEavZsIZ.js` | **920** | 299.5 | 227.4 | ⚠️ Vendor/Svelte (TROP GROS) |
| `aIWNwWfY.js` | 200.5 | 67.7 | 57.1 | libsodium (dynamic import) |
| `Bz8TaB3O.js` | 75.4 | 28.6 | 25.3 | Chess components |
| `nodes/8.js` (chat) | 75.7 | 20.1 | 15.7 | Chat page |
| `nodes/17.js` | 19.4 | 6.7 | 5.8 | Autre page |

#### ✅ Positifs
- Code splitting configuré dans `vite.config.js` (manualChunks)
- libsodium isolé et chargé dynamiquement (`preloadSodium()`)
- Compression gzip + brotli activée (`vite-plugin-compression`)
- modulepreload présent dans `index.html`

#### ⚠️ Problèmes
- **P1 (Nouveau)** : Chunk principal `HEavZsIZ.js` fait **920 kB** (non compressé)
  - Le seuil d'alerte Vite est 600 kB
  - Même compressé (299 kB gzip), c'est lourd pour un chargement initial
- **Cause possible** : Le chunk contient Svelte + toutes les dépendances non divisées
- **Recommandation** : Investiguer pourquoi le chunk vendor est si volumineux

#### Code splitting (vite.config.js)
```javascript
manualChunks(id) {
  if (id.includes('libsodium')) return 'libsodium';
  if (id.includes('chess.js') || id.includes('chessground')) return 'chess';
  if (id.includes('chart.js')) return 'chart';
  if (id.includes('node_modules/svelte')) return 'svelte';
  if (id.includes('node_modules')) return 'vendor';
}
```
⚠️ Le chunk 'vendor' reste trop gros (920 kB). Vérifier ce qu'il contient.

---

### 2. 🚀 Lazy Loading (15/15) — ✅ Excellent

#### ✅ Réalisé
- **libsodium** : Chargement dynamique via `preloadSodium()` dans `sodium.svelte.js`
- **modulepreload** : 5 fichiers critiques préchargés dans `index.html`
  ```html
  <link href="/_app/immutable/entry/start.DRDXivSR.js" rel="modulepreload">
  <link href="/_app/immutable/chunks/Bz8TaB3O.js" rel="modulepreload">
  <link href="/_app/immutable/chunks/bp-A89Kb.js" rel="modulepreload">
  <link href="/_app/immutable/entry/app.GTkIGJ3b.js" rel="modulepreload">
  <link href="/_app/immutable/chunks/PPVm8Dsz.js" rel="modulepreload">
  ```
- **SvelteKit preload** : `data-sveltekit-preload-data="hover"` activé
- **Dynamic imports** : `backup.ts`, `crypto.ts`, `storage.ts` utilisent `import()`

---

### 3. 🖼️ Images (8/15) — ⚠️ À améliorer

#### État actuel (dossier `static/`)
| Fichier | Format | Taille | Optimisé ? |
|---------|--------|--------|-------------|
| `logo-512.png` | PNG | 12 kB | ❌ Non |
| `logo-192.png` | PNG | 4 kB | ❌ Non |
| `favicon.png` | PNG | 899 B | ❌ Non |
| `icon-*.png` | PNG | 1.6-4 kB | ❌ Non |
| `logo-animated.svg` | SVG | 5.5 kB | ✅ OK |

#### ❌ Problèmes
- **P3 (Rapport 2026-04-09)** : Toujours pas d'images WebP/AVIF
- Pas de version responsive des images
- Pas d'attributs `loading="lazy"` sur les images (sauf audio/video)
- Pas de `<picture>` element pour le format moderne

#### ✅ Recommandations
1. Convertir `logo-512.png` en WebP (économie ~30-40%)
2. Ajouter des versions AVIF pour les navigateurs modernes
3. Utiliser `<picture>` avec fallbacks :
   ```html
   <picture>
     <source srcset="logo-512.avif" type="image/avif">
     <source srcset="logo-512.webp" type="image/webp">
     <img src="logo-512.png" alt="Nook Logo" loading="lazy">
   </picture>
   ```

---

### 4. 🗄️ Backend SQL (20/20) — ✅ Excellent

#### ✅ Indexes présents (vérifiés dans `/migrations/`)
- `idx_users_username`, `idx_users_email`, `idx_users_token`
- `idx_conversations_updated_at`
- `idx_messages_conversation`, `idx_messages_file_id`
- `idx_uploads_uploaded_at`, `idx_uploads_conversation`
- `idx_chess_status`, `idx_chess_created_by`, `idx_chess_updated_at`
- `idx_events_date`, `idx_events_created_by`
- `idx_polls_created_at`, `idx_poll_votes_poll`, etc.
- `idx_reactions_message`
- `idx_missed_calls_conversation`, `idx_missed_calls_callee`

#### ✅ Configuration SQLite
- **WAL mode** activé (`journal_mode(SqliteJournalMode::Wal)`)
- **Synchronous NORMAL** (`synchronous(SqliteSynchronous::Normal)`)
- `create_if_missing(true)` pour la première initialisation

#### ✅ Pas de N+1 détecté
- Les requêtes JOIN sont bien structurées (ex: `MessageWithSender` avec COALESCE)
- Utilisation de `sqlx::query_as!` pour le typage fort

---

### 5. 🔌 WebSocket (10/10) — ✅ Excellent

#### ✅ Sécurité et performance
- **Chiffrement** : XChaCha20-Poly1305 pour les fichiers et signaling
- **Limite de taille** : 64 KB par message WebSocket (SEC-05)
- **Authentification** : Cookie `auth_token` vérifié dès la connexion WS
- **Nettoyage** : Fichiers expirés après 48h, tâche de fond toutes les heures
- **Gestion des erreurs** : `tungstenite::Error` gérées correctement

#### Code (`webrtc.rs`)
```rust
// Limite 64 KB sur les messages WS
const MAX_WS_MESSAGE_SIZE: usize = 64 * 1024;

// Chiffrement XChaCha20-Poly1305 (compatible libsodium)
fn crypto_secretbox_easy(message: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8>
```

---

### 6. 🌐 Cache HTTP (7/10) — ✅ Bon

#### ✅ Headers présents (`main.rs`)
```rust
// Cache-Control: public, max-age=3600 (1h)
SetResponseHeaderLayer::overriding(
    axum::http::header::CACHE_CONTROL,
    axum::http::HeaderValue::from_static("public, max-age=3600"),
)
```

#### ✅ Compression activée
- `tower-http` avec `CompressionLayer` (gzip + brotli)
- `vite-plugin-compression` côté frontend

#### ⚠️ Amélioration possible
- **Problème** : `max-age=3600` pour TOUS les assets, y compris les fichiers hashés
- **Recommandation** : Utiliser `max-age=31536000` (1 an) pour les fichiers avec hash dans le nom
- Les fichiers non-hashés (HTML) devraient avoir `max-age=0, must-revalidate`

---

### 7. 🧹 Code Quality (2/5) — ⚠️ Trop de warnings

#### ❌ Warnings CSS (50+ durant le build)
Le build génère de nombreux warnings `css_unused_selector` :
- `chat/+page.svelte` : ~30 sélecteurs inutilisés (.message.mine, .message-sender, etc.)
- `polls/+page.svelte` : ~10 sélecteurs inutilisés
- `settings/+page.svelte` : ~5 sélecteurs inutilisés
- `login/+page.svelte` : `h1` inutilisé

#### ❌ Warnings a11y
- `chat/+page.svelte` : `onclick` sans `onkeydown`, `<div>` sans `role`
- `settings/+page.svelte` : `<label>` sans contrôle associé (3 occurrences)
- `NotificationToast.svelte` : éléments non-interactifs avec listeners

#### ✅ Action prise
Ces warnings sont supprimés de la sortie via `onwarn()` dans `vite.config.js`, mais le code reste à nettoyer.

---

## Comparaison avec le rapport précédent (2026-04-09)

| Problème | Rapport 2026-04-09 | État actuel | Statut |
|----------|-------------------|------------|--------|
| **P1** Pas de code splitting | ❌ Monolithique | ⚠️ Partiel (chunks mais vendor 920kB) | 🔧 En cours |
| **P2** Pas de cache HTTP | ❌ Absent | ✅ Présent (max-age=3600) | ✅ Résolu |
| **P3** Images non optimisées | ❌ PNG uniquement | ❌ Toujours PNG | ⚠️ Toujours ouvert |
| **P4** Pas de preload | ❌ Absent | ✅ modulepreload présent | ✅ Résolu |

### Nouveaux problèmes détectés
- **N1** : Chunk vendor trop volumineux (920 kB)
- **N2** : 50+ warnings CSS/a11y non résolus
- **N3** : Cache-Control non optimisé pour les fichiers hashés

---

## Recommandations prioritaires

### Haute priorité (à faire immédiatement)
1. **Investiguer le chunk vendor 920kB** :
   ```bash
   npx vite-bundle-visualizer  # ou analyser manuellement
   ```
   - Identifier ce qui se trouve dans `HEavZsIZ.js`
   - Séparer les dépendances restantes

2. **Optimiser les images** :
   - Convertir `logo-512.png` en WebP/AVIF
   - Économie estimée : ~4-5 kB par page

### Moyenne priorité
3. **Nettoyer les warnings CSS** :
   - Supprimer les sélecteurs inutilisés dans `chat/+page.svelte`
   - Corriger les labels sans contrôle dans `settings/+page.svelte`

4. **Optimiser le Cache-Control** :
   ```rust
   // Pour les fichiers hashés (CSS/JS) : 1 an
   "public, max-age=31536000, immutable"
   // Pour les fichiers non-hashés (HTML) : pas de cache
   "no-cache, no-store, must-revalidate"
   ```

### Basse priorité
5. **Ajouter des tests de performance** :
   - Lighthouse CI dans GitHub Actions
   - Monitoring des temps de réponse backend (Prometheus ?)

---

## Conclusion

Le projet Nook a fait des progrès significatifs depuis le rapport du 9 avril :
- ✅ Code splitting maintenant configuré
- ✅ Cache HTTP ajouté
- ✅ modulepreload pour les ressources critiques

Cependant, **la taille du bundle principal (920 kB) et les warnings CSS/a11y empêchent un score supérieur**.

**Score final : 80/100** (baisse de 2 points par rapport à la session 50 à cause de la régression bundle)

### Prochaines étapes
1. Analyser le contenu du chunk `HEavZsIZ.js` (920 kB)
2. Optimiser les images (WebP/AVIF)
3. Nettoyer les warnings CSS et a11y
4. Re-auditer dans 2 semaines

---
*Audit réalisé le 2026-05-03 par Hermes Agent (nook-performance-specialist)*
