# 🔍 Debug, CI & Glossaire — Nook

> Patterns de debug validés sur 39 sessions. Ce fichier évite de répéter
> les mêmes diagnostics. Lire avant d'analyser un rapport CI ou des logs.

---

## 🧪 Debug CI — Lecture des rapports

### TEST_REPORT.md — Interpréter rapidement
```
✓ = test passé
✘ = test échoué → chercher le message d'erreur dessous
Timeout = attente d'un élément UI → probablement waitFor manquant
expect received "" = selector présent mais vide → mauvais timing
page.goto() résolu ≠ onMount() terminé → toujours waitFor('#username')
```

### BACKEND-BUILD-REPORT-*.md — Signaux d'alerte
```
ERREUR cargo check    → problème de compilation Rust
ERREUR cargo clippy   → warning promu en erreur (-D warnings)
ERREUR sqlx           → queries.json désynchronisé → lancer sqlx-prepare.yml
cannot find rand::rng  → utiliser rand::rng() (rand 0.9), pas thread_rng()
type mismatch OsRng   → utiliser rand_core::OsRng (pas rand::rngs::OsRng)
```

### DOCKER-BUILD-REPORT.md — Signaux d'alerte
```
permission denied 65532  → init container chown manquant
no such file or directory → STATIC_FILES_DIR ou UPLOADS_DIR mal configuré
linker error aarch64     → .cargo/config.toml copié dans Docker (interdit)
```

---

## 🐛 Patterns de bugs récurrents

### Pattern 1 — Test E2E échoue avec "element not found"
```
Cause probable : fill() appelé avant que onMount() soit terminé
Fix             : waitFor('#username', { state: 'visible', timeout: 20000 })
Où              : loginAs() helper dans e2e.spec.ts
```

### Pattern 2 — Cookie non envoyé (401 inattendu)
```
Cause probable : apiFetch sans credentials:'include'
                 ou CORS bloque les credentials
Fix             : vérifier apiFetch() | vérifier ALLOWED_ORIGINS dans .env
                 LAN: SameSite=Lax | WAN: SameSite=None;Secure + HTTPS
```

### Pattern 3 — Build Rust échoue sur arm64 mais pas amd64
```
Cause probable : .cargo/config.toml présent dans le container Docker
Fix             : ne jamais COPY .cargo/ dans Dockerfile
                 (linkers cross uniquement sur les runners GitHub Actions)
```

### Pattern 4 — sqlx compile en local mais échoue en CI
```
Cause probable : .sqlx/queries.json désynchronisé après nouvelle macro sqlx!
Fix             : lancer sqlx-prepare.yml (ou cargo sqlx prepare en local)
Règle           : pas de macros sqlx! si queries.json non régénéré → utiliser
                  sqlx::query_as::<_, T>("SQL") sans macro
```

### Pattern 5 — clearSession ne fonctionne pas (tests souillés)
```
Cause probable : goto('/') avant révocation → authStore.init() avec cookie encore valide
Fix             : page.request.post('/api/auth/logout') AVANT tout goto()
Pattern validé (session 22) :
  await page.request.post('http://localhost:6300/api/auth/logout');
  await page.context().clearCookies();
```

### Pattern 6 — CORS panic au démarrage du backend
```
Message : "cannot use wildcard with credentials"
Cause   : allow_any_origin() + allow_credentials(true) interdit
Fix     : ALLOWED_ORIGINS= liste explicite dans .env
```

### Pattern 8 — winner_id chess FK violation silencieuse
```
Cause   : chess_games.winner_id est une FK vers users(id)
          Stocker "ai" → pas un user valide → UPDATE .ok() absorbe l'erreur
Fix     : winner_id = None pour les parties IA, Some(user_id) pour humain
```

### Pattern 9 — #[derive] séparé de son struct par une fn
```
Cause   : fn default_true() insérée entre #[derive(...)] et pub struct User { }
          Le derive s'applique à la fonction → E0774 + cascade FromRow
Fix     : toujours placer les fn helpers AVANT leur premier #[derive]
```

### Pattern 10 — mot réservé Rust utilisé comme binding
```
Cause   : priv, pub, crate, super, type... sont des mots réservés
          Utilisés dans un pattern match → erreur de compilation
Fix     : renommer en private_key, public_key, etc.
```

### Pattern 7 — prune.rs supprime default_global
```
Cause   : conversation sans participants → supprimée par prune
Fix     : exclure conversations système du nettoyage
          admin + e2e_ci doivent être dans conversation_participants de default_global
```

---

## 🔄 Flux de debug recommandé

### Quand un test E2E échoue en CI

```
1. Lire TEST_REPORT.md → identifier le test et le message d'erreur exact
2. Vérifier si c'est un timeout → suspect : waitFor manquant ou timing
3. Vérifier si c'est un selector → inspecter les id= et data-testid= dans le composant
4. Lancer e2e-targeted.yml avec debug_traces:true pour ce test précis
5. Télécharger les traces → screenshots Playwright pour voir l'état réel de l'UI
```

### Quand le build backend échoue

```
1. Lire BACKEND-BUILD-REPORT-*.md → chercher "error[" dans les logs
2. Si "sqlx" dans l'erreur → lancer sqlx-prepare.yml
3. Si "rand" dans l'erreur → vérifier rand::rng() (pas thread_rng()) et rand_core::OsRng
4. Si "linker" dans l'erreur → vérifier que .cargo/config.toml n'est pas dans Dockerfile
5. Si "clippy" → warning promu en erreur, corriger le warning
```

### Quand le Docker build échoue

```
1. Lire DOCKER-BUILD-REPORT.md
2. Si "permission 65532" → init container chown manquant
3. Si "artifact not found" (dawidd6) → vérifier que Backend.yml + Frontend.yml ont tourné
4. Vérifier que les artifacts ne sont pas expirés (retention 7 jours)
```

---

## 📋 Checklist pré-commit (rappel rapide)

```
□ rand::rng() (pas thread_rng())
□ rand_core::OsRng (pas rand::rngs::OsRng)
□ Routes Axum 0.8 : {param} (pas :param)
□ Message::Text → nécessite .into() pour Utf8Bytes
□ sqlx sans macros si queries.json pas régénéré
□ .svelte/.ts → livrer en .txt
□ Sélecteurs E2E : id= ou data-testid= (pas name=, pas class=)
□ Cookie CORS : ALLOWED_ORIGINS listées, pas de wildcard
□ $state Svelte 5 : mutation via propriété, pas réassignation
□ BUGS.md consulté → ne pas réintroduire les bugs résolus
□ #[derive(...)] adjacent au struct — jamais de fn libre entre les deux
□ chess_games.winner_id → None pour IA, jamais "ai" (FK users)
□ priv / pub / crate / super → mots réservés Rust, ne pas utiliser comme identifiants
□ apiFetch n'existe pas dans api.ts → utiliser fetch() avec credentials:'include'
□ POST /polls → { "poll": { id } }, pas { id } au niveau racine
```

---

## 🗺️ Pipeline CI — Ordre requis

```
1. sqlx-prepare.yml     (si migration SQL ajoutée)
2. Backend.yml          → artifacts: nook-backend-{amd64,arm64}  (7j)
3. Frontend.yml         → artifact: nook-frontend                 (7j)
4. test-nook.yml        → compile depuis sources (Dockerfile), E2E Playwright
5. Docker.yml           → assemble artifacts → image GHCR multi-arch
   └── (dawidd6/action-download-artifact@v6 pour cross-workflow)
6. ghcr-cleanup.yml     → auto après Docker.yml
7. Release.yml          → bump VERSION + Cargo.toml + package.json + tag git
```

Workflows utilitaires (indépendants) :
- `bundle-analysis.yml` → rapport tailles chunks frontend
- `e2e-targeted.yml` → debug un seul test
- `generate-android-instruction.yml` → met à jour ANDROID-INSTRUCTION.md
- `npm-audit-report.yml` → sécurité dépendances
- `npm-update-deps.yml` → mise à jour deps npm
- `auto-svelte5-migration.yml` → vérification Svelte 5 compliance

---

## 📚 Glossaire Nook

| Terme | Définition |
|-------|-----------|
| `default_global` | Conversation groupe par défaut créée au premier lancement, tous les membres y sont ajoutés |
| `e2e_ci` | Utilisateur de test créé uniquement si `E2E_SETUP=1` (CI), mdp `E2eTest123!` |
| `approved` | Champ users : `0`=en attente d'approbation admin, `1`=peut se connecter |
| `needs_password_change` | `1` = redirect forcé vers `/change-password` au prochain login |
| `conversation_participants` | Nom **exact** de la table (pas `members`, pas `members_of`) |
| `E2E_SETUP` | Variable env CI uniquement — crée `e2e_ci` + ajoute dans `default_global` |
| `distroless` | Image Docker sans shell ni outils — binaire Rust + libs système uniquement. Pas de `docker exec nook sh` possible |
| `init container` | `alpine:3` utilisé pour `chown -R 65532:65532` les volumes avant le container distroless |
| `queries.json` | Cache SQLx offline (`.sqlx/queries.json`) — doit être régénéré après toute nouvelle macro `sqlx!` |
| `DT-01` | Dette technique : libsodium-wrappers 938 kB charge de façon synchrone → retard layout |
| `DT-02` | Chess temps réel absent — adversaire voit les coups seulement au refresh |
| `SameSite=Lax` | Cookie en HTTP/LAN (192.168.x.x) |
| `SameSite=None;Secure` | Cookie en HTTPS/WAN (Nginx Proxy Manager) — déclenché par `X-Forwarded-Proto: https` |
| `auth_token` | Nom du cookie d'auth : `<userId>:<token>`, HttpOnly, Max-Age=86400 |
| `require_auth` | Middleware Axum — vérifie cookie + `approved=1` + token en DB |
| `require_admin` | Middleware Axum — vérifie `role='admin'` en plus de `require_auth` |
| `XChaCha20-Poly1305` | Algo chiffrement fichiers (nonce 24B, key 32B) — compatible libsodium |
| `Argon2id` | Algo hash mots de passe (via `rand_core::OsRng`) |
| `FEN` | Forsyth-Edwards Notation — format standard représentation position échecs |
| `SAN` | Standard Algebraic Notation — format moves échecs (ex: `e4`, `Nf3`, `O-O`) |
| `Zimaboard 832` | Homeserver de production — ARM64, 8Go RAM, SSD 32Go |
| `GHCR` | GitHub Container Registry — stockage images Docker du projet |
| `dawidd6` | Action `dawidd6/action-download-artifact@v6` — seule qui supporte les artifacts cross-workflow (nécessaire pour Docker.yml) |
| `VAPID` | Voluntary Application Server Identification — protocole JWT ES256 pour authentifier les push notifications (RFC 8292) |
| `push_subscriptions` | Table stockant les abonnements push par device (endpoint URL + clés p256dh/auth) |
| `unlockCrypto()` | Fonction `cryptoStore.svelte.ts` — déchiffre ou génère les clés E2EE après login |
| `cryptoStore.ready` | `true` si les clés X25519 sont chargées en mémoire → messages chiffrés |
| `DT-01` | libsodium-wrappers 938 kB — cosmétique (fire-and-forget depuis S37, non bloquant) |
| `DT-02` | Chess temps réel — **résolu S39** (WS broadcast serveur + refreshGame client) |


## 🌐 SFU — Debug

### Vérifier le SFU est actif
```bash
# Logs backend
docker logs nook 2>&1 | grep -i sfu

# Logs SFU join/leave
docker logs nook 2>&1 | grep "SFU join\|SFU remove"

# Logs track forwarding
docker logs nook 2>&1 | grep "SFU track received\|SFU track relayed"

# Logs ICE state
docker logs nook 2>&1 | grep "SFU ICE"

# Logs PLI forwarding
docker logs nook 2>&1 | grep "SFU forwarding PLI"
```

### Debug callStore SFU state (browser console)
```javascript
// Depuis la console du navigateur (dev tools)
// Le callStore est accessible via l'instance Svelte
```

### Scenarios de debug

| Symptôme | Cause probable | Vérification |
|----------|---------------|--------------|
| "sfu_join" jamais reçu par backend | WS pas connecté ou pas auth | Vérifier ws.readyState === 1 |
| Answer SDP vide | create_answer() avant set_remote() | Vérifier l'ordre dans handle_join |
| Track non relayée | added_sources contient déjà la clé | Logs: "SFU track already added, skip" |
| Video freeze sur un peer | PLI pas forwardé | Logs: "SFU forwarding PLI" absent |
| Negotiation échoue | signaling_state pas Stable | Logs: "SFU negotiation deferred" |

## 📖 Glossaire SFU

| Terme | Définition |
|-------|-----------|
| **SFU** | Selective Forwarding Unit — serveur qui relaye les streams media entre participants |
| **Mesh P2P** | Chaque pair se connecte à tous les autres (N×(N-1) connexions) |
| **MediaRelay** | Mécanisme rustrtc pour dupliquer une track vers plusieurs subscribers |
| **PLI** | Picture Loss Indication — demande RTCP pour forcer une keyframe |
| **FIR** | Full Intra Request — comme PLI mais plus fort |
| **Negotiation** | Échange SDP offer/answer pour ajouter/retirer des tracks |
| **added_sources** | HashSet pour éviter d'ajouter deux fois la même track au même peer |
| **SfuJoinResponse** | Réponse du backend: answer SDP + liste peers + offre renegotiation pending |
| **drain_pending_offer** | Méthode pour récupérer et vider l'offer de renegotiation pending |
