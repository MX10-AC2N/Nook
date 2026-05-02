# 🔐 Règles — Gestion des Secrets

## ⚠️ RÈGLE FONDAMENTALE

**AUCUN secret ne doit être en dur (hardcoded) dans le code, les configs ou les variables d'environnement commitées.**

---

## ✅ PRATIQUES CORRECTES

### 1. Variables d'environnement
```bash
# .env (NE PAS COMMITTRE !)
TURN_SECRET=MonSecretSuperSecureGenereAleatoirement
ADMIN_INITIAL_PASSWORD=UnAutreSecretChangeLePremierLogin
```

### 2. Valeurs par défaut sécurisées
```yaml
# docker-compose.yml
environment:
  - TURN_SECRET=${TURN_SECRET:?TURN_SECRET must be set}
  - ADMIN_INITIAL_PASSWORD=${ADMIN_INITIAL_PASSWORD:-}
```

**Explication** :
- `${TURN_SECRET:?...}` → Erreur si non défini (bloque le démarrage)
- `${ADMIN_INITIAL_PASSWORD:-}` → Vide par défaut (génère aléatoire)

### 3. Templates avec placeholders
```toml
# services/turn-rs/turnserver.conf.template
[server]
secret = "${TURN_SECRET}"
```

Puis remplacement via entrypoint :
```bash
# services/turn-rs/docker-entrypoint.sh
sed -i "s|\\${TURN_SECRET}|$TURN_SECRET|g" "$CONFIG_FILE"
```

### 4. Génération de secrets
```bash
# TURN_SECRET (32 octets, base64)
openssl rand -base64 32

# ADMIN_INITIAL_PASSWORD (16 caractères alphanumériques)
openssl rand -base64 16

# VAPID keys
npx web-push generate-vapid-keys
```

---

## 🚫 EXEMPLES DE SECRETS À PROTÉGER

| Type | Où | Comment le protéger |
|------|---|----------------------|
| **TURN_SECRET** | `docker-compose.yml`, `turnserver.conf` | Variable obligatoire, erreur si absent |
| **ADMIN_INITIAL_PASSWORD** | `docker-compose.yml`, `main.rs` | Générer ou laisser vide (auto-généré) |
| **VAPID_PRIVATE_KEY** | `docker-compose.yml` | Généré via `npx web-push` |
| **GIPHY_API_KEY** | `docker-compose.yml` | Optionnel, laisser vide si pas utilisé |
| **Database URL** | `docker-compose.yml` | SQLite local, pas de secret sensible |
| **JWT secrets** | Backend code | Dérivés du mot de passe (pas de secret séparé) |

---

## ❌ FICHIERS À NE JAMAIS COMMITTRE

```bash
# Dans .gitignore ou .dockerignore
.env
*.log
*.db
*.sqlite
*.pem
*.key
*.crt
secrets/
credentials/
```

---

## 🔍 VÉRIFICATIONS PRÉ-COMMIT

```bash
# 1. Vérifier les secrets en dur
git diff --cached | grep -i "secret\|password\|token\|key" | grep -v "TURN_SECRET:\?\|ADMIN_INITIAL_PASSWORD:-"

# 2. Vérifier .env.example
grep -E "change_this|secret_2026|password_here" .env.example
# → Doit être vide ou avec instructions openssl

# 3. Vérifier les logs
git log --oneline --all | grep -i "password\|secret"
# → Aucun mot de passe ne doit apparaître
```

---

## 📝 EXEMPLE DE .env.example SÉCURISÉ

```bash
# ⚠️  N'OUBLIEZ PAS : 
# 1. cp .env.example .env
# 2. Générez les secrets :
#      openssl rand -base64 32  → pour TURN_SECRET
#      openssl rand -base64 16  → pour ADMIN_INITIAL_PASSWORD
# 3. Ne jamais commiter .env !

TURN_SECRET=change_this_turn_secret_here
ADMIN_INITIAL_PASSWORD=change_this_admin_password_here
```

---

## 🚨 POINTS DE VIGILANCE

### ❌ Dans le code Rust
```rust
// ❌ JAMAIS
let secret = "mon_secret_en_dur";

// ✅ TOUJOURS
let secret = std::env::var("TURN_SECRET")
    .expect("TURN_SECRET must be set");
```

### ❌ Dans les configs
```yaml
# ❌ JAMAIS
environment:
  - TURN_SECRET=mon_secret_123

# ✅ TOUJOURS
environment:
  - TURN_SECRET=${TURN_SECRET:?must be set}
```

### ❌ Dans les logs
```rust
// ❌ JAMAIS
eprintln!("Admin créé avec mot de passe : {}", password);

// ✅ TOUJOURS
eprintln!("Admin créé - changer le mot de passe à la première connexion");
```

---

## 🧪 TESTS DE SÉCURITÉ

```bash
# 1. Test : TURN_SECRET manquant
docker compose up -d
docker compose logs nook-turn | grep "ERROR.*TURN_SECRET"

# 2. Test : Permissions des fichiers
docker exec nook ls -la /app/data | grep "drwxr-x---"

# 3. Test : Pas de secrets dans les logs
docker logs nook 2>&1 | grep -i "password\|secret" | grep -v "change_password"
```

---

## 📋 RÉFÉRENCE RAPIDE

| Action | Commande |
|--------|-----------|
| Générer TURN secret | `openssl rand -base64 32` |
| Générer admin password | `openssl rand -base64 16` |
| Vérifier pas de secrets | `git diff \| grep -i secret` |
| Voir les variables | `docker compose config \| grep SECRET` |
| Test entrypoint | `docker exec nook-turn cat /etc/turn-server/config.toml \| grep secret` |
