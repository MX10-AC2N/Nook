# 🧠 Nook Context — Hermes Extended Memory

> Mis à jour : 2026-06-13 | Commit stable : `c3c15f1e`

## 📊 État Actuel
| Composant | Statut | Détail |
|-----------|--------|--------|
| Backend CI | ⚠️ PENDING | musl multi-arch - Clippy warnings à fixer |
| Frontend CI | ⚠️ PENDING | package-lock drift / npm ci issues |
| Turn CI | ✅ GREEN | |
| Docker CI | ⚠️ PENDING | Backend build requis avant |
| E2EE nouveaux messages | ✅ OK | `encrypted_keys` non vide, try/catch par destinataire |
| E2EE anciens messages | ⚠️ STRUCTUREL | Indéchiffrables après rotation clé X25519 |

## 🔑 Informations Critiques
- Serveur test : http://192.168.1.192:6300 (hermes-bot / Hermes2026!)
- HTTPS local (nginx) : https://192.168.1.192:6443
- GITHUB_TOKEN PAT : /tmp/.git_token (93 chars)
- Ordre CI : Frontend → Backend → Turn → Docker
- No scheduled workflows (free GitHub account)
- Port 8080 sur 192.168.1.192 = scanservjs, pas Nook

## 🛠️ Stack Technique & Pièges
### Backend (Rust)
- Axum 0.8 : `{param}` pas `:param`, `Utf8Bytes` pas `String`
- SQLx 0.8.6 : requêtes inline, pas d'ORM
- rand 0.9 : `rng()` pas `thread_rng()`, `distr::` pas `distributions::`
- E2EE : X25519 32-bytes keypair → base64 = 44 chars; `crypto_box_seal` pour chiffrement destinataire
- `concurrent!` dans handlers Axum → utilisez `join!()` pas `spawn` isolé

### Frontend (Svelte 5)
- `$state`, `$derived.by`, `$props()` — pas de classique Svelte 3/4
- `<form onsubmit>` NE FONCTIONNE PAS → `<button type="button" onclick>` workaround
- `crypto_box_seal` → `uint8Array` 32-bytes clé + 32-bytes pubkey + 24-bytes nonce
- sodium-wrappers : `from_base64`, `to_base64`
- E2EE : `encrypted_keys` Record<string,string> par destinataire → try/catch échec individuel

### CI/CD (Docker musl)
- `musl-unknown-linux-musl` target sur runners natifs (pas dans Docker Alpine)
- `CGO_ENABLED=0` pour build release
- `sqlite-libs` (Alpine 3.20) pas `libsqlite3`
- Turn : protoc builder image `+ protoc install`
- Order :触发 Frontend → wait complete → Backend → wait → Turn → wait → Docker

## 🐛 Bugs Connus et Historique
| ID | Description | Fix Commit | Date |
|----|-------------|------------|------|
| BUG-001 | Compilation backend admin.rs parenthèses | 327b08e6 | |
| BUG-002 | E2EE refresh — pas de déchiffrement auto F5 | 0219c73e | |
| BUG-003 | P2P file transfer sécurité groupes | e9b17418 | |
| BUG-004 | Axum 0.8 panic events.rs:316 | 2568c5ef | 2026-06-13 |
| BUG-005 | E2EE clé publique désynchronisée registerPublicKeyOnServer fire-and-forget | 36eefe5c | 2026-05-15 |
| BUG-006 | E2EE encryptForRecipients try/catch par destinataire | f0a8c8d1 | 2026-05-16 |
| BUG-007 | Frontend npm ci failure | package-lock regen | |
| BUG-008 | Backend build musl | 49f40a5d | |
| BUG-009 | Turn arm64 download manquant | 087eee5f | |
| CI-001 | Docker sqlite-libs Alpine 3.20 | 0ee77f90 | |
| E2EE-OLD | Anciens messages indéchiffrables après rotation clé X25519 | ⚠️ STRUCTUREL | N/A |
| FIX-010 | Calendrier événements: routes unifiées events.rs, format frontend↔DB | 2568c5ef | 2026-06-13 |
| FIX-011 | Sondages: DELETE /api/polls/{id} cascade + WS | 2568c5ef | 2026-06-13 |
| FIX-012 | Échecs: prune parties finished/abandoned >7j | 2568c5ef | 2026-06-13 |
| FIX-013 | Scroll paramètres: overflow:hidden global retiré app.css | c3c15f1e | 2026-06-13 |

## 📝 Commandes Utiles
### Dev local
```bash
cd /opt/data/Nook
npm ci --legacy-peer-deps && npm run build    # Frontend
cargo check                                    # Backend
NOOK_ENV=development cargo run                 # Backend local HTTP
```

### Redéploiement CasaOS (192.168.1.192)
```bash
ssh root@192.168.1.192   # (si SSH autorisé)
cd /opt/data/Nook && git pull
docker compose down -v --rmi all --remove-orphans
docker compose up -d --build
# Etat services
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

### Validation E2EE serveur test
```
1. Ouvrir http://192.168.1.192:6300
2. Login hermes-bot / Hermes2026!
3. Aller sur un groupe, envoyer message test
4. Console navigateur : vérifier decryptSessionKey logs + encryptForRecipients console.info
5. Attendre rechargement / re-connexion : nouveaux messages doivent se déchiffrer
```

### CI orchestré
```bash
cd /opt/data/Nook
# Check state d'abord
gh run list --limit 5

# Puis dans l'ordre (attendre chacun avant le suivant)
gh workflow run "2==> 🎨 Frontend Build & Artifact"
# wait 35s vert
gh workflow run "1==>🏗️ Backend Build & Artifact"
# wait 4-9min vert
gh workflow run "3==> Turn-Server Build and Artifact"
# wait 2-3min vert
gh workflow run "4==> 🐳 Docker Build & Push"
```

## 🔗 Liens Rapides
- CI : https://github.com/MX10-AC2N/Nook/actions
- Repo : https://github.com/MX10-AC2N/Nook
- Serveur test : http://192.168.1.192:6300
- SessionStorage keys : `nook_privkey`, `nook_pubkey`, `nook_crypto_key`, `nook_userid`
- E2EE rotation-clé docs : `frontend/src/lib/crypto.ts` + `e2ee.rs` + `db.rs ligne 456`