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
| `SEC-FIX-3` — routes de distribution | ✅ `claim-key` (`e2ee.rs:834`), `my-key-status`, `group-key-metadata` (`e2ee.rs:961`) · `distribution_status` (migration 022) |
| Réserve majeure 1 — rotation | ➖ **ASSUMÉE** (manuelle) · écart rôle acté |
| Réserve majeure 2 — purge | ➖ **ASSUMÉE** (illimitée) |
| Réserve majeure 3 — *(découverte)* `distribution_status` | ➖ **ASSUMÉE** + carte `@coder` |

---

## 6. Ce que ce document ne fait pas

- **Pas de code** : le code est hors périmètre de la carte (docs). Les écarts vont en cartes `@coder` (§4).
- **Pas de nouvelle décision d'architecture** : ce document **ferme** des réserves ; il ne choisit pas une nouvelle politique. Chaque « assumer » est un constat de ce qui **est**, appuyé sur `fichier:ligne`.
- **Pas de réécriture rétroactive d'ADR 001-010/013-015** : décision déjà actée (`t_db6f9481`, `t_566014c5`).

---

# Tranchage E1 — `get_my_group_key` : restreindre au créateur/admin (volet B RFC-016 §5.4) ou l'exclure ?

**Date** : 2026-10-23
**Décideur** : architect (carte kanban `t_e8af977c`)
**Objet** : la liste de **3 routes** du volet B de l'écart **E1** (`DECISIONS.md` §4 #1, `docs/adr/README.md:270`) — `distribute_group_keys`, `add_member_key`, `get_my_group_key` — appliquée **littéralement** par le coder sur la branche `fix/e2ee-admin-gate-t_83868105` (`ac3727ad`).
**Méthode** : **mesure de code** sur `develop` (HEAD `8aec3d3d`, synchro `origin/develop`) et sur la branche (`ac3727ad`). Chaque affirmation est adossée à `fichier:ligne`.
**Périmètre** : **docs + tranchage**. Aucun code touché (voir §6).

---

## 1. Décision

> **Option (a) retenue : `get_my_group_key` est EXCLUS du volet B.**

Le garde « créateur du groupe ou admin » (et, pour `default_global`, « admin uniquement ») s'applique **uniquement** à :
- `POST /api/conversations/{id}/keys` → `distribute_group_keys`
- `POST /api/conversations/{id}/add-member-key` → `add_member_key`

`GET /api/conversations/{id}/my-key` → `get_my_group_key` reste en **auto-lecture membre**, bornée à `check_membership` (membre courant) + **clé scellée pour l'appelant**.

**Options (b) et (c) rejetées** — justification en §3.

---

## 2. Justification — trois mesures de code, pas une interprétation de spec

### M1 — Le nom de la route dit ce qu'elle fait, et la requête le prouve

`get_my_group_key` ne prend **aucun** paramètre de cible : ni `userId`, ni `version` obligatoire.

```
Query(MyKeyQuery { version: Option<i32> })          → backend/src/e2ee.rs:130-132
... WHERE ckr.conversation_id = ? AND ckr.user_id = ?   → backend/src/e2ee.rs:690 (versionnée) / :707 (dernière)
    .bind(&user.id)                                     → backend/src/e2ee.rs:693 / :711
```

Le `user_id` de la requête est **toujours** `user.id` — l'appelant. Un administrateur ne peut **pas** lire la clé d'un autre membre via cette route. Il n'y a donc **aucun pouvoir d'administration** à retirer : restreindre la route au créateur/admin supprime un droit **sans rien retirer** à un admin.

Le paramètre `version: Option<i32>` sert à l'**auto-lecture d'une version antérieure** de **sa propre** clé — c'est exactement le cas E2 (conserve illimitée, `DECISIONS.md` §2) vu côté client.

### M2 — `get_my_group_key` est le chemin de lecture qui supporte l'auto-lecture ET l'envoi

`frontend/src/lib/e2ee.ts:92` → `loadGroupKey()` appelle `GET .../my-key` (`:97-99`). Trois appelants :

| Appelant | Ligne | Rôle de l'appel |
|---|---|---|
| `decryptMessage()` | `frontend/src/lib/cryptoStore.svelte.ts:399` | **auto-lecture** : déchiffrer un message reçu (format `group_key_version`) |
| `encryptMessage()` | `frontend/src/lib/cryptoStore.svelte.ts:368` | **envoi** : lire la clé pour **chiffrer** ses propres messages dans `default_global` |
| `file-transfer` | `frontend/src/lib/file-transfer.svelte.ts:128` | chiffrement des fichiers |

**Conséquence mesurée** : restreindre la route au créateur/admin casse **trois** fonctions, pas une. Un membre ordinaire de `default_global` ne pourrait plus **déchiffrer les messages qu'il a lui-même reçus** (régression E2EE), ni **envoyer** un message chiffré (blocage pur). La carte d'origine ne mentionnait que l'auto-lecture ; l'envoi est le même chemin.

### M3 — Le garde n'a jamais été un contrôle d'administration, c'est un contrôle d'**appartenance**

Avant la branche, `get_my_group_key` n'avait qu'un seul garde : `check_membership(&state, &user.id, &conv_id)` (`backend/src/e2ee.rs:154`, appelé en `:683`). Ce garde est déjà **correct pour l'auto-lecture** : il garantit qu'un **non-membre** (ex. un membre **renvoyé**) ne lit rien.

**Ce que ce garde ne couvre pas** — et que la carte d'origine a confondu avec le rôle — c'est l'**historique post-départ** : un membre renvoyé dont le compte existe peut redemander **n'importe quelle version** (`MyKeyQuery.version` non borné). C'est l'écart **E2**, déjà acté dans `DECISIONS.md` §2, dont la fermeture réelle exige de **borner les versions récupérables**, pas de filtrer sur le rôle.

> **Ne pas confondre** : E1 = « qui a le droit d'**agir** sur la clé » · E2 = « jusqu'où un **membre** peut remonter dans **sa** clé ». Ce sont deux contrôles, sur deux routes, avec deux remèdes.

---

## 3. Pourquoi (b) et (c) sont rejetés

**(b) Restreindre les 3 routes + chemin d'auto-lecture distinct.** Rejetée pour **3 raisons mesurées** :
1. Le chemin d'auto-lecture distinct n'**existe pas** : il faudrait créer une route (`my-key-self`) **en doublon** de `my-key`, ou dériver la clé localement (impossible : la clé est **scellée** par destinataire avec `crypto_box_seal`, `e2ee.ts:104` — seul le serveur peut la fournir, déjà chiffrée pour l'appelant).
2. C'est de la **complexité sans bénéfice** : l'auto-lecture est **déjà** bornée par `check_membership` + le scellement par destinataire (M1). Le doublon n'ajoute **aucune** protection ; il ajoute une route, des tests, et une surface d'attaque.
3. **Règle de simplicité** (mandat architecte) : « est-ce que ça rend l'installation Docker plus complexe ? » → non, mais « est-ce que ça ajoute une surface sans valeur ? » → oui → **bloqué**. Et l'ADR-016 est **Accepté** : sa décision (Option D) ne change pas ; (b) aurait imposé de **réécrire** une spec acceptée pour un cas que le code ne traite pas.

**(c) Autre** : rien de mesuré ne justifie une troisième forme. Le seul contrôle manquant réel (E2, historique post-départ) est déjà tracé et relève d'une **autre** route (borner `MyKeyQuery.version` / purger) — pas d'un filtre de rôle.

---

## 4. Corollaire — le code de `ac3727ad` est **déjà** l'option (a)

La carte d'origine affirme : *« le code de la branche applique le volet B **littéralement aux 3 routes** »*. **C'est faux**, et la mesure le montre. Sur `ac3727ad`, le garde est **paramétré** :

```rust
async fn require_group_key_access(state, user, conv_id, require_creator: bool)
```

| Route | Appel sur `ac3727ad` | Effet |
|---|---|---|
| `distribute_group_keys` | `require_group_key_access(..., true)` | créateur ou admin (groupe privé) · **admin seul** (`default_global`) |
| `add_member_key` | `require_group_key_access(..., true)` | idem |
| `get_my_group_key` | `require_group_key_access(..., false)` | `require_creator = false` → `return Ok(())` après `check_membership` → **auto-lecture membre conservée** |

Preuve : `get_my_group_key` sur `ac3727ad` appelle `require_group_key_access(&state, &user, &conv_id, false)`, et la branche `if !require_creator { return Ok(()) }` rend le garde **no-op** pour un membre non-groupe-global. La branche **implémente donc déjà (a)** — avec une réserve sur `default_global`, cf. §5.

**Conséquence pratique** : il n'y a **aucun** conflit de code à résoudre pour ce tranchage, et **aucun** réécriture en (a) à faire. La carte d'origine demandait de ne pas résoudre un conflit avant décision ; il n'y en a pas. La revue `security-auditor` de `fix/e2ee-admin-gate-t_83868105` peut reprendre.

---

## 5. Réserve actée — le seul point de (a) qui n'est pas anodin

Pour `default_global`, `require_group_key_access` **ne regarde pas** `require_creator` :

```rust
if conv_id == "default_global" {
    if user.role.as_str() != "admin" { return Err(StatusCode::FORBIDDEN); }
    return Ok(());
}
```

Donc (a) **restreint `get_my_group_key` sur `default_global` aux admins** — un membre ordinaire du groupe global **perd l'auto-lecture** sur **cette** conversation, alors qu'un membre d'un groupe privé la conserve. C'est une **asymétrie** : le « groupe global » est le cas d'usage principal de l'E2EE groupe (ADR-016, Option D).

**Décision de la réserve** : **assumée** — l'asymétrie est **réellement** dans le code (`ac3727ad`), donc on l'écrit plutôt que de la prétendre absente. Mais elle est **marquée comme écart à corriger**, pas comme une politique : pour `default_global`, `check_membership` suffit pour l'auto-lecture (M1/M3) ; l'exigence « admin » ne doit s'appliquer qu'aux routes qui **avancent** la clé.

→ **Carte `@coder`** (créée par ce tranchage) : sur `default_global`, `get_my_group_key` doit retomber sur l'auto-lecture membre (`check_membership` seul), le filtre admin restant réservé à `distribute_group_keys` / `add_member_key`. Fichiers : `backend/src/e2ee.rs` (route + garde). SANS cela, le groupe global perd l'E2EE pour ses membres non-admins.

---

## 6. Ce que ce document ne fait pas

- **Pas de code** : `backend/src/e2ee.rs` et `frontend/src/lib/e2ee.ts` sont **hors périmètre**. Le correctif de la réserve (§5) va en carte `@coder`.
- **Pas de résolution de conflit** : la branche `fix/e2ee-admin-gate-t_83868105` ne contient **aucun** marqueur de conflit (`grep -c '<<<<<<<'` → 0). Aucun rebase à l'aveugle.
- **Pas de fusion** : la branche n'est **pas** mergée. Seule la **doc** est poussée sur `develop`.
- **Pas de réécriture d'ADR-016** : la décision (Option D) est inchangée. Seule la **liste des routes** du volet B est corrigée, et l'amendement est **rétrocompatible** (il retire une restriction, il n'en ajoute pas).

---

## 7. Amendements appliqués

| Fichier | Changement | Raison |
|---|---|---|
| `docs/rfc/RFC-016-e2ee-global-hybrid-sender-keys.md` | **§5.4** : la phrase « créateur/admin » est **précisée** comme portant sur les routes qui **distribuent ou avancent** la clé ; `get_my_group_key` est explicité comme **auto-lecture membre** (non couverte par la règle) | la spec prêtait à confusion entre « distribuer » et « lire » |
| `docs/adr/016-e2ee-group-hybrid.md` | **§Écarts E1** : la liste des routes passe de **3 à 2** (`distribute_group_keys`, `add_member_key`) ; `get_my_group_key` renvoie à E2 | aligner l'ADR Accepté sur le réel mesuré |
| `docs/adr/README.md` | **§7 #11** : même correction (2 routes), + renvoi vers ce tranchage | l'index citait les 3 routes |

`get_my_group_key` **reste cité** dans ADR-016 §75 (« fondations déjà en place ») et dans RFC-016 `:37` (inventaire des routes) : ce sont des **inventaires**, pas la règle d'accès. Aucune correction nécessaire là.

---

## 8. Statut de l'écart E1 après ce tranchage

**Fermé en code** — la décision (a) est celle **déjà implémentée** sur `ac3727ad` pour les groupes privés, avec la réserve `default_global` (§5) ouverte et routée.

| Volet | État |
|---|---|
| E1 — `distribute_group_keys` | ✅ créateur/admin (privé) · admin (`default_global`) — `ac3727ad` |
| E1 — `add_member_key` | ✅ créateur/admin (privé) · admin (`default_global`) — `ac3727ad` |
| E1 — `get_my_group_key` (privé) | ✅ **auto-lecture membre** (décision (a)) — `ac3727ad` |
| E1 — `get_my_group_key` (`default_global`) | ⚠️ admin seul dans le code → **carte `@coder`** (§5) |
| E2 — versions non bornées / pas de purge | inchangé — écart distinct, déjà tracé (`DECISIONS.md` §2) |

---

*architect · carte `t_e8af977c` · 2026-10-23 · `develop` (HEAD `8aec3d3d`)*
