# Tranchage des réserves d'ADR-016 — rotation de clé de groupe & purge des versions

**Date** : 2026-10-22
**Décideur** : architect (carte kanban `t_08632b95`)
**Objet** : les **2 réserves** de la revue `security-auditor` (`ACCEPTÉ AVEC RÉSERVES MAJEURES`, carte `t_0fc7cc3f`) sur `docs/adr/016-e2ee-group-hybrid.md`, + **1 réserve découverte** à l'occasion de cette carte.
**Méthode** : **mesure de code**, pas de supposition. Relevé sur `develop` (HEAD `eb733cf2`) — chaque affirmation est adossée à `fichier:ligne`.
**Périmètre** : **docs uniquement**. Le code est hors périmètre ; les écarts qui demandent du code sont routés en cartes `@coder` (§4).

---

## 0. Les 3 réserves, en une ligne

| # | Réserve | Mesure | Décision |
|---|---------|--------|----------|
| **R1** | Rotation time-based 7j non implémentée | `rotateGroupKey()` = dead code, **zéro** appelant ; aucun job backend ; aucun champ de config | **ASSUMÉE** (rotation **manuelle**) — + écart de contrôle de rôle acté |
| **R2** | Aucune purge des anciennes versions de clé | `get_my_group_key` accepte **n'importe quelle** `version` ; `prune.rs` ne touche jamais les clés de groupe | **ASSUMÉE** (conserve **illimitée**, **aucun** expiry) — la revue la décrivait à tort comme bornée côté client |
| **R3** | *(découverte)* `add_member_key` ne pose pas `distribution_status` | `INSERT` sans la colonne → naît `delivered` (défaut migration 022) → `claim_key` ne la servira **jamais** | **ASSUMÉE** + carte `@coder` (même famille que R2) |

**Règle appliquée aux trois** : une réserve n'est levée que par une **preuve**. Ici la preuve est que le code n'implémente pas la politique spécifiée — donc on **assume** la politique réelle et on l'écrit, au lieu d'implémenter une politique que le code n'a jamais eue. Fabriquer l'implémentation de toutes pièces, c'est fabriquer la spécification.

---

## 1. R1 — Rotation automatique 7j : **ASSUMÉE** (rotation manuelle)

### Mesure

| Affirmation | Preuve |
|---|---|
| Aucun champ `group_key_rotation_days` dans la config | `backend/src/config.rs:5-23` — `Config` n'a **pas** ce champ · `grep -rn "rotation_days\|ROTATION_DAYS" backend/ frontend/` → **aucun résultat** |
| `rotateGroupKey()` est du dead code | déclaration `frontend/src/lib/e2ee.ts:189` · `grep -rn "rotateGroupKey" frontend/src/` → **1 seule occurrence** (la déclaration) = **zéro appelant** |
| Aucun job périodique backend ne déclenche la rotation | les seuls `tokio::spawn` périodiques sont `main.rs:443` (fichiers) et `main.rs:449` (`prune_old_data`, 24 h) |

### Décision

**Assumée** : la rotation de la clé de groupe est **manuelle**. Elle est déclenchée par un membre qui appelle `rotateGroupKey()` depuis l'UI ; il n'existe **aucun** déclenchement automatique.

### Écart de spécification acté (précision qui change le sens de la réserve)

RFC-016 **§5.4** (`docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md:179`) impose : *« Pour le groupe global, **seul l'admin** peut déclencher la rotation »*.

La mesure montre que le rôle **existe** mais n'est **pas appliqué** aux routes E2EE :

- le rôle est bien modélisé : `backend/migrations/001_initial.sql:16` (`role TEXT NOT NULL DEFAULT 'user'`) et appliqué ailleurs : `backend/src/admin.rs:69,110,152,197,230` (`if user.role != "admin"` → 403) ;
- **aucun** contrôle de rôle dans les routes E2EE : `grep -n "admin\|role" backend/src/e2ee.rs` → **0** occurrence hors fixtures de test (`role: "user"`, lignes 1258/1359/2398) ;
- → **n'importe quel membre authentifié** peut appeler `distribute_group_keys` et faire avancer la version de la clé de groupe.

Ce n'est **pas** une absence de concept d'admin (correction par rapport au relevé de la revue) : c'est un **contrôle manquant** sur 3 routes précises. La réserve porte donc sur **deux** choses distinctes : (a) l'absence de *job* — assumée, (b) l'absence de *contrôle de rôle* — écart de spec, assumé et routé en carte.

### Ce qui n'est **pas** en cause

`POST /api/auth/rotate-key` (`backend/src/e2ee.rs:319-321`) **n'est pas** la rotation de clé de groupe : c'est la rotation de la **clé personnelle** de l'utilisateur (`users.public_key` → `user_key_version`, première rotation = version 1, `e2ee.rs:176-190`). L'ADR-016 ne doit pas être lu comme implémentant cette route.

---

## 2. R2 — Purge des anciennes versions : **ASSUMÉE** (conserve illimitée, aucun expiry)

### Mesure

| Affirmation | Preuve |
|---|---|
| `get_my_group_key` accepte n'importe quelle version | `MyKeyQuery { version: Option<i32> }` (`backend/src/e2ee.rs:130`) · requête paramétrée sans borne : `WHERE ... AND ckr.version = ?` (`backend/src/e2ee.rs:677-700`) |
| Aucune purge des clés de groupe | `backend/src/prune.rs` nettoie `message_keys` (`:22`), `message_reactions` (`:32`), `messages` (`:45`), `uploads` (`:84`), `conversation_participants` (`:111`), `conversations` (`:133`), `invites` (`:150`), `chess_games` (`:166`) — **jamais** `conversation_keys` ni `conversation_key_recipients` |
| Aucun expiry côté client | `frontend/src/lib/e2ee.ts` : **aucune** logique d'expiration (aucun `expir`, aucune comparaison de date) |

### Décision

**Assumée** : les anciennes versions de clé de groupe sont **conservées indéfiniment** et **restent récupérables**. Il n'existe **aucun** mécanisme d'expiry — ni côté serveur, ni côté client.

### Précision qui change le sens de la réserve

La revue `security-auditor` §3.6 présentait ce point comme *« résolu côté client par l'expiration »*. **C'est faux** : le client n'expire rien. La fenêtre de récupération d'une clé est donc **illimitée**, pas bornée par le client. La réserve n'est pas « l'expiry n'est pas appliqué », elle est « il n'y a **pas** d'expiry du tout ».

### Impact mesuré (et il est faible)

Conséquence : un membre **renvoyé** d'une conversation peut, tant que son compte existe, redemander **n'importe quelle version** antérieure de la clé de groupe et déchiffrer l'historique correspondant — y compris après son départ. C'est le **contraire** de la forward secrecy partielle revendiquée par RFC-016 §5.4:180 et ADR-016 §4.

**Ce que la purge seule ne corrige pas** : elle bornerait la *profondeur* récupérable (ex. 3 versions), mais ne ferme pas l'accès aux versions conservées. La fermeture réelle de l'accès post-départ exige de **vérifier l'appartenance courante au moment de la lecture**, pas seulement de purger. Point acté, hors périmètre docs.

---

## 3. R3 — *(découverte)* `add_member_key` ne pose pas `distribution_status` : **ASSUMÉE** + carte `@coder`

Non listée dans la carte d'origine. Détectée en mesurant R2.

### Mesure

- `add_member_key` (`backend/src/e2ee.rs:734`) fait un `INSERT OR REPLACE INTO conversation_key_recipients (conversation_id, version, user_id, encrypted_key, user_key_version)` (`e2ee.rs:780`) — la colonne `distribution_status` est **absente** de la liste.
- La colonne existe pourtant, avec défaut `delivered` : `ALTER TABLE conversation_key_recipients ADD COLUMN distribution_status TEXT NOT NULL DEFAULT 'delivered'` (`backend/migrations/022_distribution_status.sql:4`).
- `claim_key` (`backend/src/e2ee.rs:834`) ne sert **que** les entrées `pending` : `WHERE ... AND distribution_status = 'pending'` (`e2ee.rs:845`).

### Conséquence

Une clé ajoutée à un membre **sans** clé publique via ce chemin naît donc `delivered` → `claim_key` ne la servira **jamais**. Le cas « membre sans clé » prévu par ADR-016 §Option D / SEC-FIX-3 (distribution différée + statut `pending` + « reclame ta clé ») est **structurellement hors d'atteinte** par ce chemin d'ajout.

### Décision

**Assumée** — même famille que R2 (distribution des clés) → même traitement : l'ADR décrit la réalité (`add_member_key` = livraison immédiate, `pending` uniquement via `distribute_group_keys`), et l'écart est routé en carte `@coder` avec R2 (une seule passe sur la distribution).

---

## 4. Suivi — les 2 cartes `@coder` et la dette technique

| # | Contenu | Déclencheur | Fichier |
|---|---------|-------------|---------|
| **1** | Job de rotation périodique **ou** suppression de `rotateGroupKey()` (dead code) + contrôle de rôle admin sur `distribute_group_keys` / `add_member_key` / `get_my_group_key` | R1 | `backend/src/e2ee.rs`, `backend/src/main.rs`, `backend/src/config.rs` |
| **2** | Purge des anciennes versions (`prune.rs` ou migration) **+** correctif `distribution_status` dans `add_member_key` | R2 + R3 | `backend/src/prune.rs`, `backend/src/e2ee.rs:780` |

Les deux cartes sont créées par cette carte et référencées dans `docs/adr/README.md` §7.

**Dette technique** : les 3 écarts rejoignent **DT-05** (`e2ee.rs` + `e2ee.ts` — E2EE partiellement implémenté, priorité 🟡 HIGH). Aucun nouveau DT : ils sont les **écarts de spec** de DT-05, pas des dettes nouvelles.

---

## 5. Statut d'ADR-016 après ce tranchage

**`Accepté`.** Les 2 réserves de la revue ne sont plus **ouvertes sur la décision** : elles sont fermées en « politique réelle ». Les **3 CRITIQUES** de la revue (`SEC-FIX-1/2/3`) restent **levées en code** — vérifié, non supposé :

| Réserve de la revue | État |
|---|---|
| `SEC-FIX-1` — validation de l'appartenance avant insertion | ✅ `add_member_key` valide `body.userId` : `backend/src/e2ee.rs:759-776` → 403 si non-membre |
| `SEC-FIX-2` — rate limiting E2EE | ✅ limiter dédié : `backend/src/main.rs:501`, `backend/src/main.rs:612` |
| `SEC-FIX-3` — routes de distribution | ✅ `claim-key` (`e2ee.rs:834`), `my-key-status`, `group-key-metadata` (`e2ee.rs:959`) · `distribution_status` (migration 022) |
| Réserve majeure 1 — rotation | ➖ **ASSUMÉE** (manuelle) · écart rôle acté |
| Réserve majeure 2 — purge | ➖ **ASSUMÉE** (illimitée) |
| Réserve majeure 3 — *(découverte)* `distribution_status` | ➖ **ASSUMÉE** + carte `@coder` |

---

## 6. Ce que ce document ne fait pas

- **Pas de code** : le code est hors périmètre de la carte (docs). Les écarts vont en cartes `@coder` (§4).
- **Pas de nouvelle décision d'architecture** : ce document **ferme** des réserves ; il ne choisit pas une nouvelle politique. Chaque « assumer » est un constat de ce qui **est**, appuyé sur `fichier:ligne`.
- **Pas de réécriture rétroactive d'ADR 001-010/013-015** : décision déjà actée (`t_db6f9481`, `t_566014c5`).

---

*architect · carte `t_08632b95` · 2026-10-22 · branche `docs/adr-016-reserves-t_08632b95`*
